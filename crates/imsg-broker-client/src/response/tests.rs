//! Pure data transformation — no I/O, no fakes needed.

use ipc::{BrokerResponse, FolderDto, Reason, SessionState};

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
fn folders_extracts_rows_preserving_order() -> anyhow::Result<()> {
    let resp = BrokerResponse::Folders(vec![
        FolderDto { name: "inbox".to_owned() },
        FolderDto { name: "deleted".to_owned() },
    ]);

    let rows = folders_result(resp).map_err(|e| anyhow::anyhow!("expected Ok, got {e}"))?;

    let names: Vec<String> = rows.into_iter().map(|f| f.name).collect();
    assert_eq!(names, ["inbox", "deleted"]);
    Ok(())
}

/// An empty listing is a successful read, not a failure — the device genuinely reporting no
/// folders must not be indistinguishable from a rejected request.
#[test]
fn folders_extracts_empty_listing_as_success() -> anyhow::Result<()> {
    let rows = folders_result(BrokerResponse::Folders(vec![]))
        .map_err(|e| anyhow::anyhow!("expected Ok, got {e}"))?;
    assert!(rows.is_empty());
    Ok(())
}

#[test]
fn folders_maps_failed_response_to_call_error() -> anyhow::Result<()> {
    let Err(err) = folders_result(BrokerResponse::Failed(Reason::DeviceUnreachable)) else {
        return Err(anyhow::anyhow!("expected an error for a Failed response"));
    };
    assert!(matches!(err, CallError::Failed(Reason::DeviceUnreachable)));
    Ok(())
}

#[test]
fn folders_maps_wrong_row_type_to_unexpected() -> anyhow::Result<()> {
    let Err(err) = folders_result(BrokerResponse::Threads(vec![])) else {
        return Err(anyhow::anyhow!("expected an error for a Threads response"));
    };
    assert!(
        matches!(err, CallError::Unexpected(boxed) if matches!(*boxed, BrokerResponse::Threads(_)))
    );
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
