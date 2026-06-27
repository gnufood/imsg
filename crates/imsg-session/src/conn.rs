//! Connection assembly: selects the transport target and opens a MAP or PBAP session.

use std::future::Future;
use std::pin::Pin;

use anyhow::{Context, Result};
use config::Config;
use map_core::client::MapClient;
use pbap_core::client::PbapClient;
use tokio_util::either::Either;
use transport::iroh::{Endpoint, EndpointId, HubStream, MAP_ALPN, PBAP_ALPN};

/// MAP/PBAP session stream: iroh hub (Left) or RFCOMM (Right).
pub type Stream = Either<HubStream, bluer::rfcomm::Stream>;

/// Resolved transport target for one MAP or PBAP invocation.
enum Target {
    /// iroh hub mode — connects via QUIC to the given hub node.
    Hub(EndpointId),
    /// RFCOMM device address and channel.
    Rfcomm(bluer::Address, u8),
}

/// Selects the transport target from CLI flags and config primitives.
///
/// In hub mode `hub_node_key` must be `Some` and parseable as an [`EndpointId`] — returns an
/// error otherwise. In RFCOMM mode, resolves device address from `device_override` then
/// `device_addr`. Does not validate that `channel` is within the RFCOMM range `[1, 30]` —
/// validated at [`config::load`] time.
///
/// # Errors
///
/// Returns an error if `hub` is `true` and `hub_node_key` is absent or not a valid
/// [`EndpointId`], or if the RFCOMM device address cannot be parsed as `XX:XX:XX:XX:XX:XX`.
fn target(
    hub: bool,
    hub_node_key: Option<&str>,
    device_override: Option<&str>,
    device_addr: &str,
    channel: u8,
) -> Result<Target> {
    if hub {
        Ok(Target::Hub(resolve_hub_id(hub_node_key)?))
    } else {
        let addr_str = device_override.unwrap_or(device_addr);
        let addr = addr_str
            .parse::<bluer::Address>()
            .with_context(|| format!("invalid device address: {addr_str}"))?;
        Ok(Target::Rfcomm(addr, channel))
    }
}

/// Parses the configured hub node key into an [`EndpointId`].
///
/// Does not validate hub reachability or pairing — those surface at connect time.
///
/// # Errors
///
/// Returns an error if `node_key` is `None` (hub not configured) or is not a valid iroh
/// public key.
pub fn resolve_hub_id(node_key: Option<&str>) -> Result<EndpointId> {
    let key_str = node_key
        .ok_or_else(|| anyhow::anyhow!("hub.node_key is not set; run `imsg spoke add <KEY>`"))?;
    key_str.parse::<EndpointId>().context("invalid hub.node_key")
}

/// Opens a MAP session, connecting via iroh hub or RFCOMM depending on `endpoint`.
///
/// Hub path: connects to the configured hub over [`MAP_ALPN`] using the caller-owned
/// `endpoint` and completes OBEX CONNECT with event notifications enabled. RFCOMM path:
/// identical behaviour to before hub/spoke was introduced. Caller must hold the returned
/// client alive — iOS drops the notification registration on OBEX DISCONNECT.
///
/// # Errors
///
/// Returns an error if target resolution fails, transport connection fails, or OBEX session
/// establishment fails.
#[must_use]
pub fn connect_map<'a>(
    cfg: &'a Config,
    endpoint: Option<&'a Endpoint>,
    device_override: Option<&'a str>,
) -> Pin<Box<dyn Future<Output = Result<MapClient<Stream>>> + Send + 'a>> {
    // Box the iroh handshake state onto the heap so caller futures stay small (clippy::large_futures).
    Box::pin(connect_map_inner(cfg, endpoint, device_override))
}

async fn connect_map_inner(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device_override: Option<&str>,
) -> Result<MapClient<Stream>> {
    let tgt = target(
        endpoint.is_some(),
        cfg.hub.node_key.as_deref(),
        device_override,
        cfg.device.address(),
        cfg.device.map_channel,
    )?;
    match tgt {
        Target::Hub(id) => {
            let ep = endpoint
                .ok_or_else(|| anyhow::anyhow!("internal: hub target requires an endpoint"))?;
            let conn = ep.connect(id, MAP_ALPN).await.context("iroh connect (MAP)")?;
            let (send, recv) = conn.open_bi().await.context("iroh open_bi (MAP)")?;
            let stream: Stream = Either::Left(HubStream::new(tokio::io::join(recv, send), conn));
            crate::lifecycle::establish_map_session(stream)
                .await
                .context("establishing MAP session over iroh")
        }
        Target::Rfcomm(addr, channel) => {
            let stream = transport::rfcomm::connect(
                addr,
                channel,
                transport::rfcomm::DEFAULT_BT_CONNECTED_GATE,
            )
            .await
            .context("RFCOMM connect (MAP)")?;
            crate::lifecycle::establish_map_session(Either::Right(stream))
                .await
                .context("establishing MAP session")
        }
    }
}

