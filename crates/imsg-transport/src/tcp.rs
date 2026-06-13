//! TCP transport — TCP bridge for remote Bluetooth dev.

use std::net::SocketAddr;

use tokio::net::TcpStream;

use crate::TransportError;

/// Production: use [`super::rfcomm::connect`].
///
/// # Errors
///
/// Returns [`TransportError::Io`] on TCP connect failure.
pub async fn connect(addr: SocketAddr) -> Result<TcpStream, TransportError> {
    let stream = TcpStream::connect(addr).await?;
    Ok(stream)
}
