//! Pool membership marker component.

use crate::ecs::Component;

/// Marker component attached to entities managed by an entity pool.
///
/// This component tracks which pool an entity belongs to, enabling
/// the pool system to identify and manage pooled entities.
#[derive(Debug, Clone, Copy)]
pub struct PoolMember {
    /// The pool handle this entity belongs to.
    pub pool_id: u32,
}

impl PoolMember {
    /// Creates a new pool member marker.
    #[inline]
    pub fn new(pool_id: u32) -> Self {
        Self { pool_id }
    }
}

impl Component for PoolMember {}
