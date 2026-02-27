//! Sparse set data structure for component storage.
//!
//! A sparse set provides O(1) insertion, deletion, and lookup while maintaining
//! cache-friendly iteration over dense data. This is the foundation for ECS
//! component storage.
//!
//! # Design Pattern: Sparse Set
//!
//! The sparse set uses two arrays:
//!
//! - **Sparse**: Maps entity indices to dense array indices (may have gaps)
//! - **Dense**: Packed array of entities for cache-friendly iteration
//! - **Values**: Packed array of component values, parallel to dense
//!
//! ```text
//! Entity(3) has component "A"
//! Entity(7) has component "B"
//! Entity(1) has component "C"
//!
//! Sparse:  [_, Some(2), _, Some(0), _, _, _, Some(1)]
//!           0    1      2    3      4  5  6    7
//!
//! Dense:   [Entity(3), Entity(7), Entity(1)]
//!              0          1          2
//!
//! Values:  ["A", "B", "C"]
//!            0     1    2
//! ```
//!
//! # Performance Characteristics
//!
//! | Operation | Time Complexity | Notes |
//! |-----------|-----------------|-------|
//! | insert    | O(1) amortized  | May grow sparse vec |
//! | remove    | O(1)            | Uses swap-remove |
//! | get       | O(1)            | |
//! | contains  | O(1)            | |
//! | iterate   | O(n)            | Cache-friendly, contiguous |
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::{Entity, SparseSet};
//!
//! let mut set: SparseSet<String> = SparseSet::new();
//!
//! let e1 = Entity::new(0, 1);
//! let e2 = Entity::new(5, 1);
//!
//! set.insert(e1, "Hello".to_string());
//! set.insert(e2, "World".to_string());
//!
//! assert_eq!(set.get(e1), Some(&"Hello".to_string()));
//! assert!(set.contains(e2));
//!
//! // Cache-friendly iteration
//! for (entity, value) in set.iter() {
//!     println!("{}: {}", entity, value);
//! }
//! ```
//!
//! # Thread Safety
//!
//! `SparseSet<T>` is `Send` if `T: Send` and `Sync` if `T: Sync`.
//! The sparse set itself is not internally synchronized - wrap in
//! `RwLock` or similar for concurrent access.

use super::Entity;

/// A sparse set storing values of type `T` indexed by [`Entity`].
///
/// Provides O(1) access operations while maintaining cache-friendly iteration.
/// This is the primary storage backend for ECS components.
///
/// # Type Parameters
///
/// - `T`: The value type stored in the set
///
/// # Memory Layout
///
/// The sparse set trades memory for performance:
///
/// - Sparse vec grows to max entity index seen (sparse memory usage)
/// - Dense and values vecs are tightly packed (no gaps)
///
/// For entities with indices 0, 100, 1000, the sparse vec will have 1001 entries,
/// but dense/values will only have 3 entries.
#[derive(Debug)]
pub struct SparseSet<T> {
    /// Maps entity index to position in dense array.
    /// `sparse[entity.index()]` = Some(dense_index) if entity has a value.
    sparse: Vec<Option<usize>>,

    /// Packed array of entities that have values.
    /// Enables O(1) removal via swap-remove.
    dense: Vec<Entity>,

    /// Packed array of values, parallel to `dense`.
    /// `values[i]` is the value for `dense[i]`.
    values: Vec<T>,
}

