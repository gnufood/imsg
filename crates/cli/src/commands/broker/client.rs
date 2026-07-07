//! IPC transport: connect to the broker abstract socket, send/receive length-delimited frames.

use anyhow::{Context, Result};
use bytes::Bytes;
use config::Config;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::{tokio::Stream as LocalStream, ConnectOptions};
use ipc::{BrokerRequest, BrokerResponse, MAX_FRAME_LEN};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

/// Sends `req` over a fresh connection and returns one response frame.
///
/// Does not auto-start the broker — callers must call `ensure_running` first.
///
/// # Errors
///
/// Returns an error if the connection fails or frame encoding/decoding fails.
pub(super) async fn send_request(addr: &str, req: BrokerRequest) -> Result<BrokerResponse> {
    let mut framed = connect_raw(addr).await?;
    send_frame(&mut framed, &req).await?;
    recv_frame(&mut framed).await
}

/// Returns a live `Framed` connection to the broker's abstract socket.
///
/// Does not auto-start the broker — callers must call `ensure_running` first.
///
/// # Errors
///
/// Returns an error if the abstract socket connect fails.
async fn connect_raw(addr: &str) -> Result<Framed<LocalStream, LengthDelimitedCodec>> {
    let name = config::broker_abstract_name(addr).context("building broker socket name")?;
    let stream =
        ConnectOptions::new().name(name).connect_tokio().await.context("connecting to broker")?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    Ok(Framed::new(stream, codec))
}

/// Returns a one-line health summary, or `"not running"` if the broker is unreachable.
///
/// `label` prefixes the summary (e.g. `"broker"` or `"daemon"`) to match the command the
/// caller invoked — the underlying probe is identical regardless of which spawned the process.
///
/// Does not start the broker — only probes whether it is already listening.
///
/// # Errors
///
/// Returns an error if a running broker returns a malformed response.
pub(in crate::commands) async fn run_status(
    cfg: &Config,
    device: Option<&str>,
    label: &str,
) -> Result<String> {
    let addr = device.unwrap_or_else(|| cfg.device.address());
    let Ok(mut framed) = connect_raw(addr).await else {
        return Ok(format!("{label} for {addr}: not running"));
    };
    send_frame(&mut framed, &BrokerRequest::Status).await?;
    match recv_frame(&mut framed).await? {
        BrokerResponse::StatusInfo { state, device: dev } => {
            Ok(format!("{label} for {dev}: {state}"))
        }
        other => Ok(format!("unexpected response: {other:?}")),
    }
}

/// Sends a graceful `Shutdown` request, or reports `"not running"` if unreachable.
///
/// Idempotent by design — stopping something that isn't running is a no-op, not an error, and
/// this never auto-starts the broker just to shut it back down.
///
/// # Errors
///
/// Returns an error if the connection succeeds but sending or receiving the frame fails.
pub(in crate::commands) async fn run_stop(cfg: &Config, device: Option<&str>) -> Result<String> {
    let addr = device.unwrap_or_else(|| cfg.device.address());
    let Ok(mut framed) = connect_raw(addr).await else {
        return Ok(format!("daemon for {addr}: not running"));
    };
    send_frame(&mut framed, &BrokerRequest::Shutdown).await?;
    match recv_frame(&mut framed).await? {
        BrokerResponse::Ok => Ok(format!("daemon for {addr}: stopping")),
        BrokerResponse::Error(e) => Ok(format!("daemon for {addr}: {e}")),
        other => Ok(format!("unexpected response: {other:?}")),
    }
}

/// Encodes `req` as JSON and writes one length-delimited frame.
///
/// # Errors
///
/// Returns an error if serialisation or the socket write fails.
async fn send_frame<T: tokio::io::AsyncWrite + Unpin>(
    framed: &mut Framed<T, LengthDelimitedCodec>,
    req: &BrokerRequest,
) -> Result<()> {
    let bytes = Bytes::from(serde_json::to_vec(req).context("serialising request")?);
    framed.send(bytes).await.context("sending request frame")
}

/// Reads one response frame and deserialises it.
///
/// # Errors
///
/// Returns an error if the connection closes unexpectedly or deserialisation fails.
async fn recv_frame<T: tokio::io::AsyncRead + Unpin>(
    framed: &mut Framed<T, LengthDelimitedCodec>,
) -> Result<BrokerResponse> {
    let frame = framed
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("broker closed connection without sending a response"))?
        .context("reading response frame")?;
    serde_json::from_slice(&frame).context("deserialising response")
}
