//! Unit tests for [`super::bind_or_exit`] and [`super::serve_actor`].

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName as _};
use secrecy::SecretBox;

use super::*;

const MAP_CONNECT_RSP: &[u8] =
    include_bytes!("../../../../imsg-obex/tests/fixtures/connect_rsp.bin");
const NOTIF_REG_OK: &[u8] = &[0xA0, 0x00, 0x03];
// OBEX CONNECT OK response with a ConnectionId header (same fixture bytes as imsg-pbap's
// `tests/fixtures/pbap_connect_rsp.bin`).
const PBAP_CONNECT_RSP: &[u8] = &[
    0xa0, 0x00, 0x1f, 0x10, 0x00, 0x0f, 0xa0, 0xcb, 0xdd, 0x20, 0x40, 0xd0, 0x4a, 0x00, 0x13, 0x79,
    0x61, 0x35, 0xf0, 0xf0, 0xc5, 0x11, 0xd8, 0x09, 0x66, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

/// PBAP connector whose every call answers CONNECT only — `serve_actor` never drives it
/// further in this test, which exercises idle-wiring, not `SyncContacts` dispatch.
fn fake_pbap_connector() -> PbapConnector<tokio::io::DuplexStream> {
    Box::new(|| {
        Box::pin(async {
            let (client_io, server_io) = tokio::io::duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.next().await;
                t.send(Bytes::from_static(PBAP_CONNECT_RSP)).await.ok();
            });
            pbap_core::client::PbapClient::connect(client_io)
                .await
                .map_err(session::SessionError::from)
        })
    })
}

/// Connector whose every call yields a fresh minimal fake OBEX server over an in-memory
/// duplex stream, mirroring [`make_connector`]'s contract without touching real Bluetooth.
fn fake_connector() -> Connector<tokio::io::DuplexStream> {
    Box::new(|| {
        Box::pin(async {
            let (client_io, server_io) = tokio::io::duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.send(Bytes::from_static(MAP_CONNECT_RSP)).await.ok();
                t.next().await; // consume SetNotificationRegistration
                t.send(Bytes::from_static(NOTIF_REG_OK)).await.ok();
                while t.next().await.is_some() {
                    t.send(Bytes::from_static(NOTIF_REG_OK)).await.ok();
                }
            });
            session::lifecycle::establish_map_session(client_io).await
        })
    })
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

/// `serve_daemon`'s reason to exist: unlike `serve`, it must never let the actor's idle
/// timeout fire, however long the daemon sits with no `DeviceOp`. Regression guard for the
/// "`None` wiring silently reverted to `Some(cfg.broker.idle())`" bug class the daemon
/// caller exists to prevent — exercises the same connector-generic core `serve_daemon` runs
/// through, `serve_actor`, so it doesn't depend on a real MAP session.
#[tokio::test]
async fn serve_actor_never_exits_on_idle_when_none() -> anyhow::Result<()> {
    let name = "imsg/broker/test-serve-actor-idle-none".to_ns_name::<GenericNamespaced>()?;
    let listener = ListenerOptions::new().name(name).create_tokio()?;
    let (store, _dir) = fake_store().await?;

    let task = tokio::spawn(async move {
        serve_actor(
            Connectors { map: fake_connector(), pbap: fake_pbap_connector() },
            store,
            None,
            test_policy(),
            &listener,
            "test-device".to_owned(),
            Duration::from_millis(50),
        )
        .await
    });

    let outcome = tokio::time::timeout(Duration::from_millis(300), task).await;
    assert!(outcome.is_err(), "serve_actor returned despite idle: None");
    Ok(())
}

/// The kernel produces `EADDRINUSE` when the abstract name is already bound.
///
/// `bind_or_exit` maps this to `process::exit(0)`. This test verifies the invariant it
/// relies on without calling `exit` (which would kill the test process).
#[tokio::test]
async fn abstract_name_election_is_atomic() -> anyhow::Result<()> {
    let n1 = "imsg/broker/FE:ED:DE:AD:00:01".to_ns_name::<GenericNamespaced>()?;
    let n2 = "imsg/broker/FE:ED:DE:AD:00:01".to_ns_name::<GenericNamespaced>()?;
    let _l1 = ListenerOptions::new().name(n1).create_tokio()?;
    let Err(err) = ListenerOptions::new().name(n2).create_tokio() else {
        return Err(anyhow::anyhow!("second bind to a held abstract name unexpectedly succeeded"));
    };
    assert_eq!(err.kind(), std::io::ErrorKind::AddrInUse);
    Ok(())
}

#[test]
fn security_from_config_none_stays_none() {
    assert_eq!(security_from_config(None), None);
}

#[test]
fn security_from_config_maps_each_level() {
    for (cfg_level, bluer_level) in [
        (config::SecurityLevel::Sdp, bluer::rfcomm::SecurityLevel::Sdp),
        (config::SecurityLevel::Low, bluer::rfcomm::SecurityLevel::Low),
        (config::SecurityLevel::Medium, bluer::rfcomm::SecurityLevel::Medium),
        (config::SecurityLevel::High, bluer::rfcomm::SecurityLevel::High),
    ] {
        let expected = bluer::rfcomm::Security { level: bluer_level, key_size: 0 };
        assert_eq!(security_from_config(Some(cfg_level)), Some(expected));
    }
}

/// The abstract name is released the instant the listener is dropped.
///
/// This is the anti-regression for the entire stale-socket bug class: with filesystem
/// sockets a crash leaves an inode behind; with abstract sockets the kernel cleans up
/// atomically on any exit.
#[tokio::test]
async fn abstract_name_released_on_listener_drop() -> anyhow::Result<()> {
    let n1 = "imsg/broker/FE:ED:DE:AD:00:02".to_ns_name::<GenericNamespaced>()?;
    let n2 = "imsg/broker/FE:ED:DE:AD:00:02".to_ns_name::<GenericNamespaced>()?;
    let l = ListenerOptions::new().name(n1).create_tokio()?;
    drop(l);
    // Would be EADDRINUSE if the abstract name leaked.
    ListenerOptions::new().name(n2).create_tokio()?;
    Ok(())
}
