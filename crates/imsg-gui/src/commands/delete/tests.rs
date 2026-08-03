//! Real-socket tests for `delete`, same fake-broker approach as `crate::delete`'s own tests.

use crate::test_support::{bind_for, serve_one};

use super::*;

#[tokio::test]
async fn delete_returns_broker_text_on_success() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:07:01";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, ipc::BrokerResponse::Text("deleted H1".to_owned())));

    let got = delete(addr.to_owned(), "H1".to_owned(), "telecom/msg/inbox".to_owned()).await?;

    assert_eq!(got, "deleted H1");
    server.await??;
    Ok(())
}

#[tokio::test]
async fn delete_maps_unreachable_broker_to_command_error() -> anyhow::Result<()> {
    let result =
        delete("TE:ST:00:00:07:02".to_owned(), "H1".to_owned(), "telecom/msg/inbox".to_owned())
            .await;
    assert!(result.is_err());
    Ok(())
}
