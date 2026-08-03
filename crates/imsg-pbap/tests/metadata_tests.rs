//! Drives `phonebook_metadata` over an in-memory duplex pair, verifying `MaxListCount=0`
//! triggers a bodyless response and that `AppParams` TLVs land in the right `PhonebookMetadata` fields.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_pbap::{client::PbapClient, phonebook::PhonebookPath, PbapError};
use obex_core::headers::Header;
use obex_core::packet::{OpCode, Packet, PacketExtra};

const CONNECT_RSP: &[u8] = include_bytes!("fixtures/pbap_connect_rsp.bin");

fn tlv(tag: u8, value: &[u8]) -> Vec<u8> {
    let mut out = vec![tag, u8::try_from(value.len()).unwrap_or(u8::MAX)];
    out.extend_from_slice(value);
    out
}

fn metadata_ok_response(
    db_id: [u8; 16],
    primary: [u8; 16],
    secondary: [u8; 16],
    size: u16,
) -> Result<Bytes, PbapError> {
    let mut app_params = tlv(0x08, &size.to_be_bytes());
    app_params.extend(tlv(0x0D, &db_id));
    app_params.extend(tlv(0x0A, &primary));
    app_params.extend(tlv(0x0B, &secondary));
    Packet {
        opcode: OpCode::Ok,
        extra: PacketExtra::None,
        headers: vec![Header::AppParams(Bytes::from(app_params))],
    }
    .encode()
    .map_err(|_| PbapError::UnexpectedEof)
}

#[tokio::test]
async fn phonebook_metadata_parses_response_app_params() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);
    let db_id = [0xAA; 16];
    let primary = [0xBB; 16];
    let secondary = [0xCC; 16];

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert!(req.windows(14).any(|w| w == b"x-bt/phonebook"));
            assert!(req.windows(7).any(|w| w == [0x07, 0x01, 0x01, 0x04, 0x02, 0x00, 0x00]));
            srv.send(metadata_ok_response(db_id, primary, secondary, 5)?)
                .await
                .map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.phonebook_metadata(PhonebookPath::Pb).await
        },
    );
    server_result?;
    let meta = client_result?;
    assert_eq!(meta.size, Some(5));
    assert_eq!(meta.database_id, Some(db_id));
    assert_eq!(meta.primary_version, Some(primary));
    assert_eq!(meta.secondary_version, Some(secondary));
    Ok(())
}

#[tokio::test]
async fn phonebook_metadata_no_body_fetched() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let _ =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            srv.send(metadata_ok_response([0; 16], [0; 16], [0; 16], 0)?)
                .await
                .map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.phonebook_metadata(PhonebookPath::Pb).await
        },
    );
    server_result?;
    let meta = client_result?;
    assert_eq!(meta.size, Some(0));
    Ok(())
}
