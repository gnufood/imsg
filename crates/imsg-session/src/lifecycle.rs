//! Connection lifecycle: RFCOMM connect, OBEX session establishment, and teardown, given an
//! already-resolved device address and channel (SDP lookup happens in `imsg-transport::discover`).

use std::time::Duration;

use map_core::client::MapClient;
use pbap_core::client::PbapClient;
use tokio::io::{AsyncRead, AsyncWrite};
use transport::rfcomm::DEFAULT_BT_CONNECTED_GATE;

use crate::SessionError;

/// OBEX CONNECT + enable MAP notifications. Caller must hold the returned client — iOS drops
/// notification registration on OBEX DISCONNECT.
///
/// # Errors
///
/// Returns [`SessionError::Map`] on OBEX protocol error or server refusal.
pub async fn establish_map_session<T: AsyncRead + AsyncWrite + Unpin>(
    stream: T,
) -> Result<MapClient<T>, SessionError> {
    let mut client = MapClient::connect(stream).await.inspect_err(|e| {
        tracing::warn!("MAP session: OBEX CONNECT failed: {e}");
    })?;
    tracing::debug!("MAP session: OBEX CONNECT ok, registering notifications");
    client.set_notification_registration(true).await.inspect_err(|e| {
        tracing::warn!("MAP session: notification registration failed: {e}");
    })?;
    tracing::debug!("MAP session: notification registration ok");
    Ok(client)
}

/// RFCOMM connect to `addr`:`channel` (gating on `BT_CONNECTED` up to `bt_gate`, requesting
/// `security` if given) then [`establish_map_session`].
///
/// # Errors
///
/// Returns [`SessionError::Transport`] on RFCOMM failure or `BT_CONNECTED` timeout,
/// [`SessionError::Map`] on OBEX failure.
pub async fn connect_map(
    addr: bluer::Address,
    channel: u8,
    bt_gate: Duration,
    security: Option<bluer::rfcomm::Security>,
) -> Result<MapClient<bluer::rfcomm::Stream>, SessionError> {
    let stream = transport::rfcomm::connect(addr, channel, bt_gate, security).await?;
    establish_map_session(stream).await
}

/// RFCOMM connect to `addr`:`channel` then PBAP OBEX CONNECT.
///
/// # Errors
///
/// Returns [`SessionError::Transport`] on RFCOMM failure, [`SessionError::Pbap`] on OBEX failure.
pub async fn connect_pbap(
    addr: bluer::Address,
    channel: u8,
) -> Result<PbapClient<bluer::rfcomm::Stream>, SessionError> {
    let stream = transport::rfcomm::connect(addr, channel, DEFAULT_BT_CONNECTED_GATE, None).await?;
    let client = PbapClient::connect(stream).await?;
    Ok(client)
}
