//! Real-socket tests for [`super::sync_contacts`], same fake-broker approach as
//! `write/tests.rs`.

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::{BrokerRequest, BrokerResponse, Reason, RefreshDto, SyncReportDto, MAX_FRAME_LEN};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::{sync_contacts, ContactsError};
use crate::CallError;

fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

/// Accepts one connection, decodes the request, replies with `resp`.
async fn serve_one(listener: Listener, resp: BrokerResponse) -> anyhow::Result<()> {
    let stream = listener.accept().await?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    let mut framed = Framed::new(stream, codec);
    let frame = framed.next().await.ok_or_else(|| anyhow::anyhow!("no request frame"))??;
    let _req: BrokerRequest = serde_json::from_slice(&frame)?;
    let bytes = Bytes::from(serde_json::to_vec(&resp)?);
    framed.send(bytes).await?;
    Ok(())
}

#[tokio::test]
async fn sync_contacts_returns_report_on_success() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:04:01";
    let listener = bind_for(addr)?;
    let report = SyncReportDto::Refreshed(RefreshDto {
        listed: 8,
        pull_failed: 1,
        no_uid: 0,
        written: 7,
        wiped: false,
    });
    let server = tokio::spawn(serve_one(listener, BrokerResponse::ContactsSynced { report }));

    let got = sync_contacts(addr).await?;

    assert_eq!(got, report);
    server.await??;
    Ok(())
}

/// The no-op outcome has to survive the wire as itself, not collapse into a zero count — the
/// whole reason the response carries a report.
#[tokio::test]
async fn sync_contacts_returns_up_to_date_distinctly() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:04:05";
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(
        listener,
        BrokerResponse::ContactsSynced { report: SyncReportDto::UpToDate },
    ));

    let got = sync_contacts(addr).await?;

    assert_eq!(got, SyncReportDto::UpToDate);
    server.await??;
    Ok(())
}

#[tokio::test]
async fn sync_contacts_maps_failed_response_to_call_error() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:04:02";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, BrokerResponse::Failed(Reason::DeviceUnreachable)));

    let Err(err) = sync_contacts(addr).await else {
        return Err(anyhow::anyhow!("expected an error for a Failed response"));
    };
    assert!(matches!(err, ContactsError::Call(CallError::Failed(Reason::DeviceUnreachable))));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn sync_contacts_maps_error_response_to_call_error() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:04:03";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, BrokerResponse::Error("broker shutting down".to_owned())));

    let Err(err) = sync_contacts(addr).await else {
        return Err(anyhow::anyhow!("expected an error for an Error response"));
    };
    assert!(
        matches!(err, ContactsError::Call(CallError::Error(msg)) if msg == "broker shutting down")
    );
    server.await??;
    Ok(())
}

#[tokio::test]
async fn sync_contacts_maps_unreachable_broker_to_connect_error() -> anyhow::Result<()> {
    let Err(err) = sync_contacts("TE:ST:00:00:04:04").await else {
        return Err(anyhow::anyhow!("expected an error when nothing is listening"));
    };
    assert!(matches!(err, ContactsError::Connect(_)));
    Ok(())
}
