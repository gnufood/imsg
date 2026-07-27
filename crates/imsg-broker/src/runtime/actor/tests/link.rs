//! Physical-link liveness: the actor must learn the device is gone from the transport itself,
//! not only from an operation that happens to fail against it.

use super::*;

/// Persistent (daemon) policy: unbounded attempts, no startup deadline.
fn persistent_policy() -> ConnectPolicy {
    ConnectPolicy {
        initial_backoff: Duration::from_millis(1),
        max_backoff: Duration::from_millis(2),
        max_attempts: u32::MAX,
        startup_budget: None,
    }
}

/// A watcher whose first subscription ends immediately and whose second reports `Down`.
fn link_ends_then_reports_down() -> LinkWatcher {
    let mut calls = 0_u32;
    Box::new(move || {
        let first = calls == 0;
        calls = calls.saturating_add(1);
        Box::pin(async move {
            let events: LinkEvents = if first {
                Box::pin(futures::stream::empty())
            } else {
                Box::pin(futures::stream::once(async { LinkState::Down }))
            };
            events
        })
    })
}

/// A link stream that ends means the transport stopped reporting, not that the link is healthy.
///
/// The actor must resubscribe. Reaching `Reconnecting` here is only possible via the *second*
/// subscription, so passing proves the end of the first stream was not swallowed — the failure
/// mode this mirrors is the MNS arm, which used to disable itself on stream end and never
/// recover.
#[tokio::test]
async fn link_stream_end_resubscribes() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let h = spawn(
        connect_once_then_hang(),
        fake_pbap_connector(),
        link_ends_then_reports_down(),
        store,
        None,
        persistent_policy(),
    );
    let mut state = h.state.clone();
    tokio::time::timeout(
        Duration::from_secs(3),
        state.wait_for(|s| matches!(s, ConnState::Reconnecting)),
    )
    .await??;
    Ok(())
}

/// A link-down report with no traffic in flight must drop the session.
///
/// Without this the actor stays `Active` indefinitely: it only ever learns of a dead link
/// mid-op, and a daemon in steady state issues no ops — so nothing ever probes, and `Status`
/// keeps answering `Active` over a link that is physically gone.
#[tokio::test]
async fn link_down_while_idle_drops_session() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let (link_tx, link) = link_channel();
    let h = spawn(
        connect_once_then_hang(),
        fake_pbap_connector(),
        link,
        store,
        None,
        persistent_policy(),
    );
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Active)).await?;

    link_tx.send(LinkState::Down).await?;

    tokio::time::timeout(
        Duration::from_secs(3),
        state.wait_for(|s| matches!(s, ConnState::Reconnecting)),
    )
    .await??;
    Ok(())
}
