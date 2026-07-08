//! Frame-level transport: connect to the broker socket, send/receive length-delimited frames.

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::{tokio::Stream as LocalStream, ConnectOptions};
use ipc::{BrokerRequest, BrokerResponse, MAX_FRAME_LEN};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

/// Sends `req` over a fresh connection and returns one response frame.
///
/// Does not auto-start the broker — callers must ensure it's already running.
///
/// # Errors
///
/// Returns an error if the connection fails or frame encoding/decoding fails.
pub async fn send_request(addr: &str, req: BrokerRequest) -> Result<BrokerResponse> {
    let mut framed = connect_raw(addr).await?;
    send_frame(&mut framed, &req).await?;
    recv_frame(&mut framed).await
}

/// Returns a live `Framed` connection to the broker's abstract socket.
///
/// # Errors
///
/// Returns an error if the abstract socket connect fails.
pub(crate) async fn connect_raw(addr: &str) -> Result<Framed<LocalStream, LengthDelimitedCodec>> {
    let name = config::broker_abstract_name(addr).context("building broker socket name")?;
    let stream =
        ConnectOptions::new().name(name).connect_tokio().await.context("connecting to broker")?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    Ok(Framed::new(stream, codec))
}

/// Encodes `req` as JSON and writes one length-delimited frame.
///
/// # Errors
///
/// Returns an error if serialisation or the socket write fails.
pub(crate) async fn send_frame<T: tokio::io::AsyncWrite + Unpin>(
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
pub(crate) async fn recv_frame<T: tokio::io::AsyncRead + Unpin>(
    framed: &mut Framed<T, LengthDelimitedCodec>,
) -> Result<BrokerResponse> {
    let frame = framed
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("broker closed connection without sending a response"))?
        .context("reading response frame")?;
    serde_json::from_slice(&frame).context("deserialising response")
}

#[cfg(test)]
mod tests;