/// Opens a PBAP session, connecting via iroh hub or RFCOMM depending on `endpoint`.
///
/// Hub path: connects over [`PBAP_ALPN`] using the caller-owned `endpoint`. RFCOMM path:
/// connects directly to the paired Bluetooth device.
///
/// # Errors
///
/// Returns an error if target resolution fails, transport connection fails, or OBEX session
/// establishment fails.
#[must_use]
pub fn connect_pbap<'a>(
    cfg: &'a Config,
    endpoint: Option<&'a Endpoint>,
    device_override: Option<&'a str>,
) -> Pin<Box<dyn Future<Output = Result<PbapClient<Stream>>> + Send + 'a>> {
    // Box the iroh handshake state onto the heap so caller futures stay small (clippy::large_futures).
    Box::pin(connect_pbap_inner(cfg, endpoint, device_override))
}

async fn connect_pbap_inner(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device_override: Option<&str>,
) -> Result<PbapClient<Stream>> {
    let tgt = target(
        endpoint.is_some(),
        cfg.hub.node_key.as_deref(),
        device_override,
        cfg.device.address(),
        cfg.device.pbap_channel,
    )?;
    match tgt {
        Target::Hub(id) => {
            let ep = endpoint
                .ok_or_else(|| anyhow::anyhow!("internal: hub target requires an endpoint"))?;
            let conn = ep.connect(id, PBAP_ALPN).await.context("iroh connect (PBAP)")?;
            let (send, recv) = conn.open_bi().await.context("iroh open_bi (PBAP)")?;
            let stream: Stream = Either::Left(HubStream::new(tokio::io::join(recv, send), conn));
            PbapClient::connect(stream).await.context("establishing PBAP session over iroh")
        }
        Target::Rfcomm(addr, channel) => {
            let stream = transport::rfcomm::connect(
                addr,
                channel,
                transport::rfcomm::DEFAULT_BT_CONNECTED_GATE,
            )
            .await
            .context("RFCOMM connect (PBAP)")?;
            PbapClient::connect(Either::Right(stream)).await.context("establishing PBAP session")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_hub_key_absent_returns_err() {
        let result = target(true, None, None, "AA:BB:CC:DD:EE:FF", 2);
        assert!(result.is_err());
    }

    #[test]
    fn target_hub_key_invalid_returns_err() {
        let result = target(true, Some("not-a-valid-key"), None, "AA:BB:CC:DD:EE:FF", 2);
        assert!(result.is_err());
    }

    #[test]
    fn target_hub_valid_key() -> Result<()> {
        let key = transport::iroh::SecretKey::generate();
        let id_str = key.public().to_string();
        let tgt = target(true, Some(&id_str), None, "AA:BB:CC:DD:EE:FF", 2)?;
        assert!(matches!(tgt, Target::Hub(_)));
        Ok(())
    }

    #[test]
    fn target_rfcomm_valid_addr() -> Result<()> {
        let tgt = target(false, None, None, "AA:BB:CC:DD:EE:FF", 2)?;
        assert!(matches!(tgt, Target::Rfcomm(_, 2)));
        Ok(())
    }

    #[test]
    fn target_rfcomm_device_override() -> Result<()> {
        let tgt = target(false, None, Some("11:22:33:44:55:66"), "AA:BB:CC:DD:EE:FF", 13)?;
        let Target::Rfcomm(addr, ch) = tgt else {
            anyhow::bail!("expected Rfcomm");
        };
        assert_eq!(addr.to_string(), "11:22:33:44:55:66");
        assert_eq!(ch, 13);
        Ok(())
    }

    #[test]
    fn target_rfcomm_invalid_addr_returns_err() {
        let result = target(false, None, None, "not-a-mac", 2);
        assert!(result.is_err());
    }
}
