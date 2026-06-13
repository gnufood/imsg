//! RFCOMM transport — Bluetooth Classic via [`bluer`].

use bluer::rfcomm::{Profile, Role, SocketAddr, Stream};
use futures::StreamExt;
use uuid::{uuid, Uuid};

use crate::TransportError;

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

/// Connects to `addr` on `channel` without SDP — channel must be known in advance.
///
/// # Errors
///
/// Returns [`TransportError::Io`] on socket creation or RFCOMM connect failure.
pub async fn connect(addr: bluer::Address, channel: u8) -> Result<Stream, TransportError> {
    let stream = Stream::connect(SocketAddr::new(addr, channel)).await?;
    Ok(stream)
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
