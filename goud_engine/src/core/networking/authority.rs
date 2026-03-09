//! Authority policy system for server-side validation of client commands.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::core::providers::network_types::ConnectionId;

/// Validation input passed to authority policies.
#[derive(Debug, Clone)]
pub struct ValidationContext<'a> {
    /// Client connection ID that submitted the command.
    pub connection: ConnectionId,
    /// Opaque command payload bytes.
    pub payload: &'a [u8],
    /// Server-local timestamp of validation.
    pub received_at: Instant,
}

/// Result of authority validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorityDecision {
    /// Command is valid and may be applied server-side.
    Accept,
    /// Command is invalid and must be rejected.
    Reject {
        /// Human-readable rejection reason.
        reason: String,
    },
}

/// Pluggable authority policy used by session servers.
pub trait AuthorityPolicy: Send {
    /// Validates one client command.
    fn validate(&mut self, context: &ValidationContext<'_>) -> AuthorityDecision;

    /// Optional lifecycle callback when a client disconnects.
    fn on_client_disconnected(&mut self, _connection: ConnectionId) {}
}

/// Built-in policy selector.
#[derive(Debug, Clone)]
pub enum BuiltInAuthorityPolicy {
    /// Accept all commands.
    AllowAll,
    /// Validate payload schema tag + payload length bounds.
    SchemaBounds(SchemaBoundsConfig),
    /// Enforce per-client command rate limits.
    RateLimited(RateLimitConfig),
}

impl BuiltInAuthorityPolicy {
    /// Builds a boxed authority policy from this selector.
    pub fn build(self) -> Box<dyn AuthorityPolicy> {
        match self {
            Self::AllowAll => Box::new(AllowAllAuthority),
            Self::SchemaBounds(config) => Box::new(SchemaBoundsAuthority::new(config)),
            Self::RateLimited(config) => Box::new(RateLimitedAuthority::new(config)),
        }
    }
}

/// Authority policy that accepts all commands.
#[derive(Debug, Default)]
pub struct AllowAllAuthority;

impl AuthorityPolicy for AllowAllAuthority {
    fn validate(&mut self, _context: &ValidationContext<'_>) -> AuthorityDecision {
        AuthorityDecision::Accept
    }
}

/// Configuration for [`SchemaBoundsAuthority`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaBoundsConfig {
    /// Minimum allowed payload size in bytes.
    pub min_payload_len: usize,
    /// Maximum allowed payload size in bytes.
    pub max_payload_len: usize,
    /// Optional set of allowed command tags (first payload byte).
    pub allowed_command_tags: Option<Vec<u8>>,
}

impl SchemaBoundsConfig {
    /// Creates a config with no tag restriction.
    pub fn new(min_payload_len: usize, max_payload_len: usize) -> Self {
        Self {
            min_payload_len,
            max_payload_len,
            allowed_command_tags: None,
        }
    }
}

/// Schema + bounds authority validator.
#[derive(Debug, Clone)]
pub struct SchemaBoundsAuthority {
    config: SchemaBoundsConfig,
}

impl SchemaBoundsAuthority {
    /// Creates a schema+bounds authority validator.
    pub fn new(config: SchemaBoundsConfig) -> Self {
        Self { config }
    }
}

impl AuthorityPolicy for SchemaBoundsAuthority {
    fn validate(&mut self, context: &ValidationContext<'_>) -> AuthorityDecision {
        let payload_len = context.payload.len();

        if payload_len < self.config.min_payload_len || payload_len > self.config.max_payload_len {
            return AuthorityDecision::Reject {
                reason: format!(
                    "Payload length {} outside bounds {}..={}",
                    payload_len, self.config.min_payload_len, self.config.max_payload_len
                ),
            };
        }

        if let Some(allowed_tags) = &self.config.allowed_command_tags {
            let Some(tag) = context.payload.first() else {
                return AuthorityDecision::Reject {
                    reason: "Payload missing required command tag".to_string(),
                };
            };

            if !allowed_tags.contains(tag) {
                return AuthorityDecision::Reject {
                    reason: format!("Command tag {} is not allowed", tag),
                };
            }
        }

        AuthorityDecision::Accept
    }
}

