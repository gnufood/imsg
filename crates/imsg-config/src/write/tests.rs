use serial_test::serial;

use super::*;

#[test]
fn broker_abstract_name_produces_valid_name() -> Result<(), ConfigError> {
    broker_abstract_name("AA:BB:CC:DD:EE:FF")?;
    broker_abstract_name("00:00:00:00:00:00")?;
    Ok(())
}

#[test]
fn set_hub_key_rejects_empty() {
    let result = set_hub_key("");
    assert!(matches!(result, Err(ConfigError::Invalid { field: "hub.node_key", .. })));
}

#[test]
#[serial]
fn set_hub_key_roundtrip() {
    figment::Jail::expect_with(|jail| {
        let tmp = jail.directory().to_path_buf();
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        jail.set_env("HOME", tmp.to_str().unwrap_or_default());
        jail.set_env("XDG_CONFIG_HOME", tmp.to_str().unwrap_or_default());
        let key = "fakehubkey456";
        set_hub_key(key).map_err(|e| figment::Error::from(e.to_string()))?;
        // crate::figment(None) includes DEFAULTS + user file at HOME/.config/imsg/imsg.toml
        let cfg: crate::Config = crate::figment(None).extract()?;
        assert_eq!(cfg.hub.node_key.as_deref(), Some(key));
        Ok(())
    });
}

#[test]
#[serial]
fn set_broker_security_level_roundtrip() {
    figment::Jail::expect_with(|jail| {
        let tmp = jail.directory().to_path_buf();
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        jail.set_env("HOME", tmp.to_str().unwrap_or_default());
        jail.set_env("XDG_CONFIG_HOME", tmp.to_str().unwrap_or_default());
        set_broker_security_level(SecurityLevel::High)
            .map_err(|e| figment::Error::from(e.to_string()))?;
        let cfg: crate::Config = crate::figment(None).extract()?;
        assert_eq!(cfg.broker.security_level, Some(SecurityLevel::High));
        Ok(())
    });
}

#[test]
fn set_channels_rejects_out_of_bounds_map_channel() {
    let result = set_channels(0, 13);
    assert!(matches!(result, Err(ConfigError::Invalid { field: "device.map_channel", .. })));
}

#[test]
fn set_channels_rejects_out_of_bounds_pbap_channel() {
    let result = set_channels(2, 31);
    assert!(matches!(result, Err(ConfigError::Invalid { field: "device.pbap_channel", .. })));
}

#[test]
fn set_channels_rejects_equal_channels() {
    let result = set_channels(9, 9);
    assert!(matches!(result, Err(ConfigError::Invalid { field: "device.pbap_channel", .. })));
}

#[test]
#[serial]
fn set_channels_roundtrip() {
    figment::Jail::expect_with(|jail| {
        let tmp = jail.directory().to_path_buf();
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        jail.set_env("HOME", tmp.to_str().unwrap_or_default());
        jail.set_env("XDG_CONFIG_HOME", tmp.to_str().unwrap_or_default());
        set_channels(6, 22).map_err(|e| figment::Error::from(e.to_string()))?;
        // Typed extraction into `u8` fails if these were written as quoted TOML strings
        // instead of bare integers, so this also proves `patch_config`'s generalization.
        let cfg: crate::Config = crate::figment(None).extract()?;
        assert_eq!(cfg.device.map_channel, 6_u8);
        assert_eq!(cfg.device.pbap_channel, 22_u8);
        Ok(())
    });
}

#[test]
#[serial]
fn set_channels_rejects_equal_channels_without_writing() {
    figment::Jail::expect_with(|jail| {
        let tmp = jail.directory().to_path_buf();
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        jail.set_env("HOME", tmp.to_str().unwrap_or_default());
        jail.set_env("XDG_CONFIG_HOME", tmp.to_str().unwrap_or_default());
        // Seed prior values so a would-be partial write would be observable.
        set_channels(6, 22).map_err(|e| figment::Error::from(e.to_string()))?;
        let result = set_channels(9, 9);
        assert!(matches!(result, Err(ConfigError::Invalid { field: "device.pbap_channel", .. })));
        let cfg: crate::Config = crate::figment(None).extract()?;
        assert_eq!(cfg.device.map_channel, 6_u8, "rejected pair must not overwrite map_channel");
        assert_eq!(cfg.device.pbap_channel, 22_u8, "rejected pair must not overwrite pbap_channel");
        Ok(())
    });
}

