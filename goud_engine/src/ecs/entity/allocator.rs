//! Entity ID allocation with generation counting and free-list recycling.

use std::fmt;

use super::types::Entity;

// =============================================================================
// EntityAllocator
// =============================================================================

/// Manages entity ID allocation with generation counting and free-list recycling.
///
/// The `EntityAllocator` is responsible for creating and invalidating entities
/// in a memory-efficient manner. It uses a generational index scheme where:
///
/// - Each slot has a generation counter that increments on deallocation
/// - Deallocated slots are recycled via a free list
/// - Entities carry their generation, allowing stale entity detection
///
/// # Design Pattern: Generational Arena Allocator
///
/// This pattern provides:
/// - O(1) allocation (pop from free list or push new entry)
/// - O(1) deallocation (push to free list, increment generation)
/// - O(1) liveness check (compare generations)
/// - Memory reuse without use-after-free bugs
///
/// # Difference from HandleAllocator
///
/// Unlike [`HandleAllocator<T>`](crate::core::handle::HandleAllocator), which is
/// generic over a marker type for type safety, `EntityAllocator` is specifically
/// designed for ECS entities and produces non-generic [`Entity`] values.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::entity::EntityAllocator;
///
/// let mut allocator = EntityAllocator::new();
///
/// // Allocate some entities
/// let e1 = allocator.allocate();
/// let e2 = allocator.allocate();
///
/// assert!(allocator.is_alive(e1));
/// assert!(allocator.is_alive(e2));
///
/// // Deallocate e1
/// assert!(allocator.deallocate(e1));
/// assert!(!allocator.is_alive(e1));  // e1 is now stale
///
/// // Allocate again - may reuse e1's slot with new generation
/// let e3 = allocator.allocate();
/// assert!(allocator.is_alive(e3));
///
/// // e1 still references old generation, so it's still not alive
/// assert!(!allocator.is_alive(e1));
/// ```
///
/// # Thread Safety
///
/// `EntityAllocator` is NOT thread-safe. For concurrent access, wrap in
/// appropriate synchronization primitives (Mutex, RwLock, etc.).
pub struct EntityAllocator {
    /// Generation counter for each slot.
    ///
    /// Index `i` stores the current generation for slot `i`.
    /// Generation starts at 1 for new slots (0 is reserved for never-allocated).
    pub(super) generations: Vec<u32>,

    /// Stack of free slot indices available for reuse.
    ///
    /// When an entity is deallocated, its index is pushed here.
    /// On allocation, we prefer to pop from this list before growing.
    pub(super) free_list: Vec<u32>,
}

