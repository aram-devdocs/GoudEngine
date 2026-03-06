//! Core sparse set data structure for component storage.
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

use super::super::Entity;

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
    pub(super) sparse: Vec<Option<usize>>,

    /// Packed array of entities that have values.
    /// Enables O(1) removal via swap-remove.
    pub(super) dense: Vec<Entity>,

    /// Packed array of values, parallel to `dense`.
    /// `values[i]` is the value for `dense[i]`.
    pub(super) values: Vec<T>,

    /// Tick at which each component was added, parallel to `dense`/`values`.
    added_ticks: Vec<u32>,

    /// Tick at which each component was last changed, parallel to `dense`/`values`.
    changed_ticks: Vec<u32>,
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
            added_ticks: Vec::new(),
            changed_ticks: Vec::new(),
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
            added_ticks: Vec::with_capacity(capacity),
            changed_ticks: Vec::with_capacity(capacity),
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
        self.insert_with_tick(entity, value, 0)
    }

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
            self.added_ticks.swap(dense_index, last_index);
            self.changed_ticks.swap(dense_index, last_index);

            // Update sparse pointer for swapped entity
            self.sparse[last_entity.index() as usize] = Some(dense_index);
        }

        // Remove last element (which is now our removed entity)
        self.dense.pop();
        self.added_ticks.pop();
        self.changed_ticks.pop();
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
        self.added_ticks.clear();
        self.changed_ticks.clear();
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
        self.added_ticks.reserve(additional);
        self.changed_ticks.reserve(additional);
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
    fn dense_index_of(&self, entity: Entity) -> Option<usize> {
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
            added_ticks: self.added_ticks.clone(),
            changed_ticks: self.changed_ticks.clone(),
        }
    }
}
