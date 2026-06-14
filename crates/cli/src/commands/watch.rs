//! `watch` subcommand — MAP notification events via stdout or ratatui panel.

use anyhow::Result;
use config::Config;
use session::{mns, MnsEvent};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use transport::iroh::Endpoint;
use transport::rfcomm;

use crate::commands::conn;
#[cfg(not(feature = "tui"))]
use crate::output;

/// With `hub = true`, subscribes to the hub's MNS stream over iroh (stdout; TUI is RFCOMM-only).
/// Otherwise connects via RFCOMM and registers MNS with `BlueZ`; with `--features tui` renders
/// in ratatui. Does NOT reconnect on session drop.
///
/// # Errors
///
/// Returns error if `hub.node_key` is unset/invalid, MAP or hub connection fails, `BlueZ` MNS
/// registration fails, or the MNS task panics.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
) -> Result<()> {
    if let Some(ep) = endpoint {
        return super::watch_hub::run(cfg, ep).await;
    }
    #[cfg(feature = "tui")]
    {
        return run_tui(cfg, device).await;
    }
    #[cfg(not(feature = "tui"))]
    run_plain(cfg, device).await
}

/// Formats one MNS event as a single stdout line: `<EventType>  handle=<h>  folder=<f>`.
/// Absent handle or folder renders as `-`.
pub(crate) fn format_event(event: &MnsEvent) -> String {
    format!(
        "{:?}  handle={}  folder={}",
        event.event_type(),
        event.handle().unwrap_or("-"),
        event.folder().unwrap_or("-"),
    )
}

/// Registers the MNS RFCOMM profile with `BlueZ` and spawns the MNS accept loop.
///
/// Returns `Err` immediately if `BlueZ` profile registration fails.
/// The returned handle's inner `Result` always resolves to `Ok(())` on clean exit.
async fn spawn_mns_task(
    event_tx: mpsc::Sender<MnsEvent>,
    cancel_rx: watch::Receiver<bool>,
) -> Result<JoinHandle<Result<()>>> {
    let listener = rfcomm::listen_mns().await?;
    Ok(tokio::spawn(async move {
        mns::run_mns_session(listener, event_tx, cancel_rx).await;
        Ok(())
    }))
}

#[cfg(not(feature = "tui"))]
async fn run_plain(cfg: &Config, device: Option<&str>) -> Result<()> {
    let (cancel_tx, cancel_rx) = watch::channel(false);
    let (event_tx, mut event_rx) = mpsc::channel::<MnsEvent>(32);

    let mut client = conn::connect_map(cfg, None, device).await?;
    // Hold the MAP connection alive; iOS drops notification registration on OBEX DISCONNECT.
    let _hold = tokio::spawn(async move { client.hold().await });

    let mns_handle = spawn_mns_task(event_tx, cancel_rx).await?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            maybe = event_rx.recv() => match maybe {
                Some(event) => output::line(&format_event(&event))?,
                None => break,
            }
        }
    }

    let _ = cancel_tx.send(true);
    // Tokio's SIGINT handler is permanent once installed; without this arm a second
    // Ctrl+C during mns_handle.await is absorbed with no effect.
    let join_result = tokio::select! {
        r = mns_handle => r,
        _ = tokio::signal::ctrl_c() => return Ok(()),
    };
    match join_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(e) => return Err(anyhow::anyhow!("MNS task panicked: {e}")),
    }
    Ok(())
}

/// Exits on Ctrl+C, `q`, `Esc`, or MNS close; does not reconnect.
/// Calls `ratatui::restore()` on all exit paths.
#[cfg(feature = "tui")]
async fn run_tui(cfg: &Config, device: Option<&str>) -> Result<()> {
    let (cancel_tx, cancel_rx) = watch::channel(false);
    let (event_tx, mut event_rx) = mpsc::channel::<MnsEvent>(32);
    let (key_tx, mut key_rx) = mpsc::channel::<crossterm::event::KeyEvent>(8);

    let mut client = conn::connect_map(cfg, None, device).await?;
    // Hold the MAP connection alive; iOS drops notification registration on OBEX DISCONNECT.
    let _hold = tokio::spawn(async move { client.hold().await });

    let mns_handle = spawn_mns_task(event_tx, cancel_rx).await?;

    let mut terminal = ratatui::init();

    // std::thread::spawn (not spawn_blocking) — long-lived loop; 50 ms poll lets it notice key_tx closing.
    std::thread::spawn(move || {
        use std::time::Duration;
        loop {
            match crossterm::event::poll(Duration::from_millis(50)) {
                Ok(true) => match crossterm::event::read() {
                    Ok(crossterm::event::Event::Key(k)) if k.is_press() => {
                        if key_tx.blocking_send(k).is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("crossterm::event::read error: {e}");
                        break;
                    }
                },
                Ok(false) => {
                    if key_tx.is_closed() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("crossterm::event::poll error: {e}");
                    break;
                }
            }
        }
    });

    let loop_result = run_loop(&mut terminal, &mut event_rx, &mut key_rx).await;

    ratatui::restore();
    let _ = cancel_tx.send(true);
    // Tokio's SIGINT handler is permanent once installed; without this arm a Ctrl+C
    // during mns_handle.await is absorbed with no effect.
    let join_result = tokio::select! {
        r = mns_handle => r,
        _ = tokio::signal::ctrl_c() => return loop_result,
    };
    match join_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(e) => return Err(anyhow::anyhow!("MNS task panicked: {e}")),
    }
    loop_result
}

