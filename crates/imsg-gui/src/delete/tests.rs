//! Real-socket tests for `delete`, same fake-broker approach as `crate::daemon`'s own tests.

use crate::test_support::{bind_for, serve_one};

use super::*;

#[tokio::test]
async fn delete_returns_broker_text_on_success() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:05:01";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, ipc::BrokerResponse::Text("deleted H1".to_owned())));

    let got = delete(addr, "H1".to_owned(), "telecom/msg/inbox".to_owned()).await?;

    assert_eq!(got, "deleted H1");
    server.await??;
    Ok(())
}

#[tokio::test]
async fn delete_maps_unreachable_broker_to_write_error() -> anyhow::Result<()> {
    let Err(err) =
        delete("TE:ST:00:00:05:02", "H1".to_owned(), "telecom/msg/inbox".to_owned()).await
    else {
        return Err(anyhow::anyhow!("expected an error when nothing is listening"));
    };
    assert!(matches!(err, broker_client::WriteError::Connect(_)));
    Ok(())
}
