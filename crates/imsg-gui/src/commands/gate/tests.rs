//! Shim-level tests: the managed [`GateState`] round-trips through the command surface, and
//! `gate_proceed` genuinely wakes a parked `gate::run`. Loop behavior itself is covered by
//! `gate/tests.rs` — the wake test here stops at the loop's re-publish, needing no broker or
//! store.

use std::time::Duration;

use serial_test::serial;
use tauri::Manager as _;

use crate::gate::GateStatus;

use super::*;

const WAIT: Duration = Duration::from_secs(15);

/// Never reached — the jail has no device config, so `run` parks before opening a store.
fn failing_opener(_cfg: &config::Config) -> crate::gate::OpenStoreFuture<'_> {
    Box::pin(std::future::ready(Err(crate::reads::Error::NoDataDir)))
}

#[tokio::test]
async fn gate_status_reports_the_managed_state() {
    let app = tauri::test::mock_app();
    app.manage(GateState::default());

    assert_eq!(gate_status(app.state::<GateState>()), GateStatus::Initializing);
}

#[test]
#[serial]
fn gate_proceed_wakes_a_parked_run() -> anyhow::Result<()> {
    figment::Jail::expect_with(|jail| {
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());
        tokio::runtime::Runtime::new().map_err(|e| figment::Error::from(e.to_string()))?.block_on(
            async {
                let app = tauri::test::mock_app();
                app.manage(GateState::default());
                let mut rx = app.state::<GateState>().subscribe();
                let handle = app.handle().clone();
                tokio::spawn(async move {
                    let state = handle.state::<GateState>();
                    crate::gate::run(&state, None, failing_opener, |_| {}).await;
                });

                tokio::time::timeout(WAIT, rx.wait_for(|s| *s == GateStatus::AwaitingDeviceConfig))
                    .await
                    .map_err(|_| figment::Error::from("timed out waiting for park"))?
                    .map_err(|e| figment::Error::from(e.to_string()))?;

                // `send_replace` notifies on every publish, so the next `changed()` can only fire
                // once the woken loop re-runs and re-parks — proof the poke reached it.
                rx.borrow_and_update();
                gate_proceed(app.state::<GateState>());
                tokio::time::timeout(WAIT, rx.changed())
                    .await
                    .map_err(|_| figment::Error::from("timed out waiting for wake"))?
                    .map_err(|e| figment::Error::from(e.to_string()))?;
                assert_eq!(app.state::<GateState>().status(), GateStatus::AwaitingDeviceConfig);
                Ok(())
            },
        )
    });
    Ok(())
}
