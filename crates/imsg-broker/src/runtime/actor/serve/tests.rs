//! Unit tests for [`super::wants_mns`] and [`super::on_mns_event`].

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use map_core::mns_event::parse_event_report;
use secrecy::SecretBox;
use store::{Direction, NewMessage, Store};
use tokio::io::{duplex, DuplexStream};

use super::*;

const CONNECT_RSP: &[u8] =
    include_bytes!("../../../../../imsg-obex/tests/fixtures/connect_rsp.bin");

/// In-memory `Store` (temp-dir `SQLite`) plus the dir guard.
async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

/// A connected client whose fake server only answers CONNECT — sufficient for every MNS event
/// type except `NewMessage`, the only one that issues further MAP requests.
async fn fake_client() -> anyhow::Result<MapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(CONNECT_RSP)).await;
    });
    Ok(MapClient::connect(client_io).await?)
}

/// `on_mns_event` must both persist the event to the store and fan it out to `Watch`
/// subscribers, in one pass over the same event.
#[tokio::test]
async fn on_mns_event_writes_store_and_fans_out_watch_tx() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    store
        .upsert(NewMessage {
            map_handle: "H1".to_owned(),
            timestamp_ms: 0,
            folder: "telecom/msg/inbox".to_owned(),
            direction: Direction::Received,
            address: "+1".to_owned(),
            status: 0,
            synced_at: 0,
            text: "hi".to_owned(),
            outgoing_status: None,
        })
        .await?;
    let mut client = fake_client().await?;
    let (watch_tx, mut watch_rx) = broadcast::channel(4);
    let ev = parse_event_report(
        b"<MAP-event-report version='1.0'>\
          <event type='MessageDeleted' handle='H1' folder='telecom/msg/inbox'/>\
          </MAP-event-report>",
    )?;

    let outcome = on_mns_event(&ev, &mut client, &store, &watch_tx).await;

    assert!(matches!(outcome, OpOutcome::Continue));
    assert!(store.get_by_handle("H1").await?.is_none(), "MessageDeleted should remove the row");
    let got = watch_rx.try_recv()?;
    assert_eq!(got.event_type, "MessageDeleted");
    Ok(())
}

/// A fatal MAP transport error while fetching a `NewMessage` body must still fan the raw event
/// out to `Watch` subscribers — a live subscriber should learn a message arrived even though
/// the daemon is about to reconnect before it can store the body (regression: the fan-out used
/// to be skipped on the `SessionLost` early return).
#[tokio::test]
async fn on_mns_event_fans_out_even_on_fatal_error() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    // The fake server task ends after CONNECT, so the following SETPATH hits EOF — fatal.
    let mut client = fake_client().await?;
    let (watch_tx, mut watch_rx) = broadcast::channel(4);
    let ev = parse_event_report(
        b"<MAP-event-report version='1.0'>\
          <event type='NewMessage' handle='H1' folder='telecom/msg/inbox'/>\
          </MAP-event-report>",
    )?;

    let outcome = on_mns_event(&ev, &mut client, &store, &watch_tx).await;

    assert!(matches!(outcome, OpOutcome::SessionLost));
    let got = watch_rx.try_recv()?;
    assert_eq!(got.event_type, "NewMessage");
    Ok(())
}

#[test]
fn no_subscribers_ephemeral_does_not_want_mns() {
    assert!(!wants_mns(0, Some(Duration::from_secs(15))));
}

#[test]
fn no_subscribers_persistent_wants_mns() {
    assert!(wants_mns(0, None));
}

#[test]
fn subscribers_ephemeral_wants_mns() {
    assert!(wants_mns(1, Some(Duration::from_secs(15))));
}

#[test]
fn subscribers_persistent_wants_mns() {
    assert!(wants_mns(1, None));
}
