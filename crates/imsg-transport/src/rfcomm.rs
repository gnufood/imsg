//! RFCOMM transport — Bluetooth Classic via [`bluer`].

use std::time::Duration;

use bluer::rfcomm::{Profile, Role, Security, Socket, SocketAddr, Stream};
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
/// `security`, if given, is requested from the kernel via `setsockopt(BT_SECURITY)` before
/// connecting; `None` leaves the socket's security unmodified — whatever the existing
/// pairing/bond already negotiated applies unchanged.
///
/// # Errors
///
/// Returns [`TransportError::Io`] on socket creation, rejection of a requested `security`
/// level, RFCOMM connect failure, or if the link does not reach `BT_CONNECTED` within
/// `bt_gate`.
pub async fn connect(
    addr: bluer::Address,
    channel: u8,
    bt_gate: Duration,
    security: Option<Security>,
) -> Result<Stream, TransportError> {
    tracing::debug!("rfcomm: dialing {addr} ch{channel}");
    let socket = Socket::new()?;
    if let Some(security) = security {
        socket.set_security(security)?;
    }
    let stream = socket.connect(SocketAddr::new(addr, channel)).await.inspect_err(|e| {
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

/// Subscribes to `BlueZ`'s connection reports for `addr`.
///
/// Yields `true` when the device is reachable and `false` when `BlueZ` reports it gone. The
/// first item is the state at subscription time, not a transition — a caller that subscribes
/// after the device already dropped is told so immediately instead of waiting for an edge that
/// has already passed.
///
/// The stream ends when `BlueZ` stops reporting at all (`bluetoothd` restart, D-Bus loss, the
/// device removed). Termination says nothing about the link, so callers must resubscribe rather
/// than read the end of the stream as "still connected".
///
/// This reports the *device*'s ACL link, which is coarser than one profile's channel: a phone
/// still connected for another profile reads as `true` even when this RFCOMM channel is gone.
///
/// # Errors
///
/// Returns [`TransportError::Io`] if the D-Bus session or default adapter cannot be reached, or
/// if `addr` is unknown to `BlueZ`.
pub async fn link_events(
    addr: bluer::Address,
) -> Result<impl futures::Stream<Item = bool>, TransportError> {
    let session = bluer::Session::new().await.map_err(std::io::Error::from)?;
    let adapter = session.default_adapter().await.map_err(std::io::Error::from)?;
    let device = adapter.device(addr).map_err(std::io::Error::from)?;
    let events = device.events().await.map_err(std::io::Error::from)?;
    let initial = device.is_connected().await.map_err(std::io::Error::from)?;
    let transitions = events.filter_map(|ev| async move {
        match ev {
            bluer::DeviceEvent::PropertyChanged(bluer::DeviceProperty::Connected(up)) => Some(up),
            bluer::DeviceEvent::PropertyChanged(_) => None,
        }
    });
    Ok(futures::stream::once(async move { initial }).chain(transitions))
}

// polls is_connected every 25ms; prod wraps peer_addr().is_ok(). Fn boundary lets
// the timing logic run without RFCOMM hardware in tests; returns TimedOut once
// deadline passes before is_connected succeeds
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
mod tests;
