//! Entity identifier type for the ECS.
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
//! # FFI Safety
//!
//! Entity uses `#[repr(C)]` for predictable memory layout across FFI boundaries.
//! The struct is exactly 8 bytes: 4 bytes for index + 4 bytes for generation.

use std::fmt;
use std::hash::{Hash, Hasher};

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
    /// This is primarily used by [`EntityAllocator`](super::EntityAllocator). Direct
    /// construction is possible but not recommended for typical use.
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
