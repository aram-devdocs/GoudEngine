use serde::{Deserialize, Serialize};

/// Canonical debugger enablement settings shared across engine init surfaces.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebuggerConfig {
    /// Enables the process-local debugger runtime for this context or game.
    pub enabled: bool,
    /// Publishes local attach metadata when at least one route is attachable.
    pub publish_local_attach: bool,
    /// Optional human-friendly label for tooling surfaces.
    pub route_label: Option<String>,
}

/// Configuration used by the config-based context creation path.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Debugger runtime settings for this context.
    pub debugger: DebuggerConfig,
}
