//! imsg session broker.
//!
//! Owns the RFCOMM/OBEX/MAP session for the lifetime of a usage burst. Accepts
//! [`ipc::BrokerRequest`] frames over an abstract-namespace socket, dispatches them to
//! `imsg-session` functions, and returns [`ipc::BrokerResponse`] frames. Exits after
//! `cfg.broker.idle_secs` seconds of inactivity or on unrecoverable MAP failure.

mod runtime;

pub use runtime::{run, run_daemon};
