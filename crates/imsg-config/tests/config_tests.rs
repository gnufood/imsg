//! Integration tests for `config::set_device` and `config::set_hub_key`.

use imsg_config::{load, set_device, set_hub_key, ConfigError};
use serial_test::serial;

#[test]
#[serial]
fn set_device_writes_address() {
    figment::Jail::expect_with(|jail| {
        let xdg = jail.directory().to_path_buf();
        jail.set_env("XDG_CONFIG_HOME", xdg.to_string_lossy().as_ref());
        set_device("AA:BB:CC:DD:EE:FF").map_err(|e| e.to_string())?;
        let cfg = load(None).map_err(|e| e.to_string())?;
        assert_eq!(cfg.device.address(), "AA:BB:CC:DD:EE:FF");
        Ok(())
    });
}

#[test]
fn set_device_rejects_invalid_mac() {
    let result = set_device("NOT_A_MAC");
    assert!(matches!(result, Err(ConfigError::Invalid { field: "device.address", .. })));
}

#[test]
#[serial]
fn set_device_creates_parent_dir() {
    figment::Jail::expect_with(|jail| {
        let xdg = jail.directory().to_path_buf();
        jail.set_env("XDG_CONFIG_HOME", xdg.to_string_lossy().as_ref());
        set_device("AA:BB:CC:DD:EE:FF").map_err(|e| e.to_string())?;
        let expected = xdg.join("imsg/imsg.toml");
        assert!(expected.exists(), "expected config file at {expected:?}");
        Ok(())
    });
}

#[test]
#[serial]
fn set_device_creates_device_section_when_absent() {
    figment::Jail::expect_with(|jail| {
        let xdg = jail.directory().to_path_buf();
        jail.set_env("XDG_CONFIG_HOME", xdg.to_string_lossy().as_ref());
        let dir = xdg.join("imsg");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        // Seed file with a hub key; set_device must not clobber it.
        std::fs::write(dir.join("imsg.toml"), "[hub]\nnode_key = \"preservedkey\"\n")
            .map_err(|e| e.to_string())?;
        set_device("AA:BB:CC:DD:EE:FF").map_err(|e| e.to_string())?;
        let cfg = load(None).map_err(|e| e.to_string())?;
        assert_eq!(cfg.device.address(), "AA:BB:CC:DD:EE:FF");
        assert_eq!(cfg.hub.node_key.as_deref(), Some("preservedkey"));
        Ok(())
    });
}

#[test]
#[serial]
fn set_device_preserves_existing_keys() {
    figment::Jail::expect_with(|jail| {
        let xdg = jail.directory().to_path_buf();
        jail.set_env("XDG_CONFIG_HOME", xdg.to_string_lossy().as_ref());
        let dir = xdg.join("imsg");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::write(
            dir.join("imsg.toml"),
            "[device]\naddress = \"11:22:33:44:55:66\"\n\n[hub]\nnode_key = \"preservedkey\"\n",
        )
        .map_err(|e| e.to_string())?;
        set_device("AA:BB:CC:DD:EE:FF").map_err(|e| e.to_string())?;
        let cfg = load(None).map_err(|e| e.to_string())?;
        assert_eq!(cfg.device.address(), "AA:BB:CC:DD:EE:FF");
        assert_eq!(cfg.hub.node_key.as_deref(), Some("preservedkey"));
        Ok(())
    });
}

#[test]
#[serial]
fn set_hub_key_writes_and_load_roundtrip() {
    figment::Jail::expect_with(|jail| {
        let xdg = jail.directory().to_path_buf();
        jail.set_env("XDG_CONFIG_HOME", xdg.to_string_lossy().as_ref());
        jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
        set_hub_key("myhubkey123").map_err(|e| e.to_string())?;
        let cfg = load(None).map_err(|e| e.to_string())?;
        assert_eq!(cfg.hub.node_key.as_deref(), Some("myhubkey123"));
        Ok(())
    });
}
