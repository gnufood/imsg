//! `watch --hub`: subscribe to a remote hub's MNS event stream over iroh and print events.

use anyhow::{Context, Result};
use bytes::Bytes;
use config::Config;
use tokio::io::{AsyncRead, AsyncReadExt};
use transport::iroh::Endpoint;

use crate::commands::conn;
use crate::output;

/// Formats an MNS event for stdout: `<EventType>  handle=<h>  folder=<f>`.
fn format_event(ev: &map_core::mns_event::MnsEvent) -> String {
    format!(
        "{:?}  handle={}  folder={}",
        ev.event_type(),
        ev.handle().unwrap_or("-"),
        ev.folder().unwrap_or("-"),
    )
}

/// Upper bound on a single MNS event frame from the hub. Real MAP event reports are well under
/// 1 KiB; this cap bounds memory against a malformed or hostile length prefix.
const MAX_MNS_FRAME: u32 = 1_048_576;

/// Subscribes to a remote hub's MNS event stream over iroh and prints each event until Ctrl+C
/// or the hub closes the stream.
///
/// Uses `endpoint` to open the MNS subscription and reads length-framed event-report payloads.
/// A payload that fails to parse is logged and skipped — one malformed event does not end the
/// watch. Does not reconnect on stream drop — exit and re-run to resubscribe.
///
/// # Errors
///
/// Returns an error if `hub.node_key` is unset or invalid, the hub connection fails, or a
/// frame's declared length exceeds [`MAX_MNS_FRAME`].
pub(crate) async fn run(cfg: &Config, endpoint: &Endpoint) -> Result<()> {
    let hub_id = conn::resolve_hub_id(cfg.hub.node_key.as_deref())?;
    let mut reader = transport::iroh::connect_mns_hub(endpoint, hub_id)
        .await
        .context("connecting to hub MNS stream")?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            frame = read_mns_frame(&mut reader, MAX_MNS_FRAME) => match frame? {
                Some(body) => match map_core::mns_event::parse_event_report(&body) {
                    Ok(event) => output::line(&format_event(&event))?,
                    Err(e) => tracing::warn!("skipping malformed MNS event: {e}"),
                },
                None => break,
            }
        }
    }
    Ok(())
}

/// Reads one length-prefixed MNS event frame from `reader`.
///
/// Wire format: a 4-byte big-endian length followed by exactly that many payload bytes. Returns
/// `Ok(None)` on a clean EOF before any length bytes arrive (the hub closed the stream). Does
/// not parse the payload.
///
/// # Errors
///
/// Returns an error if the declared length exceeds `max_bytes`, or if the stream ends partway
/// through the payload.
async fn read_mns_frame(
    reader: &mut (impl AsyncRead + Unpin),
    max_bytes: u32,
) -> Result<Option<Bytes>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(anyhow::Error::new(e).context("reading MNS frame length")),
    }
    let len = u32::from_be_bytes(len_buf);
    if len > max_bytes {
        anyhow::bail!("MNS frame too large: {len} bytes (max {max_bytes})");
    }
    let n = usize::try_from(len).context("frame length overflows usize")?;
    let mut payload = vec![0u8; n];
    reader.read_exact(&mut payload).await.context("reading MNS frame payload")?;
    Ok(Some(Bytes::from(payload)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_frame_happy_path() -> Result<()> {
        let payload = b"<event/>";
        let mut framed = Vec::new();
        framed.extend_from_slice(&u32::try_from(payload.len())?.to_be_bytes());
        framed.extend_from_slice(payload);
        let mut reader = &framed[..];
        let frame = read_mns_frame(&mut reader, MAX_MNS_FRAME).await?;
        assert_eq!(frame.as_deref(), Some(&payload[..]));
        Ok(())
    }

    #[tokio::test]
    async fn read_frame_clean_eof() -> Result<()> {
        let mut reader = &b""[..];
        let frame = read_mns_frame(&mut reader, MAX_MNS_FRAME).await?;
        assert_eq!(frame, None);
        Ok(())
    }

    #[tokio::test]
    async fn read_frame_too_large() {
        let framed = (MAX_MNS_FRAME + 1).to_be_bytes();
        let mut reader = &framed[..];
        let result = read_mns_frame(&mut reader, MAX_MNS_FRAME).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn read_frame_eof_mid_payload() {
        let mut framed = Vec::new();
        framed.extend_from_slice(&5u32.to_be_bytes());
        framed.extend_from_slice(b"ab");
        let mut reader = &framed[..];
        let result = read_mns_frame(&mut reader, MAX_MNS_FRAME).await;
        assert!(result.is_err());
    }
}