/// Configuration for [`RateLimitedAuthority`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitConfig {
    /// Maximum number of accepted commands in one window.
    pub max_commands: usize,
    /// Sliding window duration.
    pub window: Duration,
}

impl RateLimitConfig {
    /// Creates a rate limit config.
    pub fn new(max_commands: usize, window: Duration) -> Self {
        Self {
            max_commands,
            window,
        }
    }
}

/// Per-client rate limiter authority policy.
#[derive(Debug, Clone)]
pub struct RateLimitedAuthority {
    config: RateLimitConfig,
    history: HashMap<ConnectionId, VecDeque<Instant>>,
}

impl RateLimitedAuthority {
    /// Creates a rate-limited authority validator.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            history: HashMap::new(),
        }
    }

    fn purge_old_entries(entries: &mut VecDeque<Instant>, now: Instant, window: Duration) {
        while let Some(oldest) = entries.front() {
            if now.duration_since(*oldest) > window {
                entries.pop_front();
            } else {
                break;
            }
        }
    }
}

impl AuthorityPolicy for RateLimitedAuthority {
    fn validate(&mut self, context: &ValidationContext<'_>) -> AuthorityDecision {
        let entries = self.history.entry(context.connection).or_default();
        Self::purge_old_entries(entries, context.received_at, self.config.window);

        if entries.len() >= self.config.max_commands {
            return AuthorityDecision::Reject {
                reason: format!(
                    "Rate limit exceeded: max {} commands per {:?}",
                    self.config.max_commands, self.config.window
                ),
            };
        }

        entries.push_back(context.received_at);
        AuthorityDecision::Accept
    }

    fn on_client_disconnected(&mut self, connection: ConnectionId) {
        self.history.remove(&connection);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_all_authority_accepts_any_payload() {
        let mut authority = AllowAllAuthority;
        let decision = authority.validate(&ValidationContext {
            connection: ConnectionId(1),
            payload: &[9, 8, 7],
            received_at: Instant::now(),
        });
        assert_eq!(decision, AuthorityDecision::Accept);
    }

    #[test]
    fn schema_bounds_rejects_out_of_bounds_payload() {
        let config = SchemaBoundsConfig::new(2, 4);
        let mut authority = SchemaBoundsAuthority::new(config);

        let decision = authority.validate(&ValidationContext {
            connection: ConnectionId(1),
            payload: &[1],
            received_at: Instant::now(),
        });

        assert!(matches!(decision, AuthorityDecision::Reject { .. }));
    }

    #[test]
    fn schema_bounds_rejects_disallowed_tag() {
        let mut config = SchemaBoundsConfig::new(1, 8);
        config.allowed_command_tags = Some(vec![0xA0]);
        let mut authority = SchemaBoundsAuthority::new(config);

        let decision = authority.validate(&ValidationContext {
            connection: ConnectionId(1),
            payload: &[0xB0, 0x01],
            received_at: Instant::now(),
        });

        assert!(matches!(decision, AuthorityDecision::Reject { .. }));
    }

    #[test]
    fn rate_limited_rejects_when_limit_exceeded() {
        let mut authority =
            RateLimitedAuthority::new(RateLimitConfig::new(2, Duration::from_millis(500)));
        let now = Instant::now();

        let d1 = authority.validate(&ValidationContext {
            connection: ConnectionId(1),
            payload: &[1],
            received_at: now,
        });
        let d2 = authority.validate(&ValidationContext {
            connection: ConnectionId(1),
            payload: &[1],
            received_at: now + Duration::from_millis(1),
        });
        let d3 = authority.validate(&ValidationContext {
            connection: ConnectionId(1),
            payload: &[1],
            received_at: now + Duration::from_millis(2),
        });

        assert_eq!(d1, AuthorityDecision::Accept);
        assert_eq!(d2, AuthorityDecision::Accept);
        assert!(matches!(d3, AuthorityDecision::Reject { .. }));
    }
}