impl EntityAllocator {
    /// Creates a new, empty entity allocator.
    ///
    /// The allocator starts with no pre-allocated capacity.
    /// Use [`with_capacity`](Self::with_capacity) for bulk allocation scenarios.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let allocator = EntityAllocator::new();
    /// assert_eq!(allocator.len(), 0);
    /// assert!(allocator.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Creates a new entity allocator with pre-allocated capacity.
    ///
    /// This is useful when you know approximately how many entities you'll need,
    /// as it avoids repeated reallocations during bulk allocation.
    ///
    /// Note that this only pre-allocates memory for the internal vectors;
    /// it does not create any entities. Use [`allocate`](Self::allocate) to
    /// actually create entities.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of slots to pre-allocate
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// // Pre-allocate space for 1000 entities
    /// let mut allocator = EntityAllocator::with_capacity(1000);
    ///
    /// // No entities allocated yet, but memory is reserved
    /// assert_eq!(allocator.len(), 0);
    /// assert!(allocator.is_empty());
    ///
    /// // Allocations up to 1000 won't cause reallocation
    /// for _ in 0..1000 {
    ///     allocator.allocate();
    /// }
    /// assert_eq!(allocator.len(), 1000);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            generations: Vec::with_capacity(capacity),
            free_list: Vec::new(), // Free list starts empty, no freed slots yet
        }
    }

    /// Allocates a new entity.
    ///
    /// If there are slots in the free list, one is reused with an incremented
    /// generation. Otherwise, a new slot is created with generation 1.
    ///
    /// # Returns
    ///
    /// A new, valid entity that can be used in ECS operations.
    ///
    /// # Panics
    ///
    /// Panics if the number of slots exceeds `u32::MAX - 1` (unlikely in practice,
    /// as this would require over 4 billion entity allocations without reuse).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    ///
    /// let e1 = allocator.allocate();
    /// let e2 = allocator.allocate();
    ///
    /// assert_ne!(e1, e2);
    /// assert!(!e1.is_placeholder());
    /// assert!(!e2.is_placeholder());
    /// ```
    pub fn allocate(&mut self) -> Entity {
        if let Some(index) = self.free_list.pop() {
            // Reuse a slot from the free list
            // Generation was already incremented during deallocation
            let generation = self.generations[index as usize];
            Entity::new(index, generation)
        } else {
            // Allocate a new slot
            let index = self.generations.len();

            // Ensure we don't exceed u32::MAX - 1 (reserve MAX for PLACEHOLDER)
            assert!(
                index < u32::MAX as usize,
                "EntityAllocator exceeded maximum capacity"
            );

            // New slots start at generation 1 (0 is reserved for never-allocated/PLACEHOLDER)
            self.generations.push(1);

            Entity::new(index as u32, 1)
        }
    }

    /// Deallocates an entity, making it invalid.
    ///
    /// The slot's generation is incremented, invalidating any entity references
    /// that use the old generation. The slot is added to the free list for
    /// future reuse.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to deallocate
    ///
    /// # Returns
    ///
    /// - `true` if the entity was valid and successfully deallocated
    /// - `false` if the entity was invalid (wrong generation, out of bounds, or PLACEHOLDER)
    ///
    /// # Double Deallocation
    ///
    /// Attempting to deallocate the same entity twice returns `false` on the
    /// second attempt, as the generation will have already been incremented.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// let entity = allocator.allocate();
    ///
    /// assert!(allocator.is_alive(entity));
    /// assert!(allocator.deallocate(entity));  // First deallocation succeeds
    /// assert!(!allocator.is_alive(entity));
    /// assert!(!allocator.deallocate(entity)); // Second deallocation fails
    /// ```
    pub fn deallocate(&mut self, entity: Entity) -> bool {
        // PLACEHOLDER entities cannot be deallocated
        if entity.is_placeholder() {
            return false;
        }

        let index = entity.index() as usize;

        // Check bounds
        if index >= self.generations.len() {
            return false;
        }

        // Check generation matches (entity is still alive)
        if self.generations[index] != entity.generation() {
            return false;
        }

        // Increment generation to invalidate existing entity references
        // Wrap at u32::MAX to 1 (skip 0, which is reserved)
        let new_gen = self.generations[index].wrapping_add(1);
        self.generations[index] = if new_gen == 0 { 1 } else { new_gen };

        // Add to free list for reuse
        self.free_list.push(entity.index());

        true
    }

    /// Checks if an entity is currently alive.
    ///
    /// An entity is "alive" if:
    /// - It is not the PLACEHOLDER sentinel
    /// - Its index is within bounds
    /// - Its generation matches the current slot generation
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// `true` if the entity is alive, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// let entity = allocator.allocate();
    ///
    /// assert!(allocator.is_alive(entity));
    ///
    /// allocator.deallocate(entity);
    /// assert!(!allocator.is_alive(entity));
    /// ```
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        // PLACEHOLDER entities are never alive
        if entity.is_placeholder() {
            return false;
        }

        let index = entity.index() as usize;

        // Check bounds and generation
        index < self.generations.len() && self.generations[index] == entity.generation()
    }

    /// Returns the number of currently allocated (alive) entities.
    ///
    /// This is the total capacity minus the number of free slots.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// assert_eq!(allocator.len(), 0);
    ///
    /// let e1 = allocator.allocate();
    /// let e2 = allocator.allocate();
    /// assert_eq!(allocator.len(), 2);
    ///
    /// allocator.deallocate(e1);
    /// assert_eq!(allocator.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.generations.len() - self.free_list.len()
    }

    /// Returns the total number of slots (both allocated and free).
    ///
    /// This represents the high-water mark of allocations.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// let e1 = allocator.allocate();
    /// let e2 = allocator.allocate();
    ///
    /// assert_eq!(allocator.capacity(), 2);
    ///
    /// allocator.deallocate(e1);
    /// // Capacity remains 2, even though one slot is free
    /// assert_eq!(allocator.capacity(), 2);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.generations.len()
    }

    /// Returns `true` if no entities are currently allocated.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// assert!(allocator.is_empty());
    ///
    /// let entity = allocator.allocate();
    /// assert!(!allocator.is_empty());
    ///
    /// allocator.deallocate(entity);
    /// assert!(allocator.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reserves capacity for at least `additional` more entities.
    ///
    /// This pre-allocates memory in the internal generations vector, avoiding
    /// reallocations during subsequent allocations. Use this when you know
    /// approximately how many entities you'll need.
    ///
    /// Note that this only affects the generations vector capacity. It does not
    /// create any entities or affect the free list.
    ///
    /// # Arguments
    ///
    /// * `additional` - The number of additional slots to reserve
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    ///
    /// // Allocate some entities
    /// let initial = allocator.allocate_batch(100);
    /// assert_eq!(allocator.len(), 100);
    ///
    /// // Reserve space for 1000 more
    /// allocator.reserve(1000);
    ///
    /// // Now allocations up to capacity won't cause reallocation
    /// let more = allocator.allocate_batch(1000);
    /// assert_eq!(allocator.len(), 1100);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.generations.reserve(additional);
    }
}

impl Default for EntityAllocator {
    /// Creates an empty entity allocator.
    ///
    /// Equivalent to [`EntityAllocator::new()`].
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for EntityAllocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityAllocator")
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("free_slots", &self.free_list.len())
            .finish()
    }
}
