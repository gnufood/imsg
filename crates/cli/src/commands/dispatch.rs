//! Opt-in dispatch: routes each read/write command to its local-store or live-device
//! implementation based on `sync_enabled`.

use std::path::Path;

use anyhow::Result;
use transport::iroh::Endpoint;

use crate::commands::{contacts, get, list, send, threads};
use crate::progress::with_spinner;

/// Dispatches `list` to the local store or phone based on opt-in state.
pub(in crate::commands) async fn run_list(
    cfg: &::config::Config,
    spoke: Option<&Endpoint>,
    device: Option<&str>,
    opts: list::ListOpts,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if is_opted_in(db).await {
        with_spinner("listing", list::run_store(opts, db)).await
    } else {
        with_spinner("listing", list::run(cfg, spoke, device, opts, config_path)).await
    }
}

/// Dispatches `contacts` to a cache refresh, or to the local store or phone based on opt-in
/// state.
///
/// `--sync` bypasses the opt-in split entirely — it's a write, not a read, so it always runs
/// against the live device regardless of `sync_enabled`.
pub(in crate::commands) async fn run_contacts(
    cfg: &::config::Config,
    spoke: Option<&Endpoint>,
    device: Option<&str>,
    opts: contacts::ContactsOpts,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if opts.sync {
        let fut = contacts::run_sync(cfg, spoke, device, db, config_path);
        let count = with_spinner("syncing contacts", fut).await?;
        return Ok(format!("synced {count} contacts"));
    }
    if is_opted_in(db).await {
        with_spinner("contacts", contacts::run_store(&opts, db)).await
    } else {
        with_spinner("contacts", contacts::run(cfg, spoke, device, &opts, config_path)).await
    }
}

/// Dispatches `get` to the local store or phone based on opt-in state.
pub(in crate::commands) async fn run_get(
    cfg: &::config::Config,
    spoke: Option<&Endpoint>,
    device: Option<&str>,
    handle: String,
    mark_read: bool,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if is_opted_in(db).await {
        with_spinner("fetching", get::run_store(handle, mark_read, db)).await
    } else {
        with_spinner("fetching", get::run(cfg, spoke, device, handle, mark_read, config_path)).await
    }
}

/// Dispatches `threads` to the local store or phone based on opt-in state.
pub(in crate::commands) async fn run_threads(
    cfg: &::config::Config,
    spoke: Option<&Endpoint>,
    device: Option<&str>,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if is_opted_in(db).await {
        with_spinner("threads", threads::run_store(db)).await
    } else {
        with_spinner("threads", threads::run(cfg, spoke, device, config_path)).await
    }
}

/// Dispatches `send` to the store-backed outbox or a device-only push based on opt-in state.
///
/// Opted-in goes through `session::outbox::send_sms` (enqueue + push + reconcile); not opted-in
/// pushes to the device only, leaving no local outbox row.
pub(in crate::commands) async fn run_send(
    cfg: &::config::Config,
    spoke: Option<&Endpoint>,
    device: Option<&str>,
    number: String,
    message: String,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if is_opted_in(db).await {
        let fut = send::run(cfg, spoke, device, number, message, db, config_path);
        with_spinner("sending", fut).await
    } else {
        let fut = send::run_live(cfg, spoke, device, number, message, config_path);
        with_spinner("sending", fut).await
    }
}

/// Returns `true` when `sync_enabled = "true"` is set in the store `meta` table.
///
/// Any store error is treated as not opted in so the caller falls back to the phone path; a
/// warning is emitted so the failure is visible in logs.
async fn is_opted_in(store: &store::Store) -> bool {
    match store.get_meta("sync_enabled").await {
        Ok(v) => v.as_deref() == Some("true"),
        Err(e) => {
            tracing::warn!("failed to read sync_enabled from store, falling back to phone: {e}");
            false
        }
    }
}
