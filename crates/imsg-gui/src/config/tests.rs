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
