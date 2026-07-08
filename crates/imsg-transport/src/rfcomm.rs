//! RFCOMM transport — Bluetooth Classic via [`bluer`].

use std::time::Duration;

use bluer::rfcomm::{Profile, Role, SocketAddr, Stream};
use futures::StreamExt;
use uuid::{uuid, Uuid};

use crate::TransportError;

/// Default `BT_CONNECTED` gate for callers without a configured budget (hub/PBAP paths).
/// The broker passes its own `cfg.broker.bt_connected()` instead.
pub const DEFAULT_BT_CONNECTED_GATE: Duration = Duration::from_secs(2);

const MNS_CHANNEL: u16 = 17;
const MNS_UUID: Uuid = uuid!("00001133-0000-1000-8000-00805f9b34fb");
const MNS_SERVICE_RECORD: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<record>
  <attribute id="0x0001"><sequence><uuid value="0x1133"/></sequence></attribute>
  <attribute id="0x0004">
    <sequence>
      <sequence><uuid value="0x0100"/></sequence>
      <sequence><uuid value="0x0003"/><uint8 value="0x11"/></sequence>
      <sequence><uuid value="0x0008"/></sequence>
    </sequence>
  </attribute>
  <attribute id="0x0009">
    <sequence><sequence><uuid value="0x1134"/><uint16 value="0x0101"/></sequence></sequence>
  </attribute>
  <attribute id="0x0100"><text value="MAP Message Notification Service"/></attribute>
  <attribute id="0x0315"><uint8 value="0x00"/></attribute>
  <attribute id="0x0316"><uint8 value="0x02"/></attribute>
</record>"#;

/// Yields incoming connection requests from the remote.
/// Keeps the [`bluer::Session`] alive — dropping this unregisters the SDP profile from `BlueZ`.
pub struct ProfileListener {
    _session: bluer::Session,
    handle: std::pin::Pin<Box<bluer::rfcomm::ProfileHandle>>,
}

impl ProfileListener {
    /// Returns `None` when the profile is unregistered or `BlueZ` terminates the session.
    pub async fn next(&mut self) -> Option<bluer::rfcomm::ConnectRequest> {
        self.handle.next().await
    }
}

/// Connects to `addr` on `channel` without SDP, then gates on `BT_CONNECTED` before returning.
///
/// `bluer`'s `Stream::connect` returns while the RFCOMM channel is still establishing
/// (`BT_CONNECT` state); the kernel confirms completion via `getpeername` succeeding rather
/// than `SO_ERROR`, which returns 0 prematurely for RFCOMM. This function polls `peer_addr`
/// every 25 ms for up to `bt_gate` so the first OBEX write never hits `ENOTCONN`. Use
/// [`DEFAULT_BT_CONNECTED_GATE`] when no configured budget applies.
///
/// # Errors
///
/// Returns [`TransportError::Io`] on socket creation, RFCOMM connect failure, or if the
/// link does not reach `BT_CONNECTED` within `bt_gate`.
pub async fn connect(
    addr: bluer::Address,
    channel: u8,
    bt_gate: Duration,
) -> Result<Stream, TransportError> {
    tracing::debug!("rfcomm: dialing {addr} ch{channel}");
    let stream = Stream::connect(SocketAddr::new(addr, channel)).await.inspect_err(|e| {
        tracing::warn!("rfcomm: socket connect to {addr} ch{channel} failed: {e}");
    })?;
    let deadline = tokio::time::Instant::now().checked_add(bt_gate).ok_or_else(|| {
        TransportError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "BT_CONNECTED deadline overflowed the monotonic clock",
        ))
    })?;
    await_bt_connected(|| stream.peer_addr().is_ok(), deadline).await.inspect_err(|e| {
        tracing::warn!("rfcomm: {addr} ch{channel} never reached BT_CONNECTED: {e}");
    })?;
    tracing::debug!("rfcomm: {addr} ch{channel} BT_CONNECTED");
    Ok(stream)
}

/// Polls `is_connected` every 25 ms until it returns `true` or `deadline` is exceeded.
///
/// In production `is_connected` wraps `stream.peer_addr().is_ok()`.  The plain `Fn`
/// boundary lets the timing logic be exercised without RFCOMM hardware in tests.
///
/// # Errors
///
/// Returns [`TransportError::Io`] with [`std::io::ErrorKind::TimedOut`] if the deadline
/// passes before `is_connected` returns `true`.
async fn await_bt_connected(
    is_connected: impl Fn() -> bool,
    deadline: tokio::time::Instant,
) -> Result<(), TransportError> {
    loop {
        if is_connected() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "RFCOMM link did not reach BT_CONNECTED within the gate",
            )));
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

