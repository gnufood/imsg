use serial_test::serial;

use super::*;

fn make_device(address: &str) -> DeviceConfig {
    DeviceConfig { address: address.to_owned(), map_channel: 2, pbap_channel: 13 }
}

#[test]
fn device_address_required() {
    // Only compiled-in defaults — no device.address present.
    let result = Figment::from(Toml::string(DEFAULTS)).extract::<Config>();
    assert!(result.is_err());
}

#[test]
fn validate_rejects_bad_address() {
    for bad in ["NOTAMAC", "GG:GG:GG:GG:GG:GG", "00:11:22:33:44", "00:11:22:33:44:55:66"] {
        let cfg = Config {
            device: make_device(bad),
            hub: HubConfig::default(),
            store: StoreConfig::default(),
            broker: BrokerConfig::default(),
        };
        assert!(
            matches!(validate(&cfg), Err(ConfigError::Invalid { field: "device.address", .. })),
            "expected Invalid for {bad:?}"
        );
    }
}

#[test]
fn validate_rejects_channel_bounds() {
    for bad_map in [0_u8, 31_u8] {
        let cfg = Config {
            device: DeviceConfig {
                address: "AA:BB:CC:DD:EE:FF".to_owned(),
                map_channel: bad_map,
                pbap_channel: 13,
            },
            hub: HubConfig::default(),
            store: StoreConfig::default(),
            broker: BrokerConfig::default(),
        };
        assert!(
            matches!(validate(&cfg), Err(ConfigError::Invalid { field: "device.map_channel", .. })),
            "expected Invalid for map_channel={bad_map}"
        );
    }
    let cfg = Config {
        device: DeviceConfig {
            address: "AA:BB:CC:DD:EE:FF".to_owned(),
            map_channel: 2,
            pbap_channel: 0,
        },
        hub: HubConfig::default(),
        store: StoreConfig::default(),
        broker: BrokerConfig::default(),
    };
    assert!(matches!(
        validate(&cfg),
        Err(ConfigError::Invalid { field: "device.pbap_channel", .. })
    ));
}

#[test]
fn validate_rejects_equal_map_and_pbap_channel() {
    let cfg = Config {
        device: DeviceConfig {
            map_channel: 9,
            pbap_channel: 9,
            ..make_device("AA:BB:CC:DD:EE:FF")
        },
        hub: HubConfig::default(),
        store: StoreConfig::default(),
        broker: BrokerConfig::default(),
    };
    assert!(matches!(
        validate(&cfg),
        Err(ConfigError::Invalid { field: "device.pbap_channel", .. })
    ));
}

#[test]
#[serial]
fn env_override_address() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        jail.set_env("IMSG_DEVICE__MAP_CHANNEL", "5");
        let cfg: Config = figment(None).extract()?;
        assert_eq!(cfg.device.address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(cfg.device.map_channel, 5_u8);
        Ok(())
    });
}

#[test]
#[serial]
fn hub_defaults_to_none() {
    figment::Jail::expect_with(|jail| {
        let dir = jail.directory().to_string_lossy().into_owned();
        jail.set_env("HOME", &dir);
        jail.set_env("XDG_CONFIG_HOME", &dir);
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let cfg: Config = figment(None).extract()?;
        assert!(cfg.hub.node_key.is_none());
        Ok(())
    });
}

#[test]
#[serial]
fn env_override_hub_node_key() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        jail.set_env("IMSG_HUB__NODE_KEY", "testkey123");
        let cfg: Config = figment(None).extract()?;
        assert_eq!(cfg.hub.node_key.as_deref(), Some("testkey123"));
        Ok(())
    });
}

#[test]
fn hub_key_path_format() {
    if let Some(p) = hub_key_path() {
        assert!(p.ends_with("imsg/hub.key"), "unexpected path: {p:?}");
    }
    // None is valid in container environments — silently skip.
}

#[test]
fn hub_lock_path_format() {
    if let Some(p) = hub_lock_path() {
        assert!(p.ends_with("imsg/hub.lock"), "unexpected path: {p:?}");
    }
    // None is valid in container environments — silently skip.
}

#[test]
#[serial]
fn broker_security_level_defaults_to_none() {
    figment::Jail::expect_with(|jail| {
        let dir = jail.directory().to_string_lossy().into_owned();
        jail.set_env("HOME", &dir);
        jail.set_env("XDG_CONFIG_HOME", &dir);
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let cfg: Config = figment(None).extract()?;
        assert_eq!(cfg.broker.security_level, None);
        Ok(())
    });
}

#[test]
#[serial]
fn env_override_broker_security_level() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        jail.set_env("IMSG_BROKER__SECURITY_LEVEL", "medium");
        let cfg: Config = figment(None).extract()?;
        assert_eq!(cfg.broker.security_level, Some(SecurityLevel::Medium));
        Ok(())
    });
}

#[test]
#[serial]
fn store_path_env_override() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        jail.set_env("IMSG_STORE__PATH", "/tmp/test.db");
        let cfg: Config = figment(None).extract()?;
        assert_eq!(cfg.store.path.as_deref(), Some(std::path::Path::new("/tmp/test.db")));
        assert_eq!(cfg.store.resolve().as_deref(), Some(std::path::Path::new("/tmp/test.db")));
        Ok(())
    });
}

#[test]
#[serial]
fn store_defaults_to_db_path() {
    figment::Jail::expect_with(|jail| {
        let dir = jail.directory().to_string_lossy().into_owned();
        jail.set_env("HOME", &dir);
        jail.set_env("XDG_CONFIG_HOME", &dir);
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        let cfg: Config = figment(None).extract()?;
        assert!(cfg.store.path.is_none());
        // resolve() falls back to db_path() — same value.
        assert_eq!(cfg.store.resolve(), db_path());
        Ok(())
    });
}

#[test]
#[serial]
fn is_device_configured_true_when_address_set() {
    figment::Jail::expect_with(|jail| {
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        assert!(is_device_configured(None));
        Ok(())
    });
}

#[test]
#[serial]
fn is_device_configured_false_when_address_unset() {
    figment::Jail::expect_with(|jail| {
        let dir = jail.directory().to_string_lossy().into_owned();
        jail.set_env("HOME", &dir);
        jail.set_env("XDG_CONFIG_HOME", &dir);
        assert!(!is_device_configured(None));
        Ok(())
    });
}

#[test]
#[serial]
fn is_device_configured_true_after_set_device_and_channels() {
    figment::Jail::expect_with(|jail| {
        let dir = jail.directory().to_string_lossy().into_owned();
        jail.set_env("HOME", &dir);
        jail.set_env("XDG_CONFIG_HOME", &dir);
        assert!(!is_device_configured(None));
        set_device_and_channels("AA:BB:CC:DD:EE:FF", 2, 13)
            .map_err(|e| figment::Error::from(e.to_string()))?;
        assert!(is_device_configured(None));
        Ok(())
    });
}
