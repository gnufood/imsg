use std::path::PathBuf;
use std::time::Duration;

use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName as _};

use super::{connect_retry, probe};

/// `probe` is `false` when nothing is listening at the abstract name.
#[tokio::test]
async fn probe_is_false_when_unreachable() {
    assert!(!probe("imsg-broker-client-test/probe-nobody-home").await);
}

/// `probe` is `true` once a listener is bound at the abstract name `probe` derives from `addr`.
#[tokio::test]
async fn probe_is_true_when_listening() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:02:01";
    let name = config::broker_abstract_name(addr)?;
    let _listener = ListenerOptions::new().name(name).create_tokio()?;
    assert!(probe(addr).await);
    Ok(())
}

/// `connect_retry` succeeds once the abstract socket becomes connectable.
///
/// A background task binds the socket after 100 ms — the retry loop must discover
/// it within the 5 s deadline without a mock or a startup handshake.
#[tokio::test]
async fn connect_retry_reaches_deferred_listener() -> anyhow::Result<()> {
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let name = "imsg/broker/FE:ED:DE:AD:00:03".to_ns_name::<GenericNamespaced>()?;
        let _l = ListenerOptions::new().name(name).create_tokio()?;
        // Keep listener alive long enough for the retry loop to connect.
        tokio::time::sleep(Duration::from_secs(2)).await;
        Ok::<(), anyhow::Error>(())
    });

    let log = PathBuf::from("/dev/null");
    let mut child = tokio::process::Command::new("sleep").arg("10").spawn()?;

    connect_retry(
        "FE:ED:DE:AD:00:03",
        &mut child,
        &log,
        Duration::from_secs(5),
        Duration::from_millis(25),
    )
    .await?;
    let _ = child.kill().await;
    Ok(())
}

/// `connect_retry` returns `Err` immediately when the child exits before binding.
///
/// `true` exits with code 0 instantly; the socket `FE:ED:DE:AD:00:04` is never bound,
/// so the only outcome is the child-exit arm of the `select!`.
#[tokio::test]
async fn connect_retry_fails_on_broker_exit() -> anyhow::Result<()> {
    let log = PathBuf::from("/dev/null");
    let mut child = tokio::process::Command::new("true").spawn()?;

    let result = connect_retry(
        "FE:ED:DE:AD:00:04",
        &mut child,
        &log,
        Duration::from_secs(5),
        Duration::from_millis(25),
    )
    .await;
    let Err(err) = result else {
        return Err(anyhow::anyhow!(
            "connect_retry should fail when the broker exits before binding"
        ));
    };
    let msg = err.to_string();
    assert!(
        msg.contains("exited during startup"),
        "expected 'exited during startup' in error, got: {msg}"
    );
    Ok(())
}
