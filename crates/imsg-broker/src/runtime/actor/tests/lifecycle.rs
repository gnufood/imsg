//! Connect/retry/reconnect lifecycle tests, against fake MAP connectors only — PBAP session
//! tests live in [`super::pbap_session`].

use super::*;

#[tokio::test]
async fn reaches_active_then_shuts_down_when_handle_dropped() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let h = spawn(
        fake_connector(),
        fake_pbap_connector(),
        store,
        Some(Duration::from_secs(60)),
        test_policy(),
    );
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Active)).await?;
    drop(h.handle);
    let mut shutdown = h.shutdown;
    tokio::time::timeout(Duration::from_secs(3), shutdown.changed()).await??;
    assert!(matches!(*shutdown.borrow(), Some(TerminalReason::Requested)));
    Ok(())
}

#[tokio::test]
async fn failed_connect_goes_terminal() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let h = spawn(
        failing_connector(),
        fake_pbap_connector(),
        store,
        Some(Duration::from_secs(60)),
        test_policy(),
    );
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Failed(_))).await?;
    Ok(())
}

/// A connect phase that gives up for good must publish [`TerminalReason::PermanentFailure`]
/// rather than [`TerminalReason::Requested`], so the daemon can exit non-zero on it instead
/// of looking identical to a clean stop.
#[tokio::test]
async fn failed_connect_reports_permanent_failure() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let h = spawn(
        failing_connector(),
        fake_pbap_connector(),
        store,
        Some(Duration::from_secs(60)),
        test_policy(),
    );
    let mut shutdown = h.shutdown;
    tokio::time::timeout(Duration::from_secs(3), shutdown.changed()).await??;
    assert!(matches!(*shutdown.borrow(), Some(TerminalReason::PermanentFailure(_))));
    Ok(())
}

#[tokio::test]
async fn idle_timeout_shuts_down_when_some() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let h = spawn(
        fake_connector(),
        fake_pbap_connector(),
        store,
        Some(Duration::from_millis(50)),
        test_policy(),
    );
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Active)).await?;
    let mut shutdown = h.shutdown;
    tokio::time::timeout(Duration::from_secs(3), shutdown.changed()).await??;
    assert!(matches!(*shutdown.borrow(), Some(TerminalReason::Requested)));
    Ok(())
}

/// Daemon's persistent mode: `idle: None` must never fire the idle timeout, however long the
/// actor sits without a [`DeviceOp`].
#[tokio::test]
async fn no_idle_timeout_when_none() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let h = spawn(fake_connector(), fake_pbap_connector(), store, None, test_policy());
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Active)).await?;
    let mut shutdown = h.shutdown;
    let outcome = tokio::time::timeout(Duration::from_millis(200), shutdown.changed()).await;
    assert!(outcome.is_err(), "actor shut down despite idle: None");
    Ok(())
}

/// `test_policy()` only tolerates 2 attempts; a connector that fails 3 times must still
/// exhaust it and go terminal, regardless of `idle`.
#[tokio::test]
async fn bounded_policy_still_gives_up_past_cap() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let h = spawn(
        flaky_connector(3),
        fake_pbap_connector(),
        store,
        Some(Duration::from_secs(60)),
        test_policy(),
    );
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Failed(_))).await?;
    Ok(())
}

/// Daemon's persistent connect policy (`startup_budget: None`, `max_attempts: u32::MAX`)
/// must survive far more transient failures than the CLI-bounded `test_policy()` would
/// tolerate — the whole point of a daemon started before the phone is in Bluetooth range.
/// A `PermissionDenied` connect failure (e.g. a kernel-refused configured `security_level`)
/// must go terminal on the very first attempt, even under the daemon's unbounded policy —
/// otherwise a permanently unsatisfiable security requirement would retry forever instead
/// of failing fast like [`failed_connect_reports_permanent_failure`] proves for a bounded
/// policy.
#[tokio::test]
async fn permanent_classification_fails_fast_under_unbounded_policy() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let policy = ConnectPolicy {
        initial_backoff: Duration::from_millis(1),
        max_backoff: Duration::from_millis(2),
        max_attempts: u32::MAX,
        startup_budget: None,
    };
    let h = spawn(
        permission_denied_connector(),
        fake_pbap_connector(),
        store,
        Some(Duration::from_secs(60)),
        policy,
    );
    let mut shutdown = h.shutdown;
    tokio::time::timeout(Duration::from_millis(500), shutdown.changed()).await??;
    assert!(matches!(*shutdown.borrow(), Some(TerminalReason::PermanentFailure(_))));
    Ok(())
}

#[tokio::test]
async fn unbounded_policy_survives_past_bounded_cap() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let policy = ConnectPolicy {
        initial_backoff: Duration::from_millis(1),
        max_backoff: Duration::from_millis(2),
        max_attempts: u32::MAX,
        startup_budget: None,
    };
    let h = spawn(
        flaky_connector(3),
        fake_pbap_connector(),
        store,
        Some(Duration::from_secs(60)),
        policy,
    );
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Active)).await?;
    Ok(())
}
