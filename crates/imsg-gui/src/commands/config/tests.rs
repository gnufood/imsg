//! Same `figment::Jail` isolation as `crate::config`'s own tests — these just prove the
//! `#[tauri::command]` shims delegate correctly and map errors through `CommandError`.

use serial_test::serial;

use super::*;

#[test]
#[serial]
fn config_show_returns_resolved_config() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        let dto = config_show().map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.device_address, "AA:BB:CC:DD:EE:FF");
        Ok(())
    });
}

#[test]
#[serial]
fn config_set_device_persists() {
    figment::Jail::expect_with(|jail| {
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        config_set_device("11:22:33:44:55:66".to_owned())
            .map_err(|e| figment::Error::from(e.to_string()))?;

        let dto = config_show().map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.device_address, "11:22:33:44:55:66");
        Ok(())
    });
}

#[test]
fn config_set_device_maps_invalid_mac_to_command_error() {
    let result = config_set_device("not-a-mac".to_owned());
    assert!(result.is_err());
}
