//! Connection lifecycle: SDP lookup, RFCOMM connect, OBEX session establishment and teardown.

use map_core::client::MapClient;
use pbap_core::client::PbapClient;
use tokio::io::{AsyncRead, AsyncWrite};

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
    let mut client = MapClient::connect(stream).await?;
    client.set_notification_registration(true).await?;
    Ok(client)
}

/// RFCOMM connect to `addr`:`channel` then [`establish_map_session`].
///
/// # Errors
///
/// Returns [`SessionError::Transport`] on RFCOMM failure, [`SessionError::Map`] on OBEX failure.
pub async fn connect_map(
    addr: bluer::Address,
    channel: u8,
) -> Result<MapClient<bluer::rfcomm::Stream>, SessionError> {
    let stream = transport::rfcomm::connect(addr, channel).await?;
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
    let stream = transport::rfcomm::connect(addr, channel).await?;
    let client = PbapClient::connect(stream).await?;
    Ok(client)
}
