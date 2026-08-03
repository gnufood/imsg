//! Real-socket tests, same fake-broker approach as `crate::sync`'s tests — no mocks.

use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::{BrokerResponse, RefreshDto, SyncReportDto, MAX_FRAME_LEN};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};

use super::*;

fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

async fn serve_one(listener: Listener, resp: BrokerResponse) -> anyhow::Result<()> {
    let stream = listener.accept().await?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    let mut framed = Framed::new(stream, codec);
    let frame = framed.next().await.ok_or_else(|| anyhow::anyhow!("no request frame"))??;
    let _req: ipc::BrokerRequest = serde_json::from_slice(&frame)?;
    let bytes = Bytes::from(serde_json::to_vec(&resp)?);
    framed.send(bytes).await?;
    Ok(())
}

#[tokio::test]
async fn succeeds_silently_on_success() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:06:01";
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(
        listener,
        BrokerResponse::ContactsSynced { report: SyncReportDto::UpToDate },
    ));

    ensure_synced_best_effort(addr).await;

    server.await??;
    Ok(())
}

#[tokio::test]
async fn swallows_failure_when_broker_unreachable() {
    // No listener bound — the broker call must fail, but this must not panic or propagate.
    ensure_synced_best_effort("TE:ST:00:00:06:02").await;
}

#[tokio::test]
async fn sync_now_returns_report_on_success() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:06:03";
    let listener = bind_for(addr)?;
    let report = SyncReportDto::Refreshed(RefreshDto {
        listed: 6,
        pull_failed: 1,
        no_uid: 0,
        written: 5,
        wiped: false,
    });
    let server = tokio::spawn(serve_one(listener, BrokerResponse::ContactsSynced { report }));

    let got = sync_now(addr).await?;

    assert_eq!(got, report, "the GUI must see the same report the CLI does, not a bare count");
    server.await??;
    Ok(())
}

#[tokio::test]
async fn sync_now_propagates_failure_when_broker_unreachable() {
    // No listener bound — unlike `ensure_synced_best_effort`, this must surface the error.
    let result = sync_now("TE:ST:00:00:06:04").await;
    assert!(result.is_err());
}