impl<T> SparseSet<T> {
    /// Creates a new, empty sparse set.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::SparseSet;
    ///
    /// let set: SparseSet<i32> = SparseSet::new();
    /// assert!(set.is_empty());
    /// assert_eq!(set.len(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Creates a sparse set with pre-allocated capacity.
    ///
    /// This pre-allocates the dense and values vectors, but not the sparse
    /// vector (which grows on demand based on entity indices).
    ///
    /// # Arguments
    ///
    /// * `capacity` - Number of elements to pre-allocate
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::SparseSet;
    ///
    /// let set: SparseSet<String> = SparseSet::with_capacity(1000);
    /// assert!(set.is_empty());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    /// Inserts a value for the given entity.
    ///
    /// If the entity already has a value, it is replaced and the old value
    /// is returned. Otherwise, `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to associate with the value
    /// * `value` - The value to store
    ///
    /// # Returns
    ///
    /// The previous value if one existed, or `None`.
    ///
    /// # Panics
    ///
    /// Panics if the entity is a placeholder (`Entity::PLACEHOLDER`).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// let entity = Entity::new(0, 1);
    ///
    /// assert_eq!(set.insert(entity, 42), None);
    /// assert_eq!(set.insert(entity, 99), Some(42)); // Returns old value
    /// assert_eq!(set.get(entity), Some(&99));
    /// ```
    pub fn insert(&mut self, entity: Entity, value: T) -> Option<T> {
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
            Some(old_value)
        } else {
            // New entity - add to dense arrays
            let dense_index = self.dense.len();
            self.sparse[index] = Some(dense_index);
            self.dense.push(entity);
            self.values.push(value);
            None
        }
    }

    /// Removes the value for the given entity.
    ///
    /// Uses swap-remove to maintain dense packing: the last element is
    /// moved to fill the gap, keeping iteration cache-friendly.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity whose value to remove
    ///
    /// # Returns
    ///
    /// The removed value if one existed, or `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// let entity = Entity::new(0, 1);
    ///
    /// set.insert(entity, "hello");
    /// assert_eq!(set.remove(entity), Some("hello"));
    /// assert_eq!(set.remove(entity), None); // Already removed
    /// ```
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        if entity.is_placeholder() {
            return None;
        }

        let index = entity.index() as usize;

        // Check bounds
        if index >= self.sparse.len() {
            return None;
        }

        // Get dense index
        let dense_index = self.sparse[index]?;
        self.sparse[index] = None;

        // Get the last entity in the dense array
        let last_index = self.dense.len() - 1;

        if dense_index != last_index {
            // Swap with last element
            let last_entity = self.dense[last_index];
            self.dense.swap(dense_index, last_index);
            self.values.swap(dense_index, last_index);

            // Update sparse pointer for swapped entity
            self.sparse[last_entity.index() as usize] = Some(dense_index);
        }

        // Remove last element (which is now our removed entity)
        self.dense.pop();
        self.values.pop()
    }

    /// Returns a reference to the value for the given entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to look up
    ///
    /// # Returns
    ///
    /// A reference to the value if the entity has one, or `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// let entity = Entity::new(0, 1);
    ///
    /// set.insert(entity, 42);
    /// assert_eq!(set.get(entity), Some(&42));
    ///
    /// let missing = Entity::new(999, 1);
    /// assert_eq!(set.get(missing), None);
    /// ```
    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&T> {
        if entity.is_placeholder() {
            return None;
        }

        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            return None;
        }

        let dense_index = self.sparse[index]?;
        Some(&self.values[dense_index])
    }

    /// Returns a mutable reference to the value for the given entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to look up
    ///
    /// # Returns
    ///
    /// A mutable reference to the value if the entity has one, or `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// let entity = Entity::new(0, 1);
    ///
    /// set.insert(entity, 42);
    ///
    /// if let Some(value) = set.get_mut(entity) {
    ///     *value = 100;
    /// }
    ///
    /// assert_eq!(set.get(entity), Some(&100));
    /// ```
    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if entity.is_placeholder() {
            return None;
        }

        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            return None;
        }

        let dense_index = self.sparse[index]?;
        Some(&mut self.values[dense_index])
    }

    /// Returns `true` if the entity has a value in this set.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// let entity = Entity::new(0, 1);
    ///
    /// assert!(!set.contains(entity));
    /// set.insert(entity, "value");
    /// assert!(set.contains(entity));
    /// ```
    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        if entity.is_placeholder() {
            return false;
        }

        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            return false;
        }

        self.sparse[index].is_some()
    }

    /// Returns the number of entities with values in this set.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// assert_eq!(set.len(), 0);
    ///
    /// set.insert(Entity::new(0, 1), "a");
    /// set.insert(Entity::new(5, 1), "b");
    /// assert_eq!(set.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    /// Returns `true` if the set contains no values.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set: SparseSet<i32> = SparseSet::new();
    /// assert!(set.is_empty());
    ///
    /// set.insert(Entity::new(0, 1), 42);
    /// assert!(!set.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    /// Removes all values from the set.
    ///
    /// This clears all three internal arrays. After calling `clear()`,
    /// `len()` will return 0.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Entity, SparseSet};
    ///
    /// let mut set = SparseSet::new();
    /// set.insert(Entity::new(0, 1), "a");
    /// set.insert(Entity::new(1, 1), "b");
    /// assert_eq!(set.len(), 2);
    ///
    /// set.clear();
    /// assert!(set.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.sparse.clear();
        self.dense.clear();
        self.values.clear();
    }

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// This affects the dense and values arrays, not the sparse array
    /// (which grows based on entity indices).
    ///
    /// # Arguments
    ///
    /// * `additional` - Number of additional elements to reserve
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::SparseSet;
    ///
    /// let mut set: SparseSet<i32> = SparseSet::new();
    /// set.reserve(1000);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.dense.reserve(additional);
        self.values.reserve(additional);
    }

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

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for SparseSet<T> {
    fn clone(&self) -> Self {
        Self {
            sparse: self.sparse.clone(),
            dense: self.dense.clone(),
            values: self.values.clone(),
        }
    }
}

