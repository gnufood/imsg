//! `spoke` subcommand: configure the iroh hub key for spoke mode.

use anyhow::{Context, Result};
use transport::iroh::EndpointId;

/// Validates `key_str` as an iroh [`EndpointId`] and persists it to the user config.
///
/// Validation is syntactic only — semantic validity (hub reachability, correct pairing) is
/// deferred to connect time. Returns a confirmation line on success.
///
/// # Errors
///
/// Returns an error if `key_str` cannot be parsed as an [`EndpointId`] or the config file
/// cannot be written.
pub(crate) async fn run_add(key_str: &str) -> Result<String> {
    key_str.parse::<EndpointId>().with_context(|| format!("invalid hub key: {key_str}"))?;
    let owned = key_str.to_owned();
    tokio::task::spawn_blocking(move || config::set_hub_key(&owned))
        .await
        .context("set_hub_key task panicked")?
        .context("writing hub key to user config")?;
    Ok(format!("hub.node_key = {key_str}"))
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[tokio::test]
    async fn spoke_add_invalid_key() {
        let result = run_add("this-is-not-a-valid-iroh-key").await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn spoke_add_valid_key() {
        let key = transport::iroh::SecretKey::generate();
        let id_str = key.public().to_string();
        let tmp = std::env::temp_dir().join(format!("imsg_spoke_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).ok();
        let saved_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", &tmp);
        let result = run_add(&id_str).await;
        match saved_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
        std::fs::remove_dir_all(&tmp).ok();
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }
}
