//! Tests `CommandError`'s conversions directly (no fakes needed — pure data), plus one smoke
//! test proving the actual wiring every command in this module tree is built for —
//! `collect_commands!` + `Builder::invoke_handler` — type-checks and constructs against every
//! real command path, on Tauri's `MockRuntime`. Per-domain command behavior is exercised in
//! each submodule's own `tests.rs` (`reads`, `config`, `daemon`, `send`, `delete`).

use secrecy::SecretBox;
use store::Store;
use tauri_specta::{collect_commands, Builder};

use super::CommandError;

#[tokio::test]
async fn missing_store_read_error_maps_to_command_error() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    // Point at a path whose parent directory doesn't exist, so `Store::open` never succeeds
    // and its error can flow through the same `From<store::Error>` conversion the commands use.
    let bad_path = dir.path().join("missing-parent").join("test.db");
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let Err(open_err) = Store::open(bad_path, key).await else {
        return Err(anyhow::anyhow!("expected open to fail for a missing parent dir"));
    };

    let command_err = CommandError::from(open_err);
    assert!(!command_err.message.is_empty());
    Ok(())
}

#[test]
fn config_error_maps_to_command_error() {
    let err = ::config::ConfigError::Invalid { field: "device.address", msg: "bad".to_owned() };
    let command_err = CommandError::from(err);
    assert!(!command_err.message.is_empty());
}

#[test]
fn service_error_maps_to_command_error() {
    let err = service::Error::NoInvokingUser;
    let command_err = CommandError::from(err);
    assert!(!command_err.message.is_empty());
}

#[test]
fn stop_error_maps_to_command_error() {
    let err = crate::daemon::StopError::Rejected("no coordinator".to_owned());
    let command_err = CommandError::from(err);
    assert!(!command_err.message.is_empty());
}

#[test]
fn write_error_maps_to_command_error() {
    let err = broker_client::WriteError::Call(broker_client::CallError::Error("x".to_owned()));
    let command_err = CommandError::from(err);
    assert!(!command_err.message.is_empty());
}

/// Proves the actual wiring every command in this crate is built for — `collect_commands!` +
/// `Builder::invoke_handler` — type-checks and constructs against real command paths, on
/// Tauri's `MockRuntime`. Full IPC-frame dispatch (`tauri::test::assert_ipc_response`) is left
/// for the `main.rs` integration once that's wired up; this only guards the registration seam.
#[test]
fn commands_register_with_tauri_specta_builder() {
    let builder = Builder::<tauri::test::MockRuntime>::new().commands(collect_commands![
        super::reads::get_by_handle,
        super::reads::list_messages,
        super::reads::mark_read,
        super::reads::threads,
        super::config::config_show,
        super::config::config_set_device,
        super::daemon::daemon_install,
        super::daemon::daemon_uninstall,
        super::daemon::daemon_status,
        super::daemon::daemon_stop,
        super::daemon::broker_status,
        super::send::send,
        super::delete::delete,
    ]);
    let _invoke_handler = builder.invoke_handler();
}
