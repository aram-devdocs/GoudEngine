//! Helper functions for component operations.
//!
//! Small utility functions shared by single-entity and batch operations.

use crate::core::context_registry::GoudContextId;
use crate::core::types::GoudEntityId;
use crate::ecs::Entity;

/// Converts an FFI GoudEntityId to an internal Entity.
#[inline]
pub(crate) fn entity_from_ffi(entity_id: GoudEntityId) -> Entity {
    Entity::from_bits(entity_id.bits())
}

/// Packs a context ID into a u64 key for storage maps.
#[inline]
pub(crate) fn context_key(context_id: GoudContextId) -> u64 {
    (context_id.generation() as u64) << 32 | (context_id.index() as u64)
}
