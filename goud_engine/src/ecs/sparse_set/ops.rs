//! Iteration and advanced accessor methods for [`SparseSet`].
//!
//! This module extends `SparseSet<T>` with iteration helpers and
//! lower-level dense-index accessors used by storage layers.

use super::super::Entity;
use super::core::SparseSet;
use super::iter::{SparseSetIter, SparseSetIterMut};

impl<T> SparseSet<T> {
    // =========================================================================
    // Iteration
    // =========================================================================

    /// Returns an iterator over `(Entity, &T)` pairs.
    ///
    /// Iteration is cache-friendly because it traverses the dense array
    /// sequentially.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// set.insert(Entity::new(0, 1), "a");
    /// set.insert(Entity::new(1, 1), "b");
    ///
    /// for (entity, value) in set.iter() {
    ///     println!("{}: {}", entity, value);
    /// }
    /// ```
    #[inline]
    pub fn iter(&self) -> SparseSetIter<'_, T> {
        SparseSetIter {
            dense: self.dense.iter(),
            values: self.values.iter(),
        }
    }

    /// Returns a mutable iterator over `(Entity, &mut T)` pairs.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// set.insert(Entity::new(0, 1), 1);
    /// set.insert(Entity::new(1, 1), 2);
    ///
    /// for (_, value) in set.iter_mut() {
    ///     *value *= 10;
    /// }
    ///
    /// assert_eq!(set.get(Entity::new(0, 1)), Some(&10));
    /// assert_eq!(set.get(Entity::new(1, 1)), Some(&20));
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> SparseSetIterMut<'_, T> {
        SparseSetIterMut {
            dense: self.dense.iter(),
            values: self.values.iter_mut(),
        }
    }

    /// Returns an iterator over all entities in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// set.insert(Entity::new(0, 1), "a");
    /// set.insert(Entity::new(5, 1), "b");
    ///
    /// let entities: Vec<_> = set.entities().collect();
    /// assert_eq!(entities.len(), 2);
    /// ```
    #[inline]
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.dense.iter().copied()
    }

    /// Returns an iterator over all values in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// set.insert(Entity::new(0, 1), 10);
    /// set.insert(Entity::new(5, 1), 20);
    ///
    /// let sum: i32 = set.values().sum();
    /// assert_eq!(sum, 30);
    /// ```
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }

    /// Returns a mutable iterator over all values in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// set.insert(Entity::new(0, 1), 10);
    /// set.insert(Entity::new(1, 1), 20);
    ///
    /// for value in set.values_mut() {
    ///     *value += 1;
    /// }
    ///
    /// let sum: i32 = set.values().sum();
    /// assert_eq!(sum, 32); // 11 + 21
    /// ```
    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.values.iter_mut()
    }

    // =========================================================================
    // Advanced / Internal Methods
    // =========================================================================

    /// Returns the raw entity array for direct access.
    ///
    /// This is useful for bulk operations or implementing custom iterators.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// set.insert(Entity::new(0, 1), "a");
    /// set.insert(Entity::new(1, 1), "b");
    ///
    /// let dense = set.dense();
    /// assert_eq!(dense.len(), 2);
    /// ```
    #[inline]
    pub fn dense(&self) -> &[Entity] {
        &self.dense
    }

    /// Returns the dense index for an entity, if it exists.
    ///
    /// This is an advanced method for implementing custom storage operations.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to look up
    ///
    /// # Returns
    ///
    /// The index in the dense array, or `None` if the entity has no value.
    #[inline]
    pub fn dense_index(&self, entity: Entity) -> Option<usize> {
        if entity.is_placeholder() {
            return None;
        }

        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            return None;
        }

        self.sparse[index]
    }

    /// Returns the value at the given dense index.
    ///
    /// # Safety
    ///
    /// This method does not validate the index. Use `dense_index()` to get
    /// a valid index first.
    ///
    /// # Arguments
    ///
    /// * `dense_index` - Index into the dense/values arrays
    #[inline]
    pub fn get_by_dense_index(&self, dense_index: usize) -> Option<&T> {
        self.values.get(dense_index)
    }

    /// Returns a mutable reference to the value at the given dense index.
    #[inline]
    pub fn get_mut_by_dense_index(&mut self, dense_index: usize) -> Option<&mut T> {
        self.values.get_mut(dense_index)
    }
}