// =============================================================================
// Iterator Types
// =============================================================================

/// An iterator over `(Entity, &T)` pairs in a sparse set.
///
/// Created by [`SparseSet::iter()`].
#[derive(Debug)]
pub struct SparseSetIter<'a, T> {
    dense: std::slice::Iter<'a, Entity>,
    values: std::slice::Iter<'a, T>,
}

impl<'a, T> Iterator for SparseSetIter<'a, T> {
    type Item = (Entity, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let entity = *self.dense.next()?;
        let value = self.values.next()?;
        Some((entity, value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.dense.size_hint()
    }
}

impl<T> ExactSizeIterator for SparseSetIter<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.dense.len()
    }
}

impl<T> std::iter::FusedIterator for SparseSetIter<'_, T> {}

/// A mutable iterator over `(Entity, &mut T)` pairs in a sparse set.
///
/// Created by [`SparseSet::iter_mut()`].
#[derive(Debug)]
pub struct SparseSetIterMut<'a, T> {
    dense: std::slice::Iter<'a, Entity>,
    values: std::slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for SparseSetIterMut<'a, T> {
    type Item = (Entity, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let entity = *self.dense.next()?;
        let value = self.values.next()?;
        Some((entity, value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.dense.size_hint()
    }
}

impl<T> ExactSizeIterator for SparseSetIterMut<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.dense.len()
    }
}

impl<T> std::iter::FusedIterator for SparseSetIterMut<'_, T> {}

// =============================================================================
// IntoIterator Implementations
// =============================================================================

impl<'a, T> IntoIterator for &'a SparseSet<T> {
    type Item = (Entity, &'a T);
    type IntoIter = SparseSetIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SparseSet<T> {
    type Item = (Entity, &'a mut T);
    type IntoIter = SparseSetIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Construction Tests
    // =========================================================================

    #[test]
    fn test_new() {
        let set: SparseSet<i32> = SparseSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let set: SparseSet<i32> = SparseSet::with_capacity(100);
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_default() {
        let set: SparseSet<i32> = SparseSet::default();
        assert!(set.is_empty());
    }

    // =========================================================================
    // Insert Tests
    // =========================================================================

    #[test]
    fn test_insert_single() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        let old = set.insert(entity, 42);
        assert_eq!(old, None);
        assert_eq!(set.len(), 1);
        assert!(set.contains(entity));
        assert_eq!(set.get(entity), Some(&42));
    }

    #[test]
    fn test_insert_multiple() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(5, 1);
        let e3 = Entity::new(10, 1);

        set.insert(e1, "a");
        set.insert(e2, "b");
        set.insert(e3, "c");

        assert_eq!(set.len(), 3);
        assert_eq!(set.get(e1), Some(&"a"));
        assert_eq!(set.get(e2), Some(&"b"));
        assert_eq!(set.get(e3), Some(&"c"));
    }

    #[test]
    fn test_insert_replace() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        let old1 = set.insert(entity, 10);
        assert_eq!(old1, None);

        let old2 = set.insert(entity, 20);
        assert_eq!(old2, Some(10));

        assert_eq!(set.len(), 1); // Still just one entry
        assert_eq!(set.get(entity), Some(&20));
    }

    #[test]
    #[should_panic(expected = "Cannot insert with placeholder")]
    fn test_insert_placeholder_panics() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(Entity::PLACEHOLDER, 42);
    }

