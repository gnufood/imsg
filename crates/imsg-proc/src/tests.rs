//! `respawn_args` is pure and tested directly. `respawn_self` always spawns `current_exe()`,
//! which under `cargo test` is this crate's own test-harness binary — real, no mock, but its
//! libtest arg parser rejects the `--device`/`--config` tail we unconditionally append, so it
//! always exits non-zero with a short, deterministic stderr message and no stdout. That's
//! enough to exercise the whole real pipeline for real (log dir/file creation, permissions,
//! argv assembly, spawn, stderr redirection) but not to distinguish `capture_stdout`'s two
//! branches by content — both `bool`/`detach` values are still exercised for regression safety
//! (no panic, spawns either way), matching the existing project precedent of not verifying
//! process-group/fd-capture side effects directly (see `imsg-gui/src/daemon/provision.rs`'s
//! former `spawn_headless`).

use std::path::Path;

use super::*;

#[test]
fn respawn_args_appends_device_and_skips_config_when_absent() -> anyhow::Result<()> {
    let args = respawn_args(&["daemon", "start", "--foreground"], "AA:BB", None)?;
    assert_eq!(args, ["daemon", "start", "--foreground", "--device", "AA:BB"]);
    Ok(())
}

#[test]
fn respawn_args_appends_config_when_present() -> anyhow::Result<()> {
    let args = respawn_args(&["__broker_serve"], "AA:BB", Some(Path::new("/tmp/imsg.toml")))?;
    assert_eq!(args, ["__broker_serve", "--device", "AA:BB", "--config", "/tmp/imsg.toml"]);
    Ok(())
}

#[cfg(unix)]
#[test]
fn respawn_args_rejects_non_utf8_config_path() -> anyhow::Result<()> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt as _;

    let bad = OsStr::from_bytes(&[0xFF, 0xFE]);
    let Err(err) = respawn_args(&["daemon"], "AA:BB", Some(Path::new(bad))) else {
        return Err(anyhow::anyhow!("expected an error for a non-UTF-8 config path"));
    };
    assert!(err.to_string().contains("UTF-8"));
    Ok(())
}

#[tokio::test]
async fn respawn_self_spawns_and_writes_a_real_log_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let log_path = dir.path().join("nested").join("child.log");

    let mut child =
        respawn_self(&["--help"], "AA:BB:CC:DD:EE:FF", None, &log_path, true, true).await?;
    child.wait().await?;

    let meta = std::fs::metadata(&log_path)?;
    assert!(meta.len() > 0, "log file should capture the child's stderr output");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }
    Ok(())
}

#[tokio::test]
async fn respawn_self_without_capture_or_detach_still_spawns() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let log_path = dir.path().join("child.log");

    let mut child =
        respawn_self(&["__broker_serve"], "AA:BB:CC:DD:EE:FF", None, &log_path, false, false)
            .await?;
    let status = child.wait().await?;

    assert!(!status.success(), "the test harness rejects the appended --device flag");
    assert!(log_path.exists());
    Ok(())
}
