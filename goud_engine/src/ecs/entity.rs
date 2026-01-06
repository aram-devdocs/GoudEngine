//! Entity identifiers for the ECS.
//!
//! Entities are lightweight identifiers that serve as keys for component data.
//! Unlike [`Handle<T>`](crate::core::handle::Handle), entities are not generic -
//! they identify an entity across all component storages rather than a specific
//! resource type.
//!
//! # Design Pattern: Generational Indices
//!
//! Entities use the same generational index pattern as handles:
//!
//! 1. Each entity has an index (slot in the entity array) and a generation
//! 2. When an entity is despawned, its generation increments
//! 3. Old entity references become stale (generation mismatch)
//! 4. The slot can be reused for new entities with the new generation
//!
//! This prevents "dangling entity" bugs where code holds a reference to a
//! despawned entity and accidentally accesses data from a new entity that
//! reused the same slot.
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::entity::Entity;
//!
//! // Entities are typically created by EntityAllocator, but can be constructed directly
//! let entity = Entity::new(0, 1);
//!
//! assert_eq!(entity.index(), 0);
//! assert_eq!(entity.generation(), 1);
//!
//! // PLACEHOLDER is a special sentinel value
//! assert!(Entity::PLACEHOLDER.is_placeholder());
//! ```
//!
//! # FFI Safety
//!
//! Entity uses `#[repr(C)]` for predictable memory layout across FFI boundaries.
//! The struct is exactly 8 bytes: 4 bytes for index + 4 bytes for generation.

use std::fmt;
use std::hash::{Hash, Hasher};

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
    generations: Vec<u32>,

    /// Stack of free slot indices available for reuse.
    ///
    /// When an entity is deallocated, its index is pushed here.
    /// On allocation, we prefer to pop from this list before growing.
    free_list: Vec<u32>,
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

    // =========================================================================
    // Bulk Operations
    // =========================================================================

    /// Allocates multiple entities at once.
    ///
    /// This is more efficient than calling [`allocate`](Self::allocate) in a loop
    /// because it pre-allocates the result vector and minimizes reallocations.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of entities to allocate
    ///
    /// # Returns
    ///
    /// A vector containing `count` newly allocated entities. All entities are
    /// guaranteed to be valid and unique.
    ///
    /// # Panics
    ///
    /// Panics if allocating `count` entities would exceed `u32::MAX - 1` total slots.
    ///
    /// # Performance
    ///
    /// For large batch allocations, this method:
    /// - Pre-allocates the result vector with exact capacity
    /// - Reuses free slots first (LIFO order)
    /// - Bulk-extends the generations vector for remaining slots
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    ///
    /// // Allocate 1000 entities in one call
    /// let entities = allocator.allocate_batch(1000);
    /// assert_eq!(entities.len(), 1000);
    /// assert_eq!(allocator.len(), 1000);
    ///
    /// // All entities are unique and alive
    /// for entity in &entities {
    ///     assert!(allocator.is_alive(*entity));
    /// }
    /// ```
    pub fn allocate_batch(&mut self, count: usize) -> Vec<Entity> {
        if count == 0 {
            return Vec::new();
        }

        let mut entities = Vec::with_capacity(count);

        // First, use up free slots
        let from_free_list = count.min(self.free_list.len());
        for _ in 0..from_free_list {
            if let Some(index) = self.free_list.pop() {
                let generation = self.generations[index as usize];
                entities.push(Entity::new(index, generation));
            }
        }

        // Then, allocate new slots for remaining count
        let remaining = count - from_free_list;
        if remaining > 0 {
            let start_index = self.generations.len();
            let end_index = start_index + remaining;

            // Ensure we don't exceed maximum capacity
            assert!(
                end_index <= u32::MAX as usize,
                "EntityAllocator would exceed maximum capacity"
            );

            // Bulk-extend generations vector with initial generation of 1
            self.generations.resize(end_index, 1);

            // Create entities for new slots
            for index in start_index..end_index {
                entities.push(Entity::new(index as u32, 1));
            }
        }

        entities
    }

    /// Deallocates multiple entities at once.
    ///
    /// This method attempts to deallocate each entity in the slice. Invalid
    /// entities (already deallocated, wrong generation, out of bounds, or
    /// PLACEHOLDER) are skipped without error.
    ///
    /// # Arguments
    ///
    /// * `entities` - A slice of entities to deallocate
    ///
    /// # Returns
    ///
    /// The number of entities that were successfully deallocated.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// let entities = allocator.allocate_batch(100);
    /// assert_eq!(allocator.len(), 100);
    ///
    /// // Deallocate all at once
    /// let deallocated = allocator.deallocate_batch(&entities);
    /// assert_eq!(deallocated, 100);
    /// assert_eq!(allocator.len(), 0);
    ///
    /// // Second deallocation returns 0 (already dead)
    /// let deallocated_again = allocator.deallocate_batch(&entities);
    /// assert_eq!(deallocated_again, 0);
    /// ```
    pub fn deallocate_batch(&mut self, entities: &[Entity]) -> usize {
        let mut success_count = 0;

        for entity in entities {
            if self.deallocate(*entity) {
                success_count += 1;
            }
        }

        success_count
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

// =============================================================================
// Entity
// =============================================================================

/// A lightweight identifier for an entity in the ECS.
///
/// Entities are the "E" in ECS - they are identifiers that components attach to.
/// An entity by itself has no data; it's purely an ID used to look up components.
///
/// # Memory Layout
///
/// ```text
/// Entity (8 bytes total):
/// ┌────────────────┬────────────────┐
/// │  index (u32)   │ generation(u32)│
/// └────────────────┴────────────────┘
/// ```
///
/// # Thread Safety
///
/// Entity is `Copy`, `Clone`, `Send`, and `Sync`. Entity values can be freely
/// shared across threads. However, operations on the ECS world that use entities
/// require appropriate synchronization.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    /// The index of this entity in the entity array.
    ///
    /// This corresponds to a slot in the entity allocator.
    index: u32,

    /// The generation of this entity.
    ///
    /// Incremented each time a slot is reused, allowing detection of stale
    /// entity references.
    generation: u32,
}

