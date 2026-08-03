//! Real-socket tests for `send`, same fake-broker approach as `crate::daemon`'s own tests.

use crate::test_support::{bind_for, serve_one};

use super::*;

#[tokio::test]
async fn send_returns_broker_text_on_success() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:04:01";
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, ipc::BrokerResponse::Text("sent".to_owned())));

    let got = send(addr, "+15550001".to_owned(), "hi".to_owned()).await?;

    assert_eq!(got, "sent");
    server.await??;
    Ok(())
}

#[tokio::test]
async fn send_maps_unreachable_broker_to_write_error() -> anyhow::Result<()> {
    let Err(err) = send("TE:ST:00:00:04:02", "+15550001".to_owned(), "hi".to_owned()).await else {
        return Err(anyhow::anyhow!("expected an error when nothing is listening"));
    };
    assert!(matches!(err, broker_client::WriteError::Connect(_)));
    Ok(())
}
