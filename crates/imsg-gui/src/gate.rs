//! The startup gate: one Rust-owned loop, spawned once by `main.rs`.
//!
//! Brings the app from cold start to a ready `Store` in the CLI's own order — `config::load` →
//! [`daemon::provision::ensure_running`](crate::daemon::provision::ensure_running) →
//! open store → [`sync::ensure_synced`](crate::sync::ensure_synced) →
//! [`contacts::ensure_synced_best_effort`](crate::contacts::ensure_synced_best_effort).
//!
//! The frontend never drives this sequence. Its whole surface is: poll the current
//! [`GateStatus`] (`commands::gate::gate_status`), supply device config through the ordinary
//! thin config commands, and poke [`GateState::proceed`] to say "conditions may have changed,
//! re-evaluate" (`commands::gate::gate_proceed`). Every pass re-derives everything from ground
//! truth, and every step is idempotent, so a stray or repeated poke is harmless.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use serde::Serialize;
use specta::Type;
use store::Store;
use tokio::sync::{watch, Notify};

/// Which startup step a [`GateStatus::Failed`] belongs to. Opening the store is folded into
/// `Daemon` — both are "getting the local daemon/store side ready", and the frontend renders
/// them identically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
pub enum GateStage {
    /// Self-provisioning the daemon or opening the local store.
    Daemon,
    /// Ensuring the store has completed its first sync.
    Sync,
}

/// The startup gate's current position — the single source of truth the frontend polls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub enum GateStatus {
    /// [`run`] hasn't reached its first config check yet.
    Initializing,
    /// No valid device config exists; parked until [`GateState::proceed`] after one is persisted.
    AwaitingDeviceConfig,
    /// `ensure_running` (or the store open that follows it) is in flight.
    StartingDaemon,
    /// `ensure_synced` is in flight.
    Syncing,
    /// `stage` failed; parked until [`GateState::proceed`]. Retrying re-runs from the top.
    Failed {
        /// The step that failed.
        stage: GateStage,
        /// The failure's `Display` text — same flat-string convention as `CommandError`.
        message: String,
    },
    /// The gate completed; the `Store` has been handed to `on_ready` and the app is usable.
    Ready,
}

/// Tauri-managed handle shared between [`run`] (the writer) and the two thin poll commands.
#[derive(Debug)]
pub struct GateState {
    status: watch::Sender<GateStatus>,
    proceed: Notify,
}

impl Default for GateState {
    fn default() -> Self {
        Self { status: watch::Sender::new(GateStatus::Initializing), proceed: Notify::new() }
    }
}

impl GateState {
    /// Returns the current status. Authoritative at any time — a remounted frontend just polls
    /// again and lands on the current state.
    #[must_use]
    pub fn status(&self) -> GateStatus {
        self.status.borrow().clone()
    }

    /// Wakes a parked [`run`] (or stores one wakeup if it isn't parked yet). Carries no
    /// information about *what* to do — the loop re-derives that itself.
    pub fn proceed(&self) {
        self.proceed.notify_one();
    }

    // Test-only: production consumers poll `status()`; only tests need change notification.
    #[cfg(test)]
    pub(crate) fn subscribe(&self) -> watch::Receiver<GateStatus> {
        self.status.subscribe()
    }

    fn set(&self, status: GateStatus) {
        self.status.send_replace(status);
    }

    /// Publishes `status` and waits for the next [`Self::proceed`].
    async fn park(&self, status: GateStatus) {
        self.set(status);
        self.proceed.notified().await;
    }
}

/// Future returned by [`run`]'s injected store-opener.
pub type OpenStoreFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Store, crate::reads::Error>> + Send + 'a>>;

/// Runs the startup gate until it completes, publishing progress through `state`.
///
/// On any failure it parks at the failing step's status and re-runs from the top after the next
/// [`GateState::proceed`] — an already-opened `Store` is kept across retries rather than
/// reopened. On success the `Store` is handed to `on_ready` (in production:
/// `app.manage(store)`) and the loop exits.
///
/// `open_store` is injected (production: `reads::open_store`) rather than called directly: the
/// real opener goes through the OS keyring, whose availability is host state (a locked or
/// absent Secret Service blocks on a prompt — indistinguishable from a hang), so tests stay
/// hermetic by substituting a direct `Store::open` with a fixed key, the same approach every
/// `Store`-needing test in this crate already uses.
pub async fn run<F, R>(
    state: &GateState,
    config_path: Option<PathBuf>,
    mut open_store: F,
    on_ready: R,
) where
    F: for<'a> FnMut(&'a config::Config) -> OpenStoreFuture<'a>,
    R: FnOnce(Store),
{
    let mut store: Option<Store> = None;
    loop {
        let Ok(cfg) = config::load(config_path.clone()) else {
            state.park(GateStatus::AwaitingDeviceConfig).await;
            continue;
        };
        state.set(GateStatus::StartingDaemon);
        if let Err(e) =
            crate::daemon::provision::ensure_running(&cfg, None, config_path.clone()).await
        {
            state.park(failed(GateStage::Daemon, &e)).await;
            continue;
        }
        let db = match store.take() {
            Some(db) => db,
            None => match open_store(&cfg).await {
                Ok(db) => db,
                Err(e) => {
                    state.park(failed(GateStage::Daemon, &e)).await;
                    continue;
                }
            },
        };
        state.set(GateStatus::Syncing);
        if let Err(e) = crate::sync::ensure_synced(&db, cfg.device.address()).await {
            store = Some(db);
            state.park(failed(GateStage::Sync, &e)).await;
            continue;
        }
        crate::contacts::ensure_synced_best_effort(cfg.device.address()).await;
        state.set(GateStatus::Ready);
        on_ready(db);
        return;
    }
}

fn failed(stage: GateStage, err: &dyn std::fmt::Display) -> GateStatus {
    GateStatus::Failed { stage, message: err.to_string() }
}

#[cfg(test)]
mod tests;