#[test]
fn set_device_and_channels_rejects_invalid_address() {
    let result = set_device_and_channels("not-an-address", 2, 13);
    assert!(matches!(result, Err(ConfigError::Invalid { field: "device.address", .. })));
}

#[test]
fn set_device_and_channels_rejects_out_of_bounds_map_channel() {
    let result = set_device_and_channels("AA:BB:CC:DD:EE:FF", 0, 13);
    assert!(matches!(result, Err(ConfigError::Invalid { field: "device.map_channel", .. })));
}

#[test]
fn set_device_and_channels_rejects_out_of_bounds_pbap_channel() {
    let result = set_device_and_channels("AA:BB:CC:DD:EE:FF", 2, 31);
    assert!(matches!(result, Err(ConfigError::Invalid { field: "device.pbap_channel", .. })));
}

#[test]
#[serial]
fn set_device_and_channels_roundtrip() {
    figment::Jail::expect_with(|jail| {
        let tmp = jail.directory().to_path_buf();
        jail.set_env("HOME", tmp.to_str().unwrap_or_default());
        jail.set_env("XDG_CONFIG_HOME", tmp.to_str().unwrap_or_default());
        set_device_and_channels("AA:BB:CC:DD:EE:FF", 5, 17)
            .map_err(|e| figment::Error::from(e.to_string()))?;
        let cfg: crate::Config = crate::figment(None).extract()?;
        assert_eq!(cfg.device.address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(cfg.device.map_channel, 5_u8);
        assert_eq!(cfg.device.pbap_channel, 17_u8);
        Ok(())
    });
}

#[test]
#[serial]
fn patch_config_writes_multiple_keys_in_one_call() {
    figment::Jail::expect_with(|jail| {
        let tmp = jail.directory().to_path_buf();
        jail.set_env("HOME", tmp.to_str().unwrap_or_default());
        jail.set_env("XDG_CONFIG_HOME", tmp.to_str().unwrap_or_default());
        patch_config(
            "device",
            &[
                ("address", "AA:BB:CC:DD:EE:FF".into()),
                ("map_channel", i64::from(7_u8).into()),
                ("pbap_channel", i64::from(19_u8).into()),
            ],
        )
        .map_err(|e| figment::Error::from(e.to_string()))?;
        // All three land from a single call -- `set_device_and_channels` relies on this to
        // avoid three separate read-modify-write cycles of the same file.
        let cfg: crate::Config = crate::figment(None).extract()?;
        assert_eq!(cfg.device.address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(cfg.device.map_channel, 7_u8);
        assert_eq!(cfg.device.pbap_channel, 19_u8);
        Ok(())
    });
}

#[test]
#[serial]
fn set_device_and_channels_rejects_invalid_pbap_without_writing_address() {
    figment::Jail::expect_with(|jail| {
        let tmp = jail.directory().to_path_buf();
        jail.set_env("HOME", tmp.to_str().unwrap_or_default());
        jail.set_env("XDG_CONFIG_HOME", tmp.to_str().unwrap_or_default());
        let result = set_device_and_channels("AA:BB:CC:DD:EE:FF", 5, 31);
        assert!(matches!(result, Err(ConfigError::Invalid { field: "device.pbap_channel", .. })));
        // `device.address` has no default, so extraction only fails here if the earlier
        // `pbap_channel` rejection genuinely happened before any write -- proving the
        // upfront validation, not partial persistence, is what keeps this atomic.
        let extracted: Result<crate::Config, figment::Error> = crate::figment(None).extract();
        assert!(extracted.is_err(), "address must not have been persisted");
        Ok(())
    });
}