/// Ctrl+C handled as `KeyCode::Char('c')` + `CONTROL` — raw mode clears `ISIG`, so no SIGINT.
#[cfg(feature = "tui")]
async fn run_loop(
    terminal: &mut ratatui::DefaultTerminal,
    event_rx: &mut mpsc::Receiver<MnsEvent>,
    key_rx: &mut mpsc::Receiver<crossterm::event::KeyEvent>,
) -> Result<()> {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    const MAX_EVENTS: usize = 500;
    let mut events: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    let mut state = ratatui::widgets::ListState::default();

    loop {
        terminal.draw(|f| render(f, &events, &mut state))?;

        tokio::select! {
            maybe = event_rx.recv() => match maybe {
                Some(ev) => {
                    if events.len() == MAX_EVENTS {
                        events.pop_front();
                    }
                    events.push_back(format_event(&ev));
                    let last = events.len().saturating_sub(1);
                    state.select(Some(last));
                }
                None => break,
            },
            maybe = key_rx.recv() => match maybe {
                Some(KeyEvent { code: KeyCode::Char('c'), modifiers, .. })
                    if modifiers.contains(KeyModifiers::CONTROL) => break,
                Some(KeyEvent { code: KeyCode::Down, .. }) => {
                    let sel = state.selected().unwrap_or(0);
                    let last = events.len().saturating_sub(1);
                    state.select(Some(sel.saturating_add(1).min(last)));
                }
                Some(KeyEvent { code: KeyCode::Up, .. }) => {
                    let sel = state.selected().unwrap_or(0);
                    state.select(Some(sel.saturating_sub(1)));
                }
                Some(KeyEvent { code: KeyCode::Char('q') | KeyCode::Esc, .. }) | None => break,
                Some(_) => {}
            },
        }
    }

    Ok(())
}

/// Scrollable event list above a one-line status bar.
/// `state.selected()` out-of-bounds is safe — ratatui clips it.
#[cfg(feature = "tui")]
fn render(
    frame: &mut ratatui::Frame,
    events: &std::collections::VecDeque<String>,
    state: &mut ratatui::widgets::ListState,
) {
    use ratatui::layout::{Constraint, Layout};
    use ratatui::style::Stylize as _;
    use ratatui::widgets::{Block, List, Paragraph};

    let areas = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(frame.area());

    let Some(list_area) = areas.first().copied() else { return };
    let Some(status_area) = areas.get(1).copied() else { return };

    let list = List::new(events.iter().map(String::as_str))
        .block(Block::bordered().title(" imsg watch "))
        .highlight_style(ratatui::style::Style::new().reversed());
    frame.render_stateful_widget(list, list_area, state);

    let status = Paragraph::new("  ↑/↓ scroll  q quit  Ctrl+C exit").dim();
    frame.render_widget(status, status_area);
}

#[cfg(all(test, feature = "tui"))]
mod tests {
    use super::render;
    use anyhow::Result;
    use ratatui::{backend::TestBackend, Terminal};

    fn make_terminal() -> Result<Terminal<TestBackend>> {
        Ok(Terminal::new(TestBackend::new(80, 24))?)
    }

    #[test]
    fn render_empty_events() -> Result<()> {
        let mut terminal = make_terminal()?;
        let events: std::collections::VecDeque<String> = std::collections::VecDeque::new();
        let mut state = ratatui::widgets::ListState::default();
        terminal.draw(|f| render(f, &events, &mut state))?;
        Ok(())
    }

    #[test]
    fn render_with_events() -> Result<()> {
        let mut terminal = make_terminal()?;
        let events: std::collections::VecDeque<String> = [
            "NewMessage  handle=1  folder=TELECOM/MSG/INBOX".to_owned(),
            "MessageDeleted  handle=2  folder=TELECOM/MSG/SENT".to_owned(),
            "SendingSuccess  handle=3  folder=TELECOM/MSG/SENT".to_owned(),
        ]
        .into();
        let mut state = ratatui::widgets::ListState::default();
        terminal.draw(|f| render(f, &events, &mut state))?;
        Ok(())
    }

    #[test]
    fn render_selected() -> Result<()> {
        let mut terminal = make_terminal()?;
        let events: std::collections::VecDeque<String> = [
            "NewMessage  handle=1  folder=TELECOM/MSG/INBOX".to_owned(),
            "MessageDeleted  handle=2  folder=TELECOM/MSG/SENT".to_owned(),
            "SendingSuccess  handle=3  folder=TELECOM/MSG/SENT".to_owned(),
        ]
        .into();
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(1));
        terminal.draw(|f| render(f, &events, &mut state))?;
        assert_eq!(state.selected(), Some(1));
        Ok(())
    }
}
