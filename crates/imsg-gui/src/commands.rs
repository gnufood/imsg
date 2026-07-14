//! `#[tauri::command]` shims — the invoke-handler boundary a `tauri_specta::Builder` wraps.
//!
//! Split by domain (`reads`, `config`, `daemon`, `send`, `delete`) to stay under the
//! 250-line module ceiling as the command surface grows; `CommandError` (shared across every
//! domain) lives here.

pub mod config;
pub mod daemon;
pub mod delete;
pub mod reads;
pub mod send;

use serde::Serialize;
use specta::Type;

/// Failure surfaced to the frontend from any `#[tauri::command]` in this crate.
///
/// Wraps the underlying error's `Display` text; none of the sources converted below leak
/// secrets (no raw key material, no `SQLite`/service-manager internals beyond a driver-level
/// description).
#[derive(Debug, Serialize, Type, thiserror::Error)]
#[error("{message}")]
pub struct CommandError {
    message: String,
}

impl From<store::Error> for CommandError {
    fn from(err: store::Error) -> Self {
        Self { message: err.to_string() }
    }
}

// Fully qualified: this module's own `pub mod config;` shadows the `config` crate name for
// unqualified references within this file.
impl From<::config::ConfigError> for CommandError {
    fn from(err: ::config::ConfigError) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<service::Error> for CommandError {
    fn from(err: service::Error) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<crate::daemon::StopError> for CommandError {
    fn from(err: crate::daemon::StopError) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<crate::daemon::provision::ProvisionError> for CommandError {
    fn from(err: crate::daemon::provision::ProvisionError) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<broker_client::WriteError> for CommandError {
    fn from(err: broker_client::WriteError) -> Self {
        Self { message: err.to_string() }
    }
}

/// Registers every `#[tauri::command]` in this crate on a fresh `tauri_specta::Builder`.
///
/// Single source of truth for the exported command surface — reused by the `bindings.ts`
/// export step (`examples/export_bindings.rs`) and by `main.rs`'s own `invoke_handler`/
/// `mount_events` wiring once that exists, so the list is never hand-duplicated.
///
/// `i64`/`u64` fields (e.g. `MessageDto::timestamp_ms`) export as lossless JS `bigint` via
/// semantic types — `specta_typescript` otherwise hard-forbids exporting them at all
/// (precision loss past 2^53 in a plain JS `number`). Only the generated `bindings.ts`
/// wrappers apply the runtime conversion; calling raw `invoke()` bypasses it.
#[must_use]
pub fn builder<R: tauri::Runtime>() -> tauri_specta::Builder<R> {
    tauri_specta::Builder::<R>::new()
        .commands(tauri_specta::collect_commands![
            reads::get_by_handle,
            reads::list_messages,
            reads::mark_read,
            reads::threads,
            config::config_show,
            config::config_set_device,
            config::config_set_map_channel,
            config::config_set_pbap_channel,
            daemon::daemon_install,
            daemon::daemon_uninstall,
            daemon::daemon_status,
            daemon::daemon_stop,
            daemon::daemon_restart,
            daemon::broker_status,
            send::send,
            delete::delete,
        ])
        .semantic_types(
            specta_typescript::semantic::Configuration::default().enable_lossless_bigints(),
        )
}

#[cfg(test)]
mod tests;
