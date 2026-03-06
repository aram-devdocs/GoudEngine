//! [`SparseSet`] implementations of the component storage traits.
//!
//! This module provides the concrete implementations of [`ComponentStorage`]
//! and [`AnyComponentStorage`] for [`SparseSet`].

use crate::ecs::{Component, Entity, SparseSet};

use super::traits::{AnyComponentStorage, ComponentStorage};

// =============================================================================
// SparseSet — ComponentStorage Implementation
// =============================================================================

impl<T: Component> ComponentStorage for SparseSet<T> {
    type Item = T;

    #[inline]
    fn insert(&mut self, entity: Entity, value: T) -> Option<T> {
        SparseSet::insert(self, entity, value)
    }

    #[inline]
    fn remove(&mut self, entity: Entity) -> Option<T> {
        SparseSet::remove(self, entity)
    }

    #[inline]
    fn get(&self, entity: Entity) -> Option<&T> {
        SparseSet::get(self, entity)
    }

    #[inline]
    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        SparseSet::get_mut(self, entity)
    }

    #[inline]
    fn contains(&self, entity: Entity) -> bool {
        SparseSet::contains(self, entity)
    }

    #[inline]
    fn len(&self) -> usize {
        SparseSet::len(self)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        SparseSet::is_empty(self)
    }
}

// =============================================================================
// SparseSet — AnyComponentStorage Implementation
// =============================================================================

impl<T: Component> AnyComponentStorage for SparseSet<T> {
    #[inline]
    fn contains_entity(&self, entity: Entity) -> bool {
        self.contains(entity)
    }

    #[inline]
    fn remove_entity(&mut self, entity: Entity) -> bool {
        self.remove(entity).is_some()
    }

    #[inline]
    fn storage_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn storage_is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn clear(&mut self) {
        SparseSet::clear(self)
    }

    #[inline]
    fn component_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
