//! IPC protocol types for the imsg session broker.
//!
//! Defines the request/response types exchanged over the Unix domain socket between the CLI
//! and the broker process. Framing uses `tokio-util` `LengthDelimitedCodec`; serialisation
//! uses `serde_json` for prototyping (to be replaced with `postcard` once the protocol is stable).

mod proto;
mod rows;
mod state;

pub use proto::{BrokerRequest, BrokerResponse, EventType, WatchEvent, MAX_FRAME_LEN};
pub use rows::{BodyDto, CardEntryDto, ContactDto, Direction, MessageDto, PhoneDto, ThreadDto};
pub use state::{Reason, SessionState};
