//! Unit tests for [`super::bind_or_exit`] and [`super::serve_actor`].

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName as _};
use secrecy::SecretBox;

use super::*;

const MAP_CONNECT_RSP: &[u8] =
    include_bytes!("../../../../imsg-obex/tests/fixtures/connect_rsp.bin");
const NOTIF_REG_OK: &[u8] = &[0xA0, 0x00, 0x03];

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
        startup_budget: Duration::from_secs(5),
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
            fake_connector(),
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