/// Registers the MNS RFCOMM server profile with `BlueZ` on channel 17 and returns a listener.
///
/// Advertises the MAP Message Notification Service SDP record. Caller must keep the returned
/// [`ProfileListener`] alive — dropping it unregisters the profile. Does not validate that
/// `BlueZ` is powered or a device is paired.
///
/// # Errors
///
/// Returns [`TransportError::Io`] if the `BlueZ` D-Bus session cannot be established or
/// profile registration is rejected.
pub async fn listen_mns() -> Result<ProfileListener, TransportError> {
    let session = bluer::Session::new().await.map_err(std::io::Error::from)?;
    let profile = Profile {
        uuid: MNS_UUID,
        name: Some("MAP Message Notification Service".into()),
        channel: Some(MNS_CHANNEL),
        role: Some(Role::Server),
        service_record: Some(MNS_SERVICE_RECORD.into()),
        ..Default::default()
    };
    let handle = session.register_profile(profile).await.map_err(std::io::Error::from)?;
    Ok(ProfileListener { _session: session, handle: Box::pin(handle) })
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use socket2::{Domain, Socket, Type};

    use super::await_bt_connected;
    use crate::TransportError;

    // --- OS-invariant tests ---
    // Prove the kernel behaviour await_bt_connected relies on.
    // Use standard sockets — no RFCOMM hardware needed.

    /// `getpeername` on a newly created, unconnected socket returns `ENOTCONN`.
    ///
    /// This mirrors the `BT_CONNECT` state the RFCOMM socket is in when
    /// `Stream::connect().await` returns.  If this invariant ever breaks the
    /// `peer_addr().is_ok()` probe loses its meaning.
    #[test]
    fn unconnected_socket_peer_addr_returns_enotconn() -> io::Result<()> {
        let sock = Socket::new(Domain::IPV4, Type::STREAM, None)?;
        let result = sock.peer_addr();
        assert!(
            matches!(&result, Err(e) if e.kind() == io::ErrorKind::NotConnected),
            "expected ENOTCONN on unconnected socket, got: {result:?}"
        );
        Ok(())
    }

    /// `getpeername` on a connected socket returns the peer address.
    ///
    /// This mirrors the `BT_CONNECTED` state — proves `peer_addr().is_ok()` is the
    /// correct exit condition for the polling loop.
    #[test]
    fn connected_socket_peer_addr_succeeds() -> io::Result<()> {
        use std::net::{TcpListener, TcpStream};
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        let client = TcpStream::connect(addr)?;
        client.peer_addr()?;
        Ok(())
    }

    // --- Logic tests for await_bt_connected ---

    /// When `is_connected` is true on the first poll, the function returns immediately
    /// without sleeping — handles the case where `BT_CONNECTED` was reached before we poll.
    #[tokio::test]
    async fn resolves_immediately_when_already_connected() -> Result<(), TransportError> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        await_bt_connected(|| true, deadline).await
    }

    /// When `is_connected` is never true the function returns `TimedOut` after the deadline.
    ///
    /// Uses a 100 ms deadline so the test completes quickly.
    #[tokio::test]
    async fn times_out_when_link_never_establishes() {
        let deadline = tokio::time::Instant::now() + Duration::from_millis(100);
        let result = await_bt_connected(|| false, deadline).await;
        assert!(
            matches!(&result, Err(TransportError::Io(e)) if e.kind() == io::ErrorKind::TimedOut),
            "expected TimedOut, got: {result:?}"
        );
    }

    /// `is_connected` returns false for the first two polls then true — the loop retries
    /// and resolves successfully rather than giving up after the first `false`.
    #[tokio::test]
    async fn resolves_after_transient_not_connected() -> Result<(), TransportError> {
        let polls = Arc::new(AtomicU32::new(0));
        let polls2 = polls.clone();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        await_bt_connected(
            move || {
                let n = polls2.fetch_add(1, Ordering::Relaxed);
                n >= 2 // false for polls 0 and 1, true from poll 2 onward
            },
            deadline,
        )
        .await?;
        assert!(polls.load(Ordering::Relaxed) >= 3);
        Ok(())
    }
}
