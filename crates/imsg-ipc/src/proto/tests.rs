//! Wire round-trip tests for [`super::BrokerRequest`]/[`super::BrokerResponse`].

use super::*;
use crate::Direction;

/// Regression: internally-tagged newtype-of-`String` variants fail to serialise. Adjacent
/// tagging fixes it, so `Error`/`Text`/`Failed` frames must round-trip.
#[test]
fn newtype_response_variants_roundtrip() -> Result<(), serde_json::Error> {
    let cases = [
        BrokerResponse::Ok,
        BrokerResponse::Text("listed 3".into()),
        BrokerResponse::Error("broker shutting down".into()),
        BrokerResponse::Failed(Reason::DeviceUnreachable),
        BrokerResponse::Failed(Reason::OperationFailed("handle 7".into())),
    ];
    for resp in cases {
        let json = serde_json::to_string(&resp)?;
        let back: BrokerResponse = serde_json::from_str(&json)?;
        assert_eq!(format!("{resp:?}"), format!("{back:?}"));
    }
    Ok(())
}

#[test]
fn live_data_response_variants_roundtrip() -> Result<(), serde_json::Error> {
    let cases = [
        BrokerResponse::Messages(vec![MessageDto {
            handle: "7".into(),
            timestamp_ms: 1_700_000_000_000,
            address: "+15550001".into(),
            folder: "inbox".into(),
            read: false,
            text: "hi".into(),
        }]),
        BrokerResponse::Threads(vec![ThreadDto {
            address: "+15550001".into(),
            latest_ms: 1_700_000_000_000,
            total: 4,
            unread: 1,
        }]),
        BrokerResponse::Body(BodyDto {
            handle: "7".into(),
            direction: Direction::Received,
            address: "+15550001".into(),
            folder: "telecom/msg/inbox".into(),
            read: true,
            text: "hi".into(),
        }),
    ];
    for resp in cases {
        let json = serde_json::to_string(&resp)?;
        let back: BrokerResponse = serde_json::from_str(&json)?;
        assert_eq!(format!("{resp:?}"), format!("{back:?}"));
    }
    Ok(())
}

#[test]
fn live_request_variants_roundtrip() -> Result<(), serde_json::Error> {
    let cases = [
        BrokerRequest::ListMessages {
            folder: Some("sent".into()),
            unread: true,
            from: Some("+15550001".into()),
            since: None,
            limit: Some(20),
            offset: 0,
        },
        BrokerRequest::GetMessage { handle: "7".into() },
        BrokerRequest::Threads,
        BrokerRequest::MarkReadDevice { handle: "7".into() },
        BrokerRequest::SendLive { number: "+15550001".into(), message: "hi".into() },
        BrokerRequest::Shutdown,
    ];
    for req in cases {
        let json = serde_json::to_string(&req)?;
        let back: BrokerRequest = serde_json::from_str(&json)?;
        assert_eq!(format!("{req:?}"), format!("{back:?}"));
    }
    Ok(())
}

#[test]
fn status_info_carries_state() -> Result<(), serde_json::Error> {
    let resp = BrokerResponse::StatusInfo {
        state: SessionState::Reconnecting,
        device: "AA:BB:CC:DD:EE:FF".into(),
        persistent: true,
    };
    let json = serde_json::to_string(&resp)?;
    let back: BrokerResponse = serde_json::from_str(&json)?;
    assert!(matches!(
        back,
        BrokerResponse::StatusInfo { state: SessionState::Reconnecting, persistent: true, .. }
    ));
    Ok(())
}
