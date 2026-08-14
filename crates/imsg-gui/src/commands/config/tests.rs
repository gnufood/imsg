//! `figment::Jail` isolation, as in `crate::config`'s tests. These prove the shims delegate
//! and map errors through `CommandError`.

use serial_test::serial;

use super::*;

#[test]
#[serial]
fn config_show_returns_resolved_config() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        let dto = config_show(None).map_err(|e| figment::Error::from(e.to_string()))?;
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

        let dto = config_show(None).map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.device_address, "11:22:33:44:55:66");
        Ok(())
    });
}

#[test]
fn config_set_device_maps_invalid_mac_to_command_error() {
    let result = config_set_device("not-a-mac".to_owned());
    assert!(result.is_err());
}

#[test]
#[serial]
fn config_set_channels_persists() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        config_set_channels(14, 19).map_err(|e| figment::Error::from(e.to_string()))?;

        let dto = config_show(None).map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.map_channel, 14);
        assert_eq!(dto.pbap_channel, 19);
        Ok(())
    });
}

#[test]
fn config_set_channels_maps_out_of_bounds_to_command_error() {
    let result = config_set_channels(0, 19);
    assert!(result.is_err());
}

#[test]
fn config_set_channels_maps_equal_channels_to_command_error() {
    let result = config_set_channels(9, 9);
    assert!(result.is_err());
}

#[test]
#[serial]
fn config_is_device_configured_false_then_true() {
    figment::Jail::expect_with(|jail| {
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());
        jail.set_env("XDG_CONFIG_HOME", home.to_str().unwrap_or_default());

        assert!(!config_is_device_configured());
        config_set_device_and_channels("AA:BB:CC:DD:EE:FF".to_owned(), 5, 17)
            .map_err(|e| figment::Error::from(e.to_string()))?;
        assert!(config_is_device_configured());
        Ok(())
    });
}

#[test]
#[serial]
fn config_set_device_and_channels_persists() {
    figment::Jail::expect_with(|jail| {
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        config_set_device_and_channels("11:22:33:44:55:66".to_owned(), 4, 19)
            .map_err(|e| figment::Error::from(e.to_string()))?;

        let dto = config_show(None).map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.device_address, "11:22:33:44:55:66");
        assert_eq!(dto.map_channel, 4);
        assert_eq!(dto.pbap_channel, 19);
        Ok(())
    });
}

#[test]
fn config_set_device_and_channels_maps_invalid_mac_to_command_error() {
    let result = config_set_device_and_channels("not-a-mac".to_owned(), 2, 13);
    assert!(result.is_err());
}

#[test]
#[serial]
fn config_set_broker_security_level_persists() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        config_set_broker_security_level(crate::dto::SecurityLevelDto::High)
            .map_err(|e| figment::Error::from(e.to_string()))?;

        let dto = config_show(None).map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.security_level, Some(crate::dto::SecurityLevelDto::High));
        Ok(())
    });
}
