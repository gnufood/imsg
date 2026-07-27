//! Shared fake-OBEX-server fixtures for the device actor's connect/reconnect/session tests.

mod lifecycle;
mod link;
mod pbap_session;

use std::sync::{Arc, Mutex};

use bytes::Bytes;
use futures::future::BoxFuture;
use futures::{SinkExt as _, StreamExt as _};
use obex_core::headers::Header;
use obex_core::packet::{OpCode, Packet, PacketExtra};
use secrecy::SecretBox;
use session::SessionError;
use tokio::io::duplex;

use super::*;
use crate::runtime::types::{no_link_events, LinkEvents, LinkState, LinkWatcher};

const MAP_CONNECT_RSP: &[u8] =
    include_bytes!("../../../../../imsg-obex/tests/fixtures/connect_rsp.bin");
const NOTIF_REG_OK: &[u8] = &[0xA0, 0x00, 0x03];
const GENERIC_OK: &[u8] = &[0xA0, 0x00, 0x03];
// OBEX CONNECT OK response with a ConnectionId header (same fixture bytes as imsg-pbap's
// `tests/fixtures/pbap_connect_rsp.bin`).
const PBAP_CONNECT_RSP: &[u8] = &[
    0xa0, 0x00, 0x1f, 0x10, 0x00, 0x0f, 0xa0, 0xcb, 0xdd, 0x20, 0x40, 0xd0, 0x4a, 0x00, 0x13, 0x79,
    0x61, 0x35, 0xf0, 0xf0, 0xc5, 0x11, 0xd8, 0x09, 0x66, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

/// Builds a PBAP connector whose every call yields a `PbapClient<DuplexStream>` backed by a
/// fresh fake OBEX server that only answers CONNECT — none of the current actor-lifecycle
/// tests drive it past that.
fn fake_pbap_connector() -> PbapConnector<tokio::io::DuplexStream> {
    Box::new(|| {
        Box::pin(async {
            let (client_io, server_io) = duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.next().await;
                t.send(Bytes::from_static(PBAP_CONNECT_RSP)).await.ok();
            });
            pbap_core::client::PbapClient::connect(client_io).await.map_err(SessionError::from)
        })
    })
}

/// Encodes a single-response OBEX OK packet carrying `body` as `EndOfBody`.
fn ok_body_packet(body: &[u8]) -> anyhow::Result<Bytes> {
    Ok(Packet {
        opcode: OpCode::Ok,
        extra: PacketExtra::None,
        headers: vec![Header::EndOfBody(Bytes::copy_from_slice(body))],
    }
    .encode()?)
}

/// Serves one `sync_contacts` request cycle (metadata probe, then an empty listing) on an
/// already-CONNECTed OBEX transport — the two requests `session::contacts::sync_contacts`
/// issues when the device reports no `DatabaseIdentifier` (always-refresh branch) and the
/// phonebook is empty, so no `pull` follows the listing.
async fn serve_one_empty_sync_cycle(t: &mut obex_core::ObexTransport<tokio::io::DuplexStream>) {
    t.next().await;
    if let Ok(pkt) = ok_body_packet(&[]) {
        t.send(pkt).await.ok();
    }
    t.next().await;
    if let Ok(pkt) = ok_body_packet(b"<vCard-listing></vCard-listing>") {
        t.send(pkt).await.ok();
    }
}

/// Builds a connector whose every call yields a `MapClient<DuplexStream>` backed by a fresh
/// minimal fake OBEX server, mirroring the production connector's contract.
fn fake_connector() -> Connector<tokio::io::DuplexStream> {
    Box::new(|| {
        Box::pin(async {
            let (client_io, server_io) = duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.send(Bytes::from_static(MAP_CONNECT_RSP)).await.ok();
                t.next().await; // consume SetNotificationRegistration
                t.send(Bytes::from_static(NOTIF_REG_OK)).await.ok();
                while t.next().await.is_some() {
                    t.send(Bytes::from_static(GENERIC_OK)).await.ok();
                }
            });
            session::lifecycle::establish_map_session(client_io).await
        })
    })
}

