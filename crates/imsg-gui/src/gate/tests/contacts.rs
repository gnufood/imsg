//! The best-effort contacts-sync step: must never block the gate from reaching `Ready`.

use std::sync::Arc;

use ipc::BrokerResponse;
use serial_test::serial;

use super::*;

#[test]
#[serial]
fn run_reaches_ready_despite_contacts_sync_failure() -> anyhow::Result<()> {
    figment::Jail::expect_with(|jail| {
        let addr = "AA:BB:CC:DD:EE:15";
        jailed_env(jail, Some(addr));
        block_on(async move {
            let dir = tempfile::tempdir()?;
            let state = Arc::new(GateState::default());
            let mut rx = state.subscribe();
            let server = tokio::spawn(serve_sequence(
                bind_for(addr)?,
                vec![
                    status_info(true),
                    BrokerResponse::Text("synced".to_owned()),
                    BrokerResponse::Error("no pbap session".to_owned()),
                ],
            ));
            let ready = spawn_run(&state, counting_opener(dir.path(), Arc::default(), 0));

            wait_for(&mut rx, |s| *s == GateStatus::Ready, "Ready").await?;
            server.await??;
            assert!(snapshot(&ready).is_some(), "a contacts sync failure must not block entry");
            Ok(())
        })
        .map_err(|e| figment::Error::from(e.to_string()))
    });
    Ok(())
}
