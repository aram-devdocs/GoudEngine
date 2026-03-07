//! Tick-tracking extensions for [`SparseSet`].
//!
//! Provides methods for storing and querying per-entity change ticks
//! alongside component values. Used by the change detection system.

use super::super::Entity;
use super::core::SparseSet;

impl<T> SparseSet<T> {
    /// Inserts a value for the given entity, recording the given `change_tick`.
    ///
    /// On a new insert both `added_tick` and `changed_tick` are set to
    /// `change_tick`. On a replacement only `changed_tick` is updated.
    ///
    /// If the entity already has a value, the old value is returned.
    ///
    /// # Panics
    ///
    /// Panics if `entity` is a placeholder.
    pub fn insert_with_tick(&mut self, entity: Entity, value: T, change_tick: u32) -> Option<T> {
        assert!(
            !entity.is_placeholder(),
            "Cannot insert with placeholder entity"
        );

        let index = entity.index() as usize;

        // Grow sparse vec if needed
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, None);
        }

        if let Some(dense_index) = self.sparse[index] {
            // Entity already has a value - replace it
            let old_value = std::mem::replace(&mut self.values[dense_index], value);
            self.changed_ticks[dense_index] = change_tick;
            Some(old_value)
        } else {
            // New entity - add to dense arrays
            let dense_index = self.dense.len();
            self.sparse[index] = Some(dense_index);
            self.dense.push(entity);
            self.values.push(value);
            self.added_ticks.push(change_tick);
            self.changed_ticks.push(change_tick);
            None
        }
    }

    /// Returns the added tick for the given entity, if present.
    #[inline]
    pub fn get_added_tick(&self, entity: Entity) -> Option<u32> {
        let dense_index = self.dense_index_of(entity)?;
        Some(self.added_ticks[dense_index])
    }

    /// Returns the changed tick for the given entity, if present.
    #[inline]
    pub fn get_changed_tick(&self, entity: Entity) -> Option<u32> {
        let dense_index = self.dense_index_of(entity)?;
        Some(self.changed_ticks[dense_index])
    }

    /// Sets the changed tick for the given entity.
    ///
    /// Does nothing if the entity is not in the set.
    #[inline]
    pub fn set_changed_tick(&mut self, entity: Entity, tick: u32) {
        if let Some(dense_index) = self.dense_index_of(entity) {
            self.changed_ticks[dense_index] = tick;
        }
    }

    /// Internal helper: returns the dense index for an entity, if present.
    #[inline]
    pub(crate) fn dense_index_of(&self, entity: Entity) -> Option<usize> {
        if entity.is_placeholder() {
            return None;
        }
        let index = entity.index() as usize;
        if index >= self.sparse.len() {
            return None;
        }
        self.sparse[index]
    }
}
