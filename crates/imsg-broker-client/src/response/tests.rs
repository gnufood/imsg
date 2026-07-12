//! Pure data transformation — no I/O, no fakes needed.

use ipc::{BrokerResponse, Reason, SessionState};

use super::*;

#[test]
fn text_extracts_success_string() -> anyhow::Result<()> {
    let resp = BrokerResponse::Text("sent".to_owned());
    let text = text_result(resp).map_err(|e| anyhow::anyhow!("expected Ok, got {e}"))?;
    assert_eq!(text, "sent");
    Ok(())
}

#[test]
fn failed_maps_to_call_error_failed() -> anyhow::Result<()> {
    let resp = BrokerResponse::Failed(Reason::DeviceUnreachable);
    let Err(err) = text_result(resp) else {
        return Err(anyhow::anyhow!("expected an error for a Failed response"));
    };
    assert!(matches!(err, CallError::Failed(Reason::DeviceUnreachable)));
    Ok(())
}

#[test]
fn error_maps_to_call_error_error() -> anyhow::Result<()> {
    let resp = BrokerResponse::Error("broker shutting down".to_owned());
    let Err(err) = text_result(resp) else {
        return Err(anyhow::anyhow!("expected an error for an Error response"));
    };
    assert!(matches!(err, CallError::Error(msg) if msg == "broker shutting down"));
    Ok(())
}

#[test]
fn other_variant_maps_to_unexpected() -> anyhow::Result<()> {
    let resp = BrokerResponse::StatusInfo {
        state: SessionState::Active,
        device: "TE:ST:00:00:00:01".into(),
        persistent: true,
    };
    let Err(err) = text_result(resp) else {
        return Err(anyhow::anyhow!("expected an error for a StatusInfo response"));
    };
    assert!(
        matches!(err, CallError::Unexpected(boxed) if matches!(*boxed, BrokerResponse::StatusInfo { .. }))
    );
    Ok(())
}
