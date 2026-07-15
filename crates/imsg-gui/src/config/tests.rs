//! Real config-file I/O, isolated via `figment::Jail` (same approach `imsg-config`'s own
//! `set_hub_key_roundtrip` test uses) so these never touch this machine's real
//! `~/.config/imsg/imsg.toml`. `#[serial]` because `Jail` mutates process-global env vars.

use serial_test::serial;

use super::*;

#[test]
#[serial]
fn show_returns_resolved_config() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        let dto = show(None).map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.device_address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(dto.map_channel, 2);
        assert_eq!(dto.pbap_channel, 13);
        assert_eq!(dto.hub_node_key, None);
        Ok(())
    });
}

#[test]
#[serial]
fn set_device_persists_and_show_reflects_it() {
    figment::Jail::expect_with(|jail| {
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        set_device("11:22:33:44:55:66").map_err(|e| figment::Error::from(e.to_string()))?;

        let dto = show(None).map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.device_address, "11:22:33:44:55:66");
        Ok(())
    });
}

#[test]
fn set_device_rejects_invalid_mac() {
    let result = set_device("not-a-mac");
    assert!(matches!(result, Err(config::ConfigError::Invalid { field: "device.address", .. })));
}

#[test]
#[serial]
fn set_map_channel_persists_and_show_reflects_it() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        set_map_channel(11).map_err(|e| figment::Error::from(e.to_string()))?;

        let dto = show(None).map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.map_channel, 11);
        Ok(())
    });
}

#[test]
fn set_map_channel_rejects_out_of_bounds() {
    let result = set_map_channel(0);
    assert!(matches!(
        result,
        Err(config::ConfigError::Invalid { field: "device.map_channel", .. })
    ));
}

#[test]
#[serial]
fn set_pbap_channel_persists_and_show_reflects_it() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        set_pbap_channel(22).map_err(|e| figment::Error::from(e.to_string()))?;

        let dto = show(None).map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.pbap_channel, 22);
        Ok(())
    });
}

#[test]
fn set_pbap_channel_rejects_out_of_bounds() {
    let result = set_pbap_channel(31);
    assert!(matches!(
        result,
        Err(config::ConfigError::Invalid { field: "device.pbap_channel", .. })
    ));
}

#[test]
#[serial]
fn is_device_configured_false_then_true_after_set_device_and_channels() {
    figment::Jail::expect_with(|jail| {
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());
        jail.set_env("XDG_CONFIG_HOME", home.to_str().unwrap_or_default());

        assert!(!is_device_configured());
        set_device_and_channels("AA:BB:CC:DD:EE:FF", 5, 17)
            .map_err(|e| figment::Error::from(e.to_string()))?;
        assert!(is_device_configured());
        Ok(())
    });
}

#[test]
#[serial]
fn set_device_and_channels_persists_and_show_reflects_it() {
    figment::Jail::expect_with(|jail| {
        let home = jail.directory().to_path_buf();
        jail.set_env("HOME", home.to_str().unwrap_or_default());

        set_device_and_channels("11:22:33:44:55:66", 4, 19)
            .map_err(|e| figment::Error::from(e.to_string()))?;

        let dto = show(None).map_err(|e| figment::Error::from(e.to_string()))?;
        assert_eq!(dto.device_address, "11:22:33:44:55:66");
        assert_eq!(dto.map_channel, 4);
        assert_eq!(dto.pbap_channel, 19);
        Ok(())
    });
}

#[test]
fn set_device_and_channels_rejects_invalid_address() {
    let result = set_device_and_channels("not-an-address", 2, 13);
    assert!(matches!(result, Err(config::ConfigError::Invalid { field: "device.address", .. })));
}