impl Entity {
    /// A placeholder entity that should never be returned by the allocator.
    ///
    /// Use this as a sentinel value for "no entity" or uninitialized entity fields.
    /// The placeholder uses `u32::MAX` for the index, which the allocator will never
    /// return (it would require 4 billion+ entity allocations first).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::Entity;
    ///
    /// let mut maybe_entity = Entity::PLACEHOLDER;
    /// assert!(maybe_entity.is_placeholder());
    ///
    /// // Later, assign a real entity
    /// maybe_entity = Entity::new(0, 1);
    /// assert!(!maybe_entity.is_placeholder());
    /// ```
    pub const PLACEHOLDER: Entity = Entity {
        index: u32::MAX,
        generation: 0,
    };

    /// Creates a new entity with the given index and generation.
    ///
    /// This is primarily used by [`EntityAllocator`]. Direct construction is
    /// possible but not recommended for typical use.
    ///
    /// # Arguments
    ///
    /// * `index` - The slot index for this entity
    /// * `generation` - The generation counter for this slot
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::Entity;
    ///
    /// let entity = Entity::new(42, 1);
    /// assert_eq!(entity.index(), 42);
    /// assert_eq!(entity.generation(), 1);
    /// ```
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Returns the index of this entity.
    ///
    /// The index is the slot in the entity allocator's internal array.
    /// Multiple entities may share the same index (at different times),
    /// distinguished by their generation.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation of this entity.
    ///
    /// The generation increments each time a slot is reused. Comparing generations
    /// allows detecting stale entity references.
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns `true` if this is the placeholder entity.
    ///
    /// The placeholder entity should never be used in actual ECS operations.
    /// It's a sentinel value for "no entity" situations.
    #[inline]
    pub const fn is_placeholder(&self) -> bool {
        self.index == u32::MAX && self.generation == 0
    }

    /// Packs the entity into a single `u64` value.
    ///
    /// Format: upper 32 bits = generation, lower 32 bits = index.
    ///
    /// This is useful for FFI or when a single integer representation is needed.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::Entity;
    ///
    /// let entity = Entity::new(42, 7);
    /// let packed = entity.to_bits();
    ///
    /// // Upper 32 bits: generation (7), Lower 32 bits: index (42)
    /// assert_eq!(packed, (7_u64 << 32) | 42);
    ///
    /// // Round-trip
    /// let unpacked = Entity::from_bits(packed);
    /// assert_eq!(entity, unpacked);
    /// ```
    #[inline]
    pub const fn to_bits(&self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }

    /// Creates an entity from a packed `u64` value.
    ///
    /// Format: upper 32 bits = generation, lower 32 bits = index.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::Entity;
    ///
    /// let packed = (3_u64 << 32) | 100;
    /// let entity = Entity::from_bits(packed);
    ///
    /// assert_eq!(entity.index(), 100);
    /// assert_eq!(entity.generation(), 3);
    /// ```
    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        Self {
            index: bits as u32,
            generation: (bits >> 32) as u32,
        }
    }
}

// =============================================================================
// Trait Implementations
// =============================================================================

impl Hash for Entity {
    /// Hashes the entity by combining index and generation.
    ///
    /// Two entities with the same index but different generations will hash
    /// differently, which is important for using entities as map keys.
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash as u64 for efficiency (single hash operation)
        self.to_bits().hash(state);
    }
}

