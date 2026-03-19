//! # FFI Entity Operations
//!
//! This module provides C-compatible functions for entity lifecycle management:
//! spawning, despawning, and querying entity state.
//!
//! ## Design
//!
//! All functions:
//! - Accept `GoudContextId` as the first parameter
//! - Return error codes via thread-local storage
//! - Use `#[no_mangle]` and `extern "C"` for C ABI compatibility
//! - Validate all inputs before accessing the context
//! - Never panic - all errors are caught and converted to error codes
//!
//! ## Thread Safety
//!
//! Entity operations must be called from the thread that owns the context.
//! The context registry is thread-safe, but individual contexts are not.
//!
//! ## Submodules
//!
//! - `lifecycle` - Entity spawn and despawn functions
//! - `queries` - Entity liveness checks and counting

pub mod lifecycle;
pub mod queries;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_batch_alive;

// Re-export all public FFI functions so existing callers see the same API.
pub use lifecycle::{
    goud_entity_clone, goud_entity_clone_recursive, goud_entity_despawn, goud_entity_despawn_batch,
    goud_entity_spawn_batch, goud_entity_spawn_empty,
};
pub use queries::{goud_entity_count, goud_entity_is_alive, goud_entity_is_alive_batch};

use crate::ecs::Entity;
use crate::ffi::GoudEntityId;

/// Sentinel value for an invalid entity ID.
///
/// This is returned by entity spawn functions on failure.
/// Callers should check for this value before using the entity ID.
pub const GOUD_INVALID_ENTITY_ID: u64 = u64::MAX;

/// Converts an FFI `GoudEntityId` to an internal `Entity`.
#[inline]
pub(crate) fn entity_from_ffi(entity_id: GoudEntityId) -> Entity {
    Entity::from_bits(entity_id.bits())
}