    #[test]
    fn test_insert_sparse_indices() {
        // Test with widely spread entity indices
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1000, 1);
        let e3 = Entity::new(5000, 1);

        set.insert(e1, 1);
        set.insert(e2, 2);
        set.insert(e3, 3);

        assert_eq!(set.len(), 3);
        assert_eq!(set.get(e1), Some(&1));
        assert_eq!(set.get(e2), Some(&2));
        assert_eq!(set.get(e3), Some(&3));
    }

    // =========================================================================
    // Remove Tests
    // =========================================================================

    #[test]
    fn test_remove_single() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);
        let removed = set.remove(entity);

        assert_eq!(removed, Some(42));
        assert!(set.is_empty());
        assert!(!set.contains(entity));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut set: SparseSet<i32> = SparseSet::new();
        let entity = Entity::new(0, 1);

        let removed = set.remove(entity);
        assert_eq!(removed, None);
    }

    #[test]
    fn test_remove_placeholder() {
        let mut set: SparseSet<i32> = SparseSet::new();
        let removed = set.remove(Entity::PLACEHOLDER);
        assert_eq!(removed, None);
    }

    #[test]
    fn test_remove_double() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);

        let first = set.remove(entity);
        let second = set.remove(entity);

        assert_eq!(first, Some(42));
        assert_eq!(second, None);
    }

    #[test]
    fn test_remove_swap_correctness() {
        // Test that swap-remove maintains correctness
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        set.insert(e1, "first");
        set.insert(e2, "second");
        set.insert(e3, "third");

        // Remove e1 (first) - e3 should be swapped in
        set.remove(e1);

        // e2 and e3 should still be accessible
        assert_eq!(set.get(e2), Some(&"second"));
        assert_eq!(set.get(e3), Some(&"third"));
        assert_eq!(set.len(), 2);

        // Dense array should have e3 at index 0, e2 at index 1
        let entities: Vec<_> = set.entities().collect();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
    }

    #[test]
    fn test_remove_middle() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        set.insert(e1, 1);
        set.insert(e2, 2);
        set.insert(e3, 3);

        // Remove middle element
        set.remove(e2);

        assert_eq!(set.get(e1), Some(&1));
        assert_eq!(set.get(e2), None);
        assert_eq!(set.get(e3), Some(&3));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_remove_last() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, 1);
        set.insert(e2, 2);

        // Remove last element (no swap needed)
        set.remove(e2);

        assert_eq!(set.get(e1), Some(&1));
        assert_eq!(set.get(e2), None);
        assert_eq!(set.len(), 1);
    }

    // =========================================================================
    // Get Tests
    // =========================================================================

    #[test]
    fn test_get() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);

        assert_eq!(set.get(entity), Some(&42));
    }

    #[test]
    fn test_get_nonexistent() {
        let set: SparseSet<i32> = SparseSet::new();
        let entity = Entity::new(0, 1);

        assert_eq!(set.get(entity), None);
    }

    #[test]
    fn test_get_placeholder() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 42);

        assert_eq!(set.get(Entity::PLACEHOLDER), None);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 42);

        // Entity index beyond sparse array
        let entity = Entity::new(1000, 1);
        assert_eq!(set.get(entity), None);
    }

    #[test]
    fn test_get_mut() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);

        if let Some(value) = set.get_mut(entity) {
            *value = 100;
        }

        assert_eq!(set.get(entity), Some(&100));
    }

    #[test]
    fn test_get_mut_nonexistent() {
        let mut set: SparseSet<i32> = SparseSet::new();
        let entity = Entity::new(0, 1);

        assert_eq!(set.get_mut(entity), None);
    }

    // =========================================================================
    // Contains Tests
    // =========================================================================

    #[test]
    fn test_contains() {
        let mut set = SparseSet::new();
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, 42);

        assert!(set.contains(e1));
        assert!(!set.contains(e2));
    }

    #[test]
    fn test_contains_placeholder() {
        let set: SparseSet<i32> = SparseSet::new();
        assert!(!set.contains(Entity::PLACEHOLDER));
    }

    #[test]
    fn test_contains_after_remove() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);
        assert!(set.contains(entity));

        set.remove(entity);
        assert!(!set.contains(entity));
    }

    // =========================================================================
    // Len / IsEmpty Tests
    // =========================================================================

    #[test]
    fn test_len_empty() {
        let set: SparseSet<i32> = SparseSet::new();
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_len_after_insert() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 1);
        assert_eq!(set.len(), 1);

        set.insert(Entity::new(1, 1), 2);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_len_after_remove() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, 1);
        set.insert(e2, 2);
        assert_eq!(set.len(), 2);

        set.remove(e1);
        assert_eq!(set.len(), 1);

        set.remove(e2);
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_len_after_replace() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 1);
        assert_eq!(set.len(), 1);

        set.insert(entity, 2); // Replace
        assert_eq!(set.len(), 1); // Still 1
    }

    // =========================================================================
    // Clear Tests
    // =========================================================================

    #[test]
    fn test_clear() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 1);
        set.insert(Entity::new(1, 1), 2);
        set.insert(Entity::new(2, 1), 3);

        assert_eq!(set.len(), 3);

        set.clear();

        assert!(set.is_empty());
        assert_eq!(set.get(Entity::new(0, 1)), None);
        assert_eq!(set.get(Entity::new(1, 1)), None);
        assert_eq!(set.get(Entity::new(2, 1)), None);
    }

    // =========================================================================
    // Iteration Tests
    // =========================================================================

    #[test]
    fn test_iter() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        set.insert(e1, 10);
        set.insert(e2, 20);
        set.insert(e3, 30);

        let mut items: Vec<_> = set.iter().map(|(e, v)| (e, *v)).collect();
        items.sort_by_key(|(e, _)| e.index());

        assert_eq!(items, vec![(e1, 10), (e2, 20), (e3, 30)]);
    }

    #[test]
    fn test_iter_empty() {
        let set: SparseSet<i32> = SparseSet::new();
        let items: Vec<_> = set.iter().collect();
        assert!(items.is_empty());
    }

    #[test]
    fn test_iter_mut() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 1);
        set.insert(Entity::new(1, 1), 2);
        set.insert(Entity::new(2, 1), 3);

        for (_, value) in set.iter_mut() {
            *value *= 10;
        }

        assert_eq!(set.get(Entity::new(0, 1)), Some(&10));
        assert_eq!(set.get(Entity::new(1, 1)), Some(&20));
        assert_eq!(set.get(Entity::new(2, 1)), Some(&30));
    }

    #[test]
    fn test_entities() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(5, 1);
        let e3 = Entity::new(10, 1);

        set.insert(e1, "a");
        set.insert(e2, "b");
        set.insert(e3, "c");

        let entities: Vec<_> = set.entities().collect();
        assert_eq!(entities.len(), 3);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
    }

    #[test]
    fn test_values() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 10);
        set.insert(Entity::new(1, 1), 20);
        set.insert(Entity::new(2, 1), 30);

        let sum: i32 = set.values().sum();
        assert_eq!(sum, 60);
    }

    #[test]
    fn test_values_mut() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 1);
        set.insert(Entity::new(1, 1), 2);
        set.insert(Entity::new(2, 1), 3);

        for value in set.values_mut() {
            *value += 10;
        }

        let sum: i32 = set.values().sum();
        assert_eq!(sum, 36); // 11 + 12 + 13
    }

    #[test]
    fn test_into_iter_ref() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 42);

        let mut count = 0;
        for (_, _) in &set {
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn test_into_iter_mut() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 42);

        for (_, value) in &mut set {
            *value = 100;
        }

        assert_eq!(set.get(Entity::new(0, 1)), Some(&100));
    }

    #[test]
    fn test_iter_size_hint() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 1);
        set.insert(Entity::new(1, 1), 2);
        set.insert(Entity::new(2, 1), 3);

        let iter = set.iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.len(), 3);
    }

    // =========================================================================
    // Advanced Method Tests
    // =========================================================================

    #[test]
    fn test_dense() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, "a");
        set.insert(e2, "b");

        let dense = set.dense();
        assert_eq!(dense.len(), 2);
        assert_eq!(dense[0], e1);
        assert_eq!(dense[1], e2);
    }

    #[test]
    fn test_dense_index() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(5, 1);
        let e2 = Entity::new(10, 1);

        set.insert(e1, "a");
        set.insert(e2, "b");

        assert_eq!(set.dense_index(e1), Some(0));
        assert_eq!(set.dense_index(e2), Some(1));
        assert_eq!(set.dense_index(Entity::new(0, 1)), None);
        assert_eq!(set.dense_index(Entity::PLACEHOLDER), None);
    }

    #[test]
    fn test_get_by_dense_index() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), "first");
        set.insert(Entity::new(1, 1), "second");

        assert_eq!(set.get_by_dense_index(0), Some(&"first"));
        assert_eq!(set.get_by_dense_index(1), Some(&"second"));
        assert_eq!(set.get_by_dense_index(2), None);
    }

    #[test]
    fn test_get_mut_by_dense_index() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 10);
        set.insert(Entity::new(1, 1), 20);

        if let Some(value) = set.get_mut_by_dense_index(0) {
            *value = 100;
        }

        assert_eq!(set.get(Entity::new(0, 1)), Some(&100));
    }

    // =========================================================================
    // Clone Tests
    // =========================================================================

    #[test]
    fn test_clone() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, 10);
        set.insert(e2, 20);

        let cloned = set.clone();

        assert_eq!(cloned.len(), 2);
        assert_eq!(cloned.get(e1), Some(&10));
        assert_eq!(cloned.get(e2), Some(&20));

        // Modifications to original don't affect clone
        set.insert(e1, 100);
        assert_eq!(cloned.get(e1), Some(&10));
    }

    // =========================================================================
    // Stress Tests
    // =========================================================================

    #[test]
    fn test_stress_many_entities() {
        let mut set = SparseSet::new();
        const COUNT: usize = 10_000;

        // Insert many entities
        for i in 0..COUNT {
            let entity = Entity::new(i as u32, 1);
            set.insert(entity, i as i32);
        }

        assert_eq!(set.len(), COUNT);

        // Verify all accessible
        for i in 0..COUNT {
            let entity = Entity::new(i as u32, 1);
            assert_eq!(set.get(entity), Some(&(i as i32)));
        }

        // Remove half
        for i in (0..COUNT).step_by(2) {
            let entity = Entity::new(i as u32, 1);
            set.remove(entity);
        }

        assert_eq!(set.len(), COUNT / 2);

        // Verify removed vs remaining
        for i in 0..COUNT {
            let entity = Entity::new(i as u32, 1);
            if i % 2 == 0 {
                assert_eq!(set.get(entity), None);
            } else {
                assert_eq!(set.get(entity), Some(&(i as i32)));
            }
        }
    }

    #[test]
    fn test_stress_sparse_indices() {
        let mut set = SparseSet::new();

        // Insert with very sparse indices
        let indices: Vec<u32> = vec![0, 100, 1000, 10000, 50000];

        for (i, &idx) in indices.iter().enumerate() {
            let entity = Entity::new(idx, 1);
            set.insert(entity, i as i32);
        }

        assert_eq!(set.len(), 5);

        // Verify all accessible
        for (i, &idx) in indices.iter().enumerate() {
            let entity = Entity::new(idx, 1);
            assert_eq!(set.get(entity), Some(&(i as i32)));
        }
    }

    #[test]
    fn test_stress_insert_remove_cycle() {
        let mut set = SparseSet::new();
        const ITERATIONS: usize = 100;
        const BATCH_SIZE: usize = 100;

        for _ in 0..ITERATIONS {
            // Insert batch
            for i in 0..BATCH_SIZE {
                let entity = Entity::new(i as u32, 1);
                set.insert(entity, i as i32);
            }

            assert_eq!(set.len(), BATCH_SIZE);

            // Remove all
            for i in 0..BATCH_SIZE {
                let entity = Entity::new(i as u32, 1);
                set.remove(entity);
            }

            assert!(set.is_empty());
        }
    }

    // =========================================================================
    // Thread Safety Tests (Compile-time)
    // =========================================================================

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        // SparseSet<T> is Send if T is Send
        assert_send::<SparseSet<i32>>();
        assert_send::<SparseSet<String>>();

        // SparseSet<T> is Sync if T is Sync
        assert_sync::<SparseSet<i32>>();
        assert_sync::<SparseSet<String>>();
    }
}