impl fmt::Debug for Entity {
    /// Formats the entity as `Entity(index:generation)`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::Entity;
    ///
    /// let entity = Entity::new(42, 3);
    /// assert_eq!(format!("{:?}", entity), "Entity(42:3)");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}:{})", self.index, self.generation)
    }
}

impl fmt::Display for Entity {
    /// Formats the entity in a user-friendly way.
    ///
    /// Same format as Debug: `Entity(index:generation)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}:{})", self.index, self.generation)
    }
}

impl Default for Entity {
    /// Returns the placeholder entity.
    ///
    /// Using Default returns PLACEHOLDER, which should be treated as "no entity".
    #[inline]
    fn default() -> Self {
        Self::PLACEHOLDER
    }
}

impl From<Entity> for u64 {
    /// Converts an entity to its packed `u64` representation.
    #[inline]
    fn from(entity: Entity) -> Self {
        entity.to_bits()
    }
}

impl From<u64> for Entity {
    /// Creates an entity from a packed `u64` representation.
    #[inline]
    fn from(bits: u64) -> Self {
        Entity::from_bits(bits)
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // -------------------------------------------------------------------------
    // Structure Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_entity_new() {
        let entity = Entity::new(42, 7);
        assert_eq!(entity.index(), 42);
        assert_eq!(entity.generation(), 7);
    }

    #[test]
    fn test_entity_placeholder() {
        let placeholder = Entity::PLACEHOLDER;
        assert_eq!(placeholder.index(), u32::MAX);
        assert_eq!(placeholder.generation(), 0);
        assert!(placeholder.is_placeholder());

        // Non-placeholder entities
        let entity = Entity::new(0, 0);
        assert!(!entity.is_placeholder());

        let entity = Entity::new(u32::MAX, 1);
        assert!(!entity.is_placeholder());

        let entity = Entity::new(0, 1);
        assert!(!entity.is_placeholder());
    }

    #[test]
    fn test_entity_size() {
        // Entity should be exactly 8 bytes (2 x u32)
        assert_eq!(std::mem::size_of::<Entity>(), 8);
        assert_eq!(std::mem::align_of::<Entity>(), 4);
    }

    #[test]
    fn test_entity_copy_clone() {
        let entity1 = Entity::new(10, 5);
        let entity2 = entity1; // Copy
        let entity3 = entity1.clone(); // Clone

        assert_eq!(entity1, entity2);
        assert_eq!(entity1, entity3);
    }

    #[test]
    fn test_entity_equality() {
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(0, 1);
        let e3 = Entity::new(0, 2); // Different generation
        let e4 = Entity::new(1, 1); // Different index

        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
        assert_ne!(e1, e4);
    }

    // -------------------------------------------------------------------------
    // Bit Packing Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_entity_to_bits() {
        let entity = Entity::new(42, 7);
        let bits = entity.to_bits();

