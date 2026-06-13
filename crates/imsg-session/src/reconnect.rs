//! Reconnect policy: back-off timing and retry logic for dropped RFCOMM sessions.

use std::future::Future;
use std::time::Duration;

use map_core::client::MapClient;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::watch;
use tokio_retry::strategy::{jitter, ExponentialBackoff};

use crate::SessionError;

/// Disconnected → Connecting → Active → Reconnecting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Initial state; no connection attempted yet.
    Disconnected,
    /// RFCOMM connect + OBEX handshake ongoing.
    Connecting,
    /// OBEX CONNECT + `SetNotificationRegistration` done; session live.
    Active,
    /// Session dropped; backoff running before retry.
    Reconnecting,
}

/// Exponential backoff (2 s → 60 s, ×2, with jitter). Retries indefinitely.
pub async fn run_map_session(
    addr: bluer::Address,
    channel: u8,
    state: watch::Sender<SessionState>,
    cancel: watch::Receiver<bool>,
) {
    let strategy = ExponentialBackoff::from_millis(2)
        .factor(1000)
        .max_delay(Duration::from_secs(60))
        .map(jitter);
    run_session_loop(move || crate::lifecycle::connect_map(addr, channel), strategy, state, cancel)
        .await;
}

/// Inner reconnect loop. Resets backoff to start after each successful session.
pub(crate) async fn run_session_loop<F, Fut, T, S>(
    make_client: F,
    strategy: S,
    state: watch::Sender<SessionState>,
    mut cancel: watch::Receiver<bool>,
) where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<MapClient<T>, SessionError>>,
    T: AsyncRead + AsyncWrite + Unpin,
    S: Clone + Iterator<Item = Duration>,
{
    let mut delays = strategy.clone();

    while !*cancel.borrow() {
        let _ = state.send(SessionState::Connecting);
        tracing::info!("MAP: connecting");

        let stop = match make_client().await {
            Ok(client) => {
                delays = strategy.clone();
                serve_active(client, &state, &mut cancel).await
            }
            Err(e) => {
                tracing::warn!("MAP: connect failed: {e}");
                backoff(&mut delays, &state, &mut cancel).await
            }
        };
        if stop {
            break;
        }
    }
    let _ = state.send(SessionState::Disconnected);
}

/// Hold session until drop or cancel. Returns `true` to stop loop, `false` to retry.
async fn serve_active<T>(
    mut client: MapClient<T>,
    state: &watch::Sender<SessionState>,
    cancel: &mut watch::Receiver<bool>,
) -> bool
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let _ = state.send(SessionState::Active);
    tracing::info!("MAP: session active");
    if interrupted(client.hold(), cancel).await {
        return true;
    }
    let _ = state.send(SessionState::Reconnecting);
    tracing::warn!("MAP: session dropped, reconnecting");
    false
}

/// Wait one backoff delay. Returns `true` when strategy exhausted or cancelled.
async fn backoff<S: Iterator<Item = Duration>>(
    delays: &mut S,
    state: &watch::Sender<SessionState>,
    cancel: &mut watch::Receiver<bool>,
) -> bool {
    let Some(delay) = delays.next() else { return true };
    let _ = state.send(SessionState::Reconnecting);
    interrupted(tokio::time::sleep(delay), cancel).await
}

/// Race `until` against cancellation. Returns `true` when cancel fired.
async fn interrupted<Fut: Future>(until: Fut, cancel: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        _ = until => false,
        res = cancel.changed() => res.is_err() || *cancel.borrow(),
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use futures::{SinkExt, StreamExt};
    use std::time::Duration;
    use tokio::sync::watch;

    use crate::{lifecycle, SessionError, SessionState};

    use super::run_session_loop;

    const MAP_CONNECT_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/connect_rsp.bin");
    const NOTIF_REG_OK: &[u8] = &[0xA0, 0x00, 0x03];

    #[tokio::test]
    async fn exits_when_strategy_exhausted() {
        let (state_tx, _state_rx) = watch::channel(SessionState::Disconnected);
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let strategy = std::iter::once(Duration::ZERO);
        let make_client = || async {
            Err::<map_core::client::MapClient<tokio::io::DuplexStream>, _>(SessionError::Transport(
                obex_core::TransportError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "refused",
                )),
            ))
        };
        run_session_loop(make_client, strategy, state_tx, cancel_rx).await;
        let _ = cancel_tx.send(false);
    }

    #[tokio::test]
    async fn exits_on_cancel_before_connect() {
        let (state_tx, _state_rx) = watch::channel(SessionState::Disconnected);
        let (cancel_tx, cancel_rx) = watch::channel(true);
        let strategy = std::iter::repeat(Duration::ZERO);
        let make_client = || async {
            Err::<map_core::client::MapClient<tokio::io::DuplexStream>, _>(SessionError::Transport(
                obex_core::TransportError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "refused",
                )),
            ))
        };
        run_session_loop(make_client, strategy, state_tx, cancel_rx).await;
        let _ = cancel_tx;
    }

    #[tokio::test]
    async fn enters_active_and_reconnects_on_stream_close() {
        let (state_tx, mut state_rx) = watch::channel(SessionState::Disconnected);
        let (cancel_tx, cancel_rx) = watch::channel(false);

        let (client_io, server_io) = tokio::io::duplex(4096);
        let client_cell = std::cell::Cell::new(Some(client_io));

        let make_client = move || {
            let io = client_cell.take().unwrap_or_else(|| {
                let (io, _srv) = tokio::io::duplex(1);
                io
            });
            async move { lifecycle::establish_map_session(io).await }
        };

        let strategy = std::iter::repeat(Duration::ZERO);

        let (server_result, ()) = futures::join!(
            async {
                let mut srv = obex_core::wrap(server_io);
                let _ = srv.next().await;
                srv.send(Bytes::from_static(MAP_CONNECT_RSP))
                    .await
                    .map_err(SessionError::Transport)?;
                let _ = srv.next().await;
                srv.send(Bytes::from_static(NOTIF_REG_OK))
                    .await
                    .map_err(SessionError::Transport)?;
                state_rx.wait_for(|s| *s == SessionState::Active).await.map_err(|_| {
                    SessionError::Transport(obex_core::TransportError::Io(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "watch dropped",
                    )))
                })?;
                drop(srv);
                state_rx.wait_for(|s| *s == SessionState::Reconnecting).await.map_err(|_| {
                    SessionError::Transport(obex_core::TransportError::Io(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "watch dropped",
                    )))
                })?;
                cancel_tx.send(true).ok();
                Ok::<(), SessionError>(())
            },
            run_session_loop(make_client, strategy, state_tx, cancel_rx),
        );
        assert!(server_result.is_ok());
    }

    #[tokio::test]
    async fn state_is_connecting_before_connect_attempt() {
        let (state_tx, mut state_rx) = watch::channel(SessionState::Disconnected);
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let strategy = std::iter::once(Duration::ZERO);

        let make_client = move || async {
            Err::<map_core::client::MapClient<tokio::io::DuplexStream>, _>(SessionError::Transport(
                obex_core::TransportError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "refused",
                )),
            ))
        };

        let ((), ()) = futures::join!(
            async {
                state_rx.wait_for(|s| *s == SessionState::Connecting).await.ok();
                cancel_tx.send(true).ok();
            },
            run_session_loop(make_client, strategy, state_tx, cancel_rx),
        );
    }
}
