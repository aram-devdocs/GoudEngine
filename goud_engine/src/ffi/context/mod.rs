//! # FFI Context Management
//!
//! This module provides FFI entry points for context lifecycle operations.
//! The core context types and registry are defined in `core::context_registry`
//! and re-exported here for backward compatibility.

#![allow(clippy::arc_with_non_send_sync)]

// Re-export all context types from core for backward compatibility
pub use crate::context_registry::{
    get_context_registry, GoudContext, GoudContextHandle, GoudContextId, GoudContextRegistry,
    GOUD_INVALID_CONTEXT_ID,
};

#[cfg(test)]
mod debugger_tests;
mod lifecycle;
#[cfg(test)]
mod tests;

pub use lifecycle::{
    goud_context_create, goud_context_create_with_config, goud_context_destroy,
    goud_context_is_valid, GoudContextConfig, GoudDebuggerConfig,
};