        // Upper 32 bits: generation (7), Lower 32 bits: index (42)
        assert_eq!(bits, (7_u64 << 32) | 42);
    }

    #[test]
    fn test_entity_from_bits() {
        let bits = (3_u64 << 32) | 100;
        let entity = Entity::from_bits(bits);

        assert_eq!(entity.index(), 100);
        assert_eq!(entity.generation(), 3);
    }

    #[test]
    fn test_entity_bits_roundtrip() {
        let original = Entity::new(999, 42);
        let bits = original.to_bits();
        let restored = Entity::from_bits(bits);

        assert_eq!(original, restored);
    }

    #[test]
    fn test_entity_bits_edge_cases() {
        // Maximum values
        let max = Entity::new(u32::MAX, u32::MAX);
        assert_eq!(max, Entity::from_bits(max.to_bits()));

        // Zero values
        let zero = Entity::new(0, 0);
        assert_eq!(zero, Entity::from_bits(zero.to_bits()));

        // Mixed
        let mixed = Entity::new(u32::MAX, 0);
        assert_eq!(mixed, Entity::from_bits(mixed.to_bits()));
    }

    // -------------------------------------------------------------------------
    // Trait Implementation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_entity_hash() {
        let mut set = HashSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(0, 1);
        let e3 = Entity::new(0, 2);
        let e4 = Entity::new(1, 1);

        set.insert(e1);

        // Same entity should be found
        assert!(set.contains(&e2));

        // Different entities should not be found
        assert!(!set.contains(&e3));
        assert!(!set.contains(&e4));
    }

    #[test]
    fn test_entity_debug_format() {
        let entity = Entity::new(42, 3);
        assert_eq!(format!("{:?}", entity), "Entity(42:3)");

        let placeholder = Entity::PLACEHOLDER;
        assert_eq!(format!("{:?}", placeholder), "Entity(4294967295:0)");
    }

    #[test]
    fn test_entity_display_format() {
        let entity = Entity::new(100, 7);
        assert_eq!(format!("{}", entity), "Entity(100:7)");
    }

    #[test]
    fn test_entity_default() {
        let default_entity: Entity = Default::default();
        assert_eq!(default_entity, Entity::PLACEHOLDER);
        assert!(default_entity.is_placeholder());
    }

    // -------------------------------------------------------------------------
    // Conversion Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_entity_from_into_u64() {
        let entity = Entity::new(42, 7);

        // Into<u64>
        let bits: u64 = entity.into();
        assert_eq!(bits, entity.to_bits());

        // From<u64>
        let restored: Entity = bits.into();
        assert_eq!(restored, entity);
    }

    // -------------------------------------------------------------------------
    // Thread Safety Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_entity_send_sync() {
        // Compile-time check that Entity is Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Entity>();
    }

    // =========================================================================
    // EntityAllocator Tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // Basic Allocator Operations
    // -------------------------------------------------------------------------

    #[test]
    fn test_allocator_new() {
        let allocator = EntityAllocator::new();
        assert_eq!(allocator.len(), 0);
        assert_eq!(allocator.capacity(), 0);
        assert!(allocator.is_empty());
    }

    #[test]
    fn test_allocator_with_capacity() {
        let allocator = EntityAllocator::with_capacity(100);
        assert_eq!(allocator.len(), 0);
        assert_eq!(allocator.capacity(), 0); // capacity() returns slots used, not reserved
        assert!(allocator.is_empty());
    }

    #[test]
    fn test_allocator_default() {
        let allocator: EntityAllocator = Default::default();
        assert_eq!(allocator.len(), 0);
        assert!(allocator.is_empty());
    }

    #[test]
    fn test_allocator_debug() {
        let mut allocator = EntityAllocator::new();
        let _e1 = allocator.allocate();
        let e2 = allocator.allocate();
        allocator.deallocate(e2);

        let debug_str = format!("{:?}", allocator);
        assert!(debug_str.contains("EntityAllocator"));
        assert!(debug_str.contains("len"));
        assert!(debug_str.contains("capacity"));
        assert!(debug_str.contains("free_slots"));
    }

    // -------------------------------------------------------------------------
    // Allocation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_allocator_allocate_basic() {
        let mut allocator = EntityAllocator::new();

        let e1 = allocator.allocate();
        assert_eq!(e1.index(), 0);
        assert_eq!(e1.generation(), 1);
        assert!(!e1.is_placeholder());

        let e2 = allocator.allocate();
        assert_eq!(e2.index(), 1);
        assert_eq!(e2.generation(), 1);

        assert_ne!(e1, e2);
        assert_eq!(allocator.len(), 2);
    }

    #[test]
    fn test_allocator_allocate_multiple() {
        let mut allocator = EntityAllocator::new();
        let mut entities = Vec::new();

        for _ in 0..100 {
            entities.push(allocator.allocate());
        }

        assert_eq!(allocator.len(), 100);
        assert_eq!(allocator.capacity(), 100);

        // All entities should be unique
        let unique: HashSet<_> = entities.iter().collect();
        assert_eq!(unique.len(), 100);

        // All entities should be alive
        for entity in &entities {
            assert!(allocator.is_alive(*entity));
        }
    }

    #[test]
    fn test_allocator_first_generation_is_one() {
        let mut allocator = EntityAllocator::new();

        // All newly allocated entities should have generation 1
        for _ in 0..10 {
            let entity = allocator.allocate();
            assert_eq!(entity.generation(), 1);
        }
    }

    // -------------------------------------------------------------------------
    // Deallocation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_allocator_deallocate_basic() {
        let mut allocator = EntityAllocator::new();
        let entity = allocator.allocate();

        assert!(allocator.is_alive(entity));
        assert!(allocator.deallocate(entity));
        assert!(!allocator.is_alive(entity));
    }

    #[test]
    fn test_allocator_deallocate_returns_false_for_dead_entity() {
        let mut allocator = EntityAllocator::new();
        let entity = allocator.allocate();

        // First deallocation succeeds
        assert!(allocator.deallocate(entity));

        // Second deallocation fails
        assert!(!allocator.deallocate(entity));
    }

    #[test]
    fn test_allocator_deallocate_returns_false_for_placeholder() {
        let mut allocator = EntityAllocator::new();

        // PLACEHOLDER cannot be deallocated
        assert!(!allocator.deallocate(Entity::PLACEHOLDER));
    }

    #[test]
    fn test_allocator_deallocate_returns_false_for_out_of_bounds() {
        let mut allocator = EntityAllocator::new();
        allocator.allocate(); // Allocate slot 0

        // Entity with index 999 is out of bounds
        let fake_entity = Entity::new(999, 1);
        assert!(!allocator.deallocate(fake_entity));
    }

    #[test]
    fn test_allocator_deallocate_returns_false_for_wrong_generation() {
        let mut allocator = EntityAllocator::new();
        let entity = allocator.allocate();

        // Create a fake entity with same index but different generation
        let fake_entity = Entity::new(entity.index(), entity.generation() + 1);
        assert!(!allocator.deallocate(fake_entity));

        // Original entity is still alive
        assert!(allocator.is_alive(entity));
    }

    // -------------------------------------------------------------------------
    // is_alive Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_allocator_is_alive() {
        let mut allocator = EntityAllocator::new();
        let entity = allocator.allocate();

        assert!(allocator.is_alive(entity));

        allocator.deallocate(entity);
        assert!(!allocator.is_alive(entity));
    }

    #[test]
    fn test_allocator_is_alive_placeholder() {
        let allocator = EntityAllocator::new();
        assert!(!allocator.is_alive(Entity::PLACEHOLDER));
    }

    #[test]
    fn test_allocator_is_alive_out_of_bounds() {
        let allocator = EntityAllocator::new();
        let fake_entity = Entity::new(999, 1);
        assert!(!allocator.is_alive(fake_entity));
    }

    #[test]
    fn test_allocator_is_alive_wrong_generation() {
        let mut allocator = EntityAllocator::new();
        let entity = allocator.allocate();

        // Wrong generation
        let stale = Entity::new(entity.index(), entity.generation() + 1);
        assert!(!allocator.is_alive(stale));
    }

    // -------------------------------------------------------------------------
    // Slot Recycling Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_allocator_recycling_basic() {
        let mut allocator = EntityAllocator::new();

        // Allocate and deallocate
        let e1 = allocator.allocate();
        assert!(allocator.deallocate(e1));

        // Allocate again - should reuse the slot
        let e2 = allocator.allocate();

        // Same index, different generation
        assert_eq!(e1.index(), e2.index());
        assert_ne!(e1.generation(), e2.generation());
        assert_eq!(e2.generation(), 2); // Generation incremented

        // e1 is dead, e2 is alive
        assert!(!allocator.is_alive(e1));
        assert!(allocator.is_alive(e2));
    }

    #[test]
    fn test_allocator_recycling_multiple() {
        let mut allocator = EntityAllocator::new();

        // Allocate 5 entities
        let entities: Vec<_> = (0..5).map(|_| allocator.allocate()).collect();
        assert_eq!(allocator.len(), 5);
        assert_eq!(allocator.capacity(), 5);

        // Deallocate all
        for entity in &entities {
            assert!(allocator.deallocate(*entity));
        }
        assert_eq!(allocator.len(), 0);
        assert_eq!(allocator.capacity(), 5); // Capacity unchanged

        // Allocate 5 more - should reuse slots
        let new_entities: Vec<_> = (0..5).map(|_| allocator.allocate()).collect();
        assert_eq!(allocator.len(), 5);
        assert_eq!(allocator.capacity(), 5); // Still 5, no new slots created

        // All new entities should have generation 2
        for entity in &new_entities {
            assert_eq!(entity.generation(), 2);
        }

        // Old entities are dead, new ones are alive
        for entity in &entities {
            assert!(!allocator.is_alive(*entity));
        }
        for entity in &new_entities {
            assert!(allocator.is_alive(*entity));
        }
    }

    #[test]
    fn test_allocator_recycling_lifo_order() {
        let mut allocator = EntityAllocator::new();

        // Allocate 3 entities
        let e0 = allocator.allocate();
        let e1 = allocator.allocate();
        let e2 = allocator.allocate();

        // Deallocate in order: e0, e1, e2
        allocator.deallocate(e0);
        allocator.deallocate(e1);
        allocator.deallocate(e2);

        // Reallocate - should come back in reverse order (LIFO)
        let new_e2 = allocator.allocate(); // Should reuse e2's slot
        let new_e1 = allocator.allocate(); // Should reuse e1's slot
        let new_e0 = allocator.allocate(); // Should reuse e0's slot

        assert_eq!(new_e2.index(), e2.index());
        assert_eq!(new_e1.index(), e1.index());
        assert_eq!(new_e0.index(), e0.index());
    }

    #[test]
    fn test_allocator_generation_increment() {
        let mut allocator = EntityAllocator::new();

        // Allocate and deallocate the same slot multiple times
        let mut last_gen = 0;
        for expected_gen in 1..=10 {
            let entity = allocator.allocate();
            assert_eq!(entity.index(), 0); // Always slot 0
            assert_eq!(entity.generation(), expected_gen);
            assert!(entity.generation() > last_gen);
            last_gen = entity.generation();

            allocator.deallocate(entity);
        }
    }

    // -------------------------------------------------------------------------
    // len(), capacity(), is_empty() Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_allocator_len() {
        let mut allocator = EntityAllocator::new();
        assert_eq!(allocator.len(), 0);

        let e1 = allocator.allocate();
        assert_eq!(allocator.len(), 1);

        let e2 = allocator.allocate();
        assert_eq!(allocator.len(), 2);

        allocator.deallocate(e1);
        assert_eq!(allocator.len(), 1);

        allocator.deallocate(e2);
        assert_eq!(allocator.len(), 0);
    }

    #[test]
    fn test_allocator_capacity() {
        let mut allocator = EntityAllocator::new();
        assert_eq!(allocator.capacity(), 0);

        let e1 = allocator.allocate();
        assert_eq!(allocator.capacity(), 1);

        let e2 = allocator.allocate();
        assert_eq!(allocator.capacity(), 2);

        // Capacity doesn't decrease on deallocation
        allocator.deallocate(e1);
        assert_eq!(allocator.capacity(), 2);

        allocator.deallocate(e2);
        assert_eq!(allocator.capacity(), 2);

        // Reusing slots doesn't increase capacity
        allocator.allocate();
        allocator.allocate();
        assert_eq!(allocator.capacity(), 2);

        // New allocation beyond capacity increases it
        allocator.allocate();
        assert_eq!(allocator.capacity(), 3);
    }

    #[test]
    fn test_allocator_is_empty() {
        let mut allocator = EntityAllocator::new();
        assert!(allocator.is_empty());

        let entity = allocator.allocate();
        assert!(!allocator.is_empty());

        allocator.deallocate(entity);
        assert!(allocator.is_empty());
    }

    // -------------------------------------------------------------------------
    // Edge Cases and Stress Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_allocator_many_allocations() {
        let mut allocator = EntityAllocator::new();
        const COUNT: usize = 10_000;

        // Allocate many entities
        let entities: Vec<_> = (0..COUNT).map(|_| allocator.allocate()).collect();
        assert_eq!(allocator.len(), COUNT);

        // All should be alive
        for entity in &entities {
            assert!(allocator.is_alive(*entity));
        }

        // Deallocate half
        for entity in entities.iter().take(COUNT / 2) {
            allocator.deallocate(*entity);
        }
        assert_eq!(allocator.len(), COUNT / 2);

        // Deallocated ones are dead
        for entity in entities.iter().take(COUNT / 2) {
            assert!(!allocator.is_alive(*entity));
        }

        // Remaining are alive
        for entity in entities.iter().skip(COUNT / 2) {
            assert!(allocator.is_alive(*entity));
        }
    }

    #[test]
    fn test_allocator_stress_allocate_deallocate_cycle() {
        let mut allocator = EntityAllocator::new();
        const CYCLES: usize = 100;
        const ENTITIES_PER_CYCLE: usize = 100;

        for _ in 0..CYCLES {
            // Allocate
            let entities: Vec<_> = (0..ENTITIES_PER_CYCLE)
                .map(|_| allocator.allocate())
                .collect();

            // Verify all alive
            for entity in &entities {
                assert!(allocator.is_alive(*entity));
            }

            // Deallocate all
            for entity in &entities {
                assert!(allocator.deallocate(*entity));
            }

            // Verify all dead
            for entity in &entities {
                assert!(!allocator.is_alive(*entity));
            }
        }

        // After all cycles, should be empty but have capacity
        assert!(allocator.is_empty());
        assert_eq!(allocator.capacity(), ENTITIES_PER_CYCLE);
    }

    #[test]
    fn test_allocator_unique_entities() {
        let mut allocator = EntityAllocator::new();
        let mut seen = HashSet::new();

        // Allocate, deallocate, and reallocate many times
        for _ in 0..1000 {
            let entity = allocator.allocate();

            // Each entity should be unique (index + generation combination)
            let key = entity.to_bits();
            assert!(
                seen.insert(key),
                "Duplicate entity: {:?} (bits: {})",
                entity,
                key
            );

            // 50% chance of deallocating
            if seen.len() % 2 == 0 {
                allocator.deallocate(entity);
            }
        }
    }

    // =========================================================================
    // Bulk Operations Tests
    // =========================================================================

    #[test]
    fn test_allocate_batch_empty() {
        let mut allocator = EntityAllocator::new();

        let entities = allocator.allocate_batch(0);
        assert!(entities.is_empty());
        assert_eq!(allocator.len(), 0);
        assert_eq!(allocator.capacity(), 0);
    }

    #[test]
    fn test_allocate_batch_basic() {
        let mut allocator = EntityAllocator::new();

        let entities = allocator.allocate_batch(100);
        assert_eq!(entities.len(), 100);
        assert_eq!(allocator.len(), 100);
        assert_eq!(allocator.capacity(), 100);

        // All entities should be unique
        let unique: HashSet<_> = entities.iter().collect();
        assert_eq!(unique.len(), 100);

        // All entities should be alive
        for entity in &entities {
            assert!(allocator.is_alive(*entity));
            assert!(!entity.is_placeholder());
        }

        // All should have generation 1 (first allocation)
        for entity in &entities {
            assert_eq!(entity.generation(), 1);
        }
    }

    #[test]
    fn test_allocate_batch_reuses_free_slots() {
        let mut allocator = EntityAllocator::new();

        // Allocate 50 entities, then deallocate them all
        let first_batch = allocator.allocate_batch(50);
        assert_eq!(allocator.len(), 50);

        for entity in &first_batch {
            allocator.deallocate(*entity);
        }
        assert_eq!(allocator.len(), 0);
        assert_eq!(allocator.capacity(), 50);

        // Allocate 50 more - should reuse all freed slots
        let second_batch = allocator.allocate_batch(50);
        assert_eq!(allocator.len(), 50);
        assert_eq!(allocator.capacity(), 50); // No new slots created

        // All should have generation 2 (recycled)
        for entity in &second_batch {
            assert_eq!(entity.generation(), 2);
        }

        // All original entities should be dead
        for entity in &first_batch {
            assert!(!allocator.is_alive(*entity));
        }

        // All new entities should be alive
        for entity in &second_batch {
            assert!(allocator.is_alive(*entity));
        }
    }

    #[test]
    fn test_allocate_batch_mixed_reuse_and_new() {
        let mut allocator = EntityAllocator::new();

        // Allocate 30, deallocate all
        let first = allocator.allocate_batch(30);
        for e in &first {
            allocator.deallocate(*e);
        }
        assert_eq!(allocator.capacity(), 30);

        // Now allocate 50 - should reuse 30, create 20 new
        let second = allocator.allocate_batch(50);
        assert_eq!(second.len(), 50);
        assert_eq!(allocator.len(), 50);
        assert_eq!(allocator.capacity(), 50); // Grew by 20

        // Count generations
        let gen1_count = second.iter().filter(|e| e.generation() == 1).count();
        let gen2_count = second.iter().filter(|e| e.generation() == 2).count();

        assert_eq!(gen1_count, 20); // New slots
        assert_eq!(gen2_count, 30); // Recycled slots
    }

    #[test]
    fn test_allocate_batch_large() {
        let mut allocator = EntityAllocator::new();

        let entities = allocator.allocate_batch(10_000);
        assert_eq!(entities.len(), 10_000);
        assert_eq!(allocator.len(), 10_000);

        // All should be unique
        let unique: HashSet<_> = entities.iter().collect();
        assert_eq!(unique.len(), 10_000);

        // All should be alive
        for entity in &entities {
            assert!(allocator.is_alive(*entity));
        }
    }

    #[test]
    fn test_deallocate_batch_empty() {
        let mut allocator = EntityAllocator::new();

        let deallocated = allocator.deallocate_batch(&[]);
        assert_eq!(deallocated, 0);
    }

    #[test]
    fn test_deallocate_batch_basic() {
        let mut allocator = EntityAllocator::new();

        let entities = allocator.allocate_batch(100);
        assert_eq!(allocator.len(), 100);

        let deallocated = allocator.deallocate_batch(&entities);
        assert_eq!(deallocated, 100);
        assert_eq!(allocator.len(), 0);

        // All should be dead
        for entity in &entities {
            assert!(!allocator.is_alive(*entity));
        }
    }

    #[test]
    fn test_deallocate_batch_partial_invalid() {
        let mut allocator = EntityAllocator::new();

        let entities = allocator.allocate_batch(10);

        // Deallocate some individually first
        allocator.deallocate(entities[0]);
        allocator.deallocate(entities[2]);
        allocator.deallocate(entities[4]);

        // Now batch deallocate all - should only succeed for 7
        let deallocated = allocator.deallocate_batch(&entities);
        assert_eq!(deallocated, 7); // 10 - 3 already deallocated

        // All should be dead now
        for entity in &entities {
            assert!(!allocator.is_alive(*entity));
        }
    }

    #[test]
    fn test_deallocate_batch_all_invalid() {
        let mut allocator = EntityAllocator::new();

        let entities = allocator.allocate_batch(10);

        // Deallocate all individually
        for entity in &entities {
            allocator.deallocate(*entity);
        }

        // Batch deallocate should return 0
        let deallocated = allocator.deallocate_batch(&entities);
        assert_eq!(deallocated, 0);
    }

    #[test]
    fn test_deallocate_batch_with_placeholder() {
        let mut allocator = EntityAllocator::new();

        let entities = allocator.allocate_batch(5);
        let mut with_placeholder: Vec<Entity> = entities.clone();
        with_placeholder.push(Entity::PLACEHOLDER);
        with_placeholder.push(Entity::PLACEHOLDER);

        let deallocated = allocator.deallocate_batch(&with_placeholder);
        assert_eq!(deallocated, 5); // Only the 5 valid entities

        assert!(allocator.is_empty());
    }

    #[test]
    fn test_deallocate_batch_with_out_of_bounds() {
        let mut allocator = EntityAllocator::new();

        let entities = allocator.allocate_batch(5);
        let mut with_invalid: Vec<Entity> = entities.clone();
        with_invalid.push(Entity::new(9999, 1)); // Out of bounds
        with_invalid.push(Entity::new(10000, 1)); // Out of bounds

        let deallocated = allocator.deallocate_batch(&with_invalid);
        assert_eq!(deallocated, 5); // Only the 5 valid entities
    }

    #[test]
    fn test_reserve_basic() {
        let mut allocator = EntityAllocator::new();

        allocator.reserve(1000);

        // No entities allocated, but memory reserved
        assert_eq!(allocator.len(), 0);
        assert_eq!(allocator.capacity(), 0);

        // Now allocate - no reallocation should occur
        let entities = allocator.allocate_batch(500);
        assert_eq!(entities.len(), 500);
        assert_eq!(allocator.len(), 500);
    }

    #[test]
    fn test_reserve_after_allocations() {
        let mut allocator = EntityAllocator::new();

        // Allocate some entities first
        let _first = allocator.allocate_batch(100);
        assert_eq!(allocator.capacity(), 100);

        // Reserve more
        allocator.reserve(1000);

        // Allocate many more
        let second = allocator.allocate_batch(1000);
        assert_eq!(second.len(), 1000);
        assert_eq!(allocator.len(), 1100);
    }

    #[test]
    fn test_reserve_zero() {
        let mut allocator = EntityAllocator::new();

        allocator.reserve(0);
        assert_eq!(allocator.len(), 0);
        assert_eq!(allocator.capacity(), 0);
    }

    #[test]
    fn test_batch_stress_test() {
        let mut allocator = EntityAllocator::new();
        const BATCH_SIZE: usize = 1000;
        const ITERATIONS: usize = 100;

        for _ in 0..ITERATIONS {
            // Allocate batch
            let entities = allocator.allocate_batch(BATCH_SIZE);
            assert_eq!(entities.len(), BATCH_SIZE);

            // Verify all alive
            for entity in &entities {
                assert!(allocator.is_alive(*entity));
            }

            // Deallocate batch
            let deallocated = allocator.deallocate_batch(&entities);
            assert_eq!(deallocated, BATCH_SIZE);

            // Verify all dead
            for entity in &entities {
                assert!(!allocator.is_alive(*entity));
            }
        }

        // After all iterations, should be empty
        assert!(allocator.is_empty());

        // Capacity should be exactly BATCH_SIZE (slots reused each iteration)
        assert_eq!(allocator.capacity(), BATCH_SIZE);
    }

    #[test]
    fn test_batch_vs_individual_equivalence() {
        // Verify that batch operations produce equivalent results to individual ops

        let mut batch_allocator = EntityAllocator::new();
        let mut individual_allocator = EntityAllocator::new();

        // Allocate same count via batch vs individual
        let batch_entities = batch_allocator.allocate_batch(100);
        let individual_entities: Vec<_> =
            (0..100).map(|_| individual_allocator.allocate()).collect();

        assert_eq!(batch_allocator.len(), individual_allocator.len());
        assert_eq!(batch_allocator.capacity(), individual_allocator.capacity());

        // Same number of unique entities
        assert_eq!(batch_entities.len(), individual_entities.len());

        // All entities should match in structure (index 0-99, generation 1)
        for i in 0..100 {
            // Since both allocate sequentially with no free slots, indices match
            assert_eq!(batch_entities[i].index() as usize, i);
            assert_eq!(individual_entities[i].index() as usize, i);
            assert_eq!(batch_entities[i].generation(), 1);
            assert_eq!(individual_entities[i].generation(), 1);
        }

        // Deallocate via batch vs individual
        let batch_count = batch_allocator.deallocate_batch(&batch_entities);
        let individual_count = individual_entities
            .iter()
            .filter(|e| individual_allocator.deallocate(**e))
            .count();

        assert_eq!(batch_count, individual_count);
        assert_eq!(batch_allocator.len(), individual_allocator.len());
    }
}
