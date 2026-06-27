//! Session-broker lifecycle and startup-timing policy.

use std::time::Duration;

use serde::Deserialize;

use crate::ConfigError;

/// Margin the CLI readiness deadline must keep above the broker's own startup budget, so the
/// CLI never gives up while the broker is still legitimately connecting. Covers IPC round-trip.
const READINESS_MARGIN_SECS: u64 = 2;

/// Broker lifecycle and startup-timing policy.
///
/// All durations are whole-unit integers (`*_secs`, `*_ms`) following the `idle_secs` convention;
/// read them through the [`Duration`] accessors so unit interpretation stays in one place.
/// Configurable via `IMSG_BROKER__*` env vars.
///
/// The fields form a single budget: a startup may make up to `connect_max_attempts` attempts,
/// each gated by `bt_connected_secs`, separated by backoff from `initial_backoff_ms` doubling to
/// `max_backoff_secs`, bounded overall by `startup_budget_secs`. The CLI waits `readiness_wait_secs`
/// (polling every `readiness_poll_ms`) for the broker to come up — validated to exceed the budget.
#[derive(Debug, Deserialize)]
pub struct BrokerConfig {
    /// Seconds of inactivity after which the broker disconnects MAP and exits. Default: 15.
    pub idle_secs: u64,
    /// Maximum attempts to establish the MAP session before startup fails. Always `>= 1`. Default: 3.
    pub connect_max_attempts: u32,
    /// Per-attempt timeout for the RFCOMM `BT_CONNECTED` gate, in seconds. Default: 5.
    pub bt_connected_secs: u64,
    /// Initial backoff between MAP connect attempts, in milliseconds. Default: 500.
    pub initial_backoff_ms: u64,
    /// Backoff ceiling between MAP connect attempts, in seconds. Default: 30.
    pub max_backoff_secs: u64,
    /// Total wall-clock budget the broker may spend establishing the session, in seconds. Default: 30.
    pub startup_budget_secs: u64,
    /// CLI deadline awaiting broker readiness, in seconds. Exceeds `startup_budget_secs` by at
    /// least the IPC margin (validated). Default: 40.
    pub readiness_wait_secs: u64,
    /// CLI poll interval while awaiting broker readiness, in milliseconds. Default: 50.
    pub readiness_poll_ms: u64,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            idle_secs: 15,
            connect_max_attempts: 3,
            bt_connected_secs: 5,
            initial_backoff_ms: 500,
            max_backoff_secs: 30,
            startup_budget_secs: 30,
            readiness_wait_secs: 40,
            readiness_poll_ms: 50,
        }
    }
}

impl BrokerConfig {
    /// Inactivity timeout before the broker disconnects MAP and exits.
    #[must_use]
    pub const fn idle(&self) -> Duration {
        Duration::from_secs(self.idle_secs)
    }

    /// Per-attempt RFCOMM `BT_CONNECTED` gate.
    #[must_use]
    pub const fn bt_connected(&self) -> Duration {
        Duration::from_secs(self.bt_connected_secs)
    }

    /// Initial backoff between MAP connect attempts.
    #[must_use]
    pub const fn initial_backoff(&self) -> Duration {
        Duration::from_millis(self.initial_backoff_ms)
    }

    /// Ceiling for the backoff between MAP connect attempts.
    #[must_use]
    pub const fn max_backoff(&self) -> Duration {
        Duration::from_secs(self.max_backoff_secs)
    }

    /// Total wall-clock budget for establishing the session at startup.
    #[must_use]
    pub const fn startup_budget(&self) -> Duration {
        Duration::from_secs(self.startup_budget_secs)
    }

    /// CLI deadline awaiting broker readiness.
    #[must_use]
    pub const fn readiness_wait(&self) -> Duration {
        Duration::from_secs(self.readiness_wait_secs)
    }

    /// CLI poll interval while awaiting broker readiness.
    #[must_use]
    pub const fn readiness_poll(&self) -> Duration {
        Duration::from_millis(self.readiness_poll_ms)
    }

    /// Enforces that startup timing is internally consistent so the CLI cannot give up while the
    /// broker is still connecting.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Invalid`] when `connect_max_attempts` is `0`, or when
    /// `readiness_wait_secs` does not exceed `startup_budget_secs` by at least the IPC margin.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.connect_max_attempts == 0 {
            return Err(ConfigError::Invalid {
                field: "broker.connect_max_attempts",
                msg: "must be >= 1".to_owned(),
            });
        }
        if self.readiness_wait_secs < self.startup_budget_secs.saturating_add(READINESS_MARGIN_SECS)
        {
            return Err(ConfigError::Invalid {
                field: "broker.readiness_wait_secs",
                msg: format!(
                    "{} must exceed startup_budget_secs ({}) by at least {READINESS_MARGIN_SECS}s",
                    self.readiness_wait_secs, self.startup_budget_secs
                ),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() -> Result<(), ConfigError> {
        BrokerConfig::default().validate()
    }

    #[test]
    fn rejects_zero_attempts() {
        let b = BrokerConfig { connect_max_attempts: 0, ..BrokerConfig::default() };
        assert!(matches!(
            b.validate(),
            Err(ConfigError::Invalid { field: "broker.connect_max_attempts", .. })
        ));
    }

    #[test]
    fn rejects_readiness_below_budget_plus_margin() {
        let b = BrokerConfig {
            startup_budget_secs: 30,
            readiness_wait_secs: 31,
            ..BrokerConfig::default()
        };
        assert!(matches!(
            b.validate(),
            Err(ConfigError::Invalid { field: "broker.readiness_wait_secs", .. })
        ));
    }

    #[test]
    fn accessors_apply_correct_units() {
        let b = BrokerConfig::default();
        assert_eq!(b.bt_connected(), Duration::from_secs(5));
        assert_eq!(b.initial_backoff(), Duration::from_millis(500));
        assert_eq!(b.readiness_poll(), Duration::from_millis(50));
    }
}