/// A connector that always fails with a transient transport error.
fn failing_connector() -> Connector<tokio::io::DuplexStream> {
    Box::new(|| {
        Box::pin(async {
            Err(SessionError::Transport(obex_core::TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "no link",
            ))))
        })
    })
}

/// A connector that always fails with a permanent, security-refused transport error —
/// what a kernel-rejected `BT_SECURITY` level surfaces as (`EACCES` → `PermissionDenied`).
fn permission_denied_connector() -> Connector<tokio::io::DuplexStream> {
    Box::new(|| {
        Box::pin(async {
            Err(SessionError::Transport(obex_core::TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "BT_SECURITY level refused",
            ))))
        })
    })
}

/// Fails transiently `fails` times, then succeeds like [`fake_connector`]. Models a device
/// that's briefly out of Bluetooth range before it connects.
fn flaky_connector(mut fails: u32) -> Connector<tokio::io::DuplexStream> {
    Box::new(move || {
        if fails > 0 {
            fails = fails.saturating_sub(1);
            return Box::pin(async {
                Err(SessionError::Transport(obex_core::TransportError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "no link yet",
                ))))
            });
        }
        Box::pin(async {
            let (client_io, server_io) = duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.send(Bytes::from_static(MAP_CONNECT_RSP)).await.ok();
                t.next().await;
                t.send(Bytes::from_static(NOTIF_REG_OK)).await.ok();
                while t.next().await.is_some() {
                    t.send(Bytes::from_static(GENERIC_OK)).await.ok();
                }
            });
            session::lifecycle::establish_map_session(client_io).await
        })
    })
}

/// Connects successfully once, then never completes another connect.
///
/// Parks the actor in `Reconnecting` after a drop, so the transition is observable without
/// racing a successful reconnect straight back to `Active`.
fn connect_once_then_hang() -> Connector<tokio::io::DuplexStream> {
    let mut connected = false;
    Box::new(move || {
        if connected {
            return Box::pin(std::future::pending());
        }
        connected = true;
        Box::pin(async {
            let (client_io, server_io) = duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.send(Bytes::from_static(MAP_CONNECT_RSP)).await.ok();
                t.next().await;
                t.send(Bytes::from_static(NOTIF_REG_OK)).await.ok();
                while t.next().await.is_some() {
                    t.send(Bytes::from_static(GENERIC_OK)).await.ok();
                }
            });
            session::lifecycle::establish_map_session(client_io).await
        })
    })
}

/// A [`LinkWatcher`] driven by the returned sender: each value sent becomes one link event.
///
/// Only the first subscription is backed by the channel; later ones yield a never-ending
/// stream, so a test can assert what a resubscribe does without the sender feeding it.
fn link_channel() -> (mpsc::Sender<LinkState>, LinkWatcher) {
    let (tx, rx) = mpsc::channel(8);
    let slot = Arc::new(Mutex::new(Some(rx)));
    let watcher = Box::new(move || {
        let taken = slot.lock().map_or(None, |mut g| g.take());
        let fut: BoxFuture<'static, LinkEvents> = Box::pin(async move {
            let events: LinkEvents = match taken {
                Some(rx) => Box::pin(futures::stream::unfold(rx, |mut rx| async move {
                    rx.recv().await.map(|ev| (ev, rx))
                })),
                None => Box::pin(futures::stream::pending()),
            };
            events
        });
        fut
    });
    (tx, watcher)
}

/// In-memory `Store` (temp-dir `SQLite`) plus the dir guard.
async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

/// Fast policy for tests: millisecond backoff, two attempts.
fn test_policy() -> ConnectPolicy {
    ConnectPolicy {
        initial_backoff: Duration::from_millis(1),
        max_backoff: Duration::from_millis(2),
        max_attempts: 2,
        startup_budget: Some(Duration::from_secs(5)),
    }
}
