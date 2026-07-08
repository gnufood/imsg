use super::*;

/// `ExecStart` args always lead with the foreground subcommand, regardless of
/// which optional overrides are present.
#[test]
fn start_foreground_args_base_case() {
    assert_eq!(
        start_foreground_args(None, None),
        vec![OsString::from("daemon"), OsString::from("start"), OsString::from("--foreground"),]
    );
}

/// `--device`/`--config` are appended in order, matching `imsg daemon start`'s own
/// flag names so the installed service invokes the identical CLI surface.
#[test]
fn start_foreground_args_with_overrides() {
    let args = start_foreground_args(Some("AA:BB:CC:DD:EE:FF"), Some(Path::new("/tmp/c.toml")));
    assert_eq!(
        args,
        vec![
            OsString::from("daemon"),
            OsString::from("start"),
            OsString::from("--foreground"),
            OsString::from("--device"),
            OsString::from("AA:BB:CC:DD:EE:FF"),
            OsString::from("--config"),
            OsString::from("/tmp/c.toml"),
        ]
    );
}

/// Round-trips through [`SmServiceStatus`] preserve the stopped-reason payload.
#[test]
fn service_state_from_status_preserves_stopped_reason() {
    assert_eq!(ServiceState::from(SmServiceStatus::NotInstalled), ServiceState::NotInstalled);
    assert_eq!(ServiceState::from(SmServiceStatus::Running), ServiceState::Running);
    assert_eq!(
        ServiceState::from(SmServiceStatus::Stopped(Some("crashed".to_owned()))),
        ServiceState::Stopped(Some("crashed".to_owned()))
    );
}
