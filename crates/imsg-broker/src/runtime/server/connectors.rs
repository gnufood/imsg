//! Production connect-policy and connector/watcher factories — split out of `super` to keep
//! that module under the size ceiling.

use std::time::Duration;

use bluer::rfcomm::Stream;
use config::Config;
use futures::StreamExt as _;

use crate::runtime::types::{
    ConnectPolicy, Connector, LinkEvents, LinkState, LinkWatcher, PbapConnector,
};

/// Builds the connect-retry policy for one-shot (`serve`) mode: bounded attempts within a
/// wall-clock budget, so a CLI command fails fast and reports a clear error when the device
/// isn't reachable, rather than hanging.
pub(in crate::runtime) const fn build_policy(cfg: &Config) -> ConnectPolicy {
    ConnectPolicy {
        initial_backoff: cfg.broker.initial_backoff(),
        max_backoff: cfg.broker.max_backoff(),
        max_attempts: cfg.broker.connect_max_attempts,
        startup_budget: Some(cfg.broker.startup_budget()),
    }
}

/// Builds the connect-retry policy for persistent (`serve_daemon`) mode: unbounded attempts,
/// no wall-clock deadline. A daemon started before the phone is in Bluetooth range should keep
/// retrying (capped backoff, same schedule as one-shot mode) until it connects, not give up.
pub(in crate::runtime) const fn build_daemon_policy(cfg: &Config) -> ConnectPolicy {
    ConnectPolicy {
        initial_backoff: cfg.broker.initial_backoff(),
        max_backoff: cfg.broker.max_backoff(),
        max_attempts: u32::MAX,
        startup_budget: None,
    }
}

/// Builds the production connector: every call establishes a fresh RFCOMM/OBEX MAP session to
/// `addr`:`channel`, gating on `BT_CONNECTED` up to `bt_gate` and requesting `security` (if
/// any) from the kernel. Defined here so the transport-specific stream type stays out of the
/// actor.
pub(super) fn make_connector(
    addr: bluer::Address,
    channel: u8,
    bt_gate: Duration,
    security: Option<bluer::rfcomm::Security>,
) -> Connector<Stream> {
    Box::new(move || Box::pin(session::lifecycle::connect_map(addr, channel, bt_gate, security)))
}

/// Builds the production link watcher: each subscription asks `BlueZ` for `addr`'s connection
/// reports.
///
/// A subscription that cannot be established yields a stream that never reports, so a D-Bus
/// failure degrades to the pre-existing mid-op detection instead of being mistaken for a drop.
pub(super) fn make_link_watcher(addr: bluer::Address) -> LinkWatcher {
    Box::new(move || {
        Box::pin(async move {
            let events: LinkEvents = match transport::rfcomm::link_events(addr).await {
                Ok(events) => {
                    Box::pin(events.map(|up| if up { LinkState::Up } else { LinkState::Down }))
                }
                Err(e) => {
                    tracing::warn!(
                        "link watch for {addr} unavailable, falling back to mid-op detection: {e}"
                    );
                    Box::pin(futures::stream::pending())
                }
            };
            events
        })
    })
}

/// Maps the configured [`config::SecurityLevel`] to the `bluer::rfcomm::Security` value
/// [`make_connector`] requests from the kernel. `None` means imsg makes no explicit request —
/// the kernel/BlueZ default (whatever the existing pairing/bond negotiated) applies unchanged.
/// `key_size: 0` is the `BT_SECURITY` convention for "any size" — this only pins the policy
/// tier, not a minimum key length.
pub(super) const fn security_from_config(
    level: Option<config::SecurityLevel>,
) -> Option<bluer::rfcomm::Security> {
    let Some(level) = level else { return None };
    let level = match level {
        config::SecurityLevel::Sdp => bluer::rfcomm::SecurityLevel::Sdp,
        config::SecurityLevel::Low => bluer::rfcomm::SecurityLevel::Low,
        config::SecurityLevel::Medium => bluer::rfcomm::SecurityLevel::Medium,
        config::SecurityLevel::High => bluer::rfcomm::SecurityLevel::High,
    };
    Some(bluer::rfcomm::Security { level, key_size: 0 })
}

/// Builds the production PBAP connector: every call establishes a fresh, short-lived RFCOMM/OBEX
/// PBAP session to `addr`:`channel` — no persistent session, no notification registration.
pub(super) fn make_pbap_connector(addr: bluer::Address, channel: u8) -> PbapConnector<Stream> {
    Box::new(move || Box::pin(session::lifecycle::connect_pbap(addr, channel)))
}
