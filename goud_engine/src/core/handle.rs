//! Type-safe, generation-counted handles for engine objects.
//!
//! Handles are the primary mechanism for referencing engine objects across the FFI
//! boundary. They provide:
//!
//! - **Type safety**: Handles are generic over the resource type, preventing
//!   accidental use of a texture handle where a shader handle is expected.
//! - **Generation counting**: Each handle includes a generation counter that
//!   increments when a slot is reused, preventing use-after-free bugs.
//! - **FFI compatibility**: The `#[repr(C)]` layout ensures consistent memory
//!   representation across language boundaries.
//!
//! # Design Pattern: Generational Indices
//!
//! Generational indices solve the ABA problem in resource management:
//!
//! 1. Allocate slot 5, generation 1 -> Handle(5, 1)
//! 2. Deallocate slot 5, generation becomes 2
//! 3. Allocate slot 5 again, generation 2 -> Handle(5, 2)
//! 4. Old Handle(5, 1) is now invalid (generation mismatch)
//!
//! # Example
//!
//! ```
//! use goud_engine::core::handle::Handle;
//!
//! // Marker type for textures
//! struct Texture;
//!
//! // Create a handle (normally done by HandleAllocator)
//! let handle: Handle<Texture> = Handle::new(0, 1);
//!
//! assert_eq!(handle.index(), 0);
//! assert_eq!(handle.generation(), 1);
//! assert!(handle.is_valid());
//!
//! // Invalid handle for representing "no resource"
//! let invalid: Handle<Texture> = Handle::INVALID;
//! assert!(!invalid.is_valid());
//! ```

use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

// =============================================================================
// HandleAllocator
// =============================================================================

/// Manages handle allocation with generation counting and free-list recycling.
///
/// The `HandleAllocator` is responsible for creating and invalidating handles
/// in a memory-efficient manner. It uses a generational index scheme where:
///
/// - Each slot has a generation counter that increments on deallocation
/// - Deallocated slots are recycled via a free list
/// - Handles carry their generation, allowing stale handle detection
///
/// # Design Pattern: Generational Arena Allocator
///
/// This pattern provides:
/// - O(1) allocation (pop from free list or push new entry)
/// - O(1) deallocation (push to free list, increment generation)
/// - O(1) liveness check (compare generations)
/// - Memory reuse without use-after-free bugs
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleAllocator;
///
/// struct Texture;
///
/// let mut allocator: HandleAllocator<Texture> = HandleAllocator::new();
///
/// // Allocate some handles
/// let h1 = allocator.allocate();
/// let h2 = allocator.allocate();
///
/// assert!(allocator.is_alive(h1));
/// assert!(allocator.is_alive(h2));
///
/// // Deallocate h1
/// assert!(allocator.deallocate(h1));
/// assert!(!allocator.is_alive(h1));  // h1 is now stale
///
/// // Allocate again - may reuse h1's slot with new generation
/// let h3 = allocator.allocate();
/// assert!(allocator.is_alive(h3));
///
/// // h1 still references old generation, so it's still not alive
/// assert!(!allocator.is_alive(h1));
/// ```
///
/// # Thread Safety
///
/// `HandleAllocator` is NOT thread-safe. For concurrent access, wrap in
/// appropriate synchronization primitives (Mutex, RwLock, etc.).
pub struct HandleAllocator<T> {
    /// Generation counter for each slot.
    ///
    /// Index `i` stores the current generation for slot `i`.
    /// Generation starts at 1 for new slots (0 is reserved for never-allocated).
    generations: Vec<u32>,

    /// Stack of free slot indices available for reuse.
    ///
    /// When a handle is deallocated, its index is pushed here.
    /// On allocation, we prefer to pop from this list before growing.
    free_list: Vec<u32>,

    /// Phantom marker for type parameter.
    _marker: PhantomData<T>,
}

impl<T> HandleAllocator<T> {
    /// Creates a new, empty handle allocator.
    ///
    /// The allocator starts with no pre-allocated capacity.
    /// Use [`with_capacity`](Self::with_capacity) for bulk allocation scenarios.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Mesh;
    ///
    /// let allocator: HandleAllocator<Mesh> = HandleAllocator::new();
    /// assert_eq!(allocator.len(), 0);
    /// assert!(allocator.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Creates a new handle allocator with pre-allocated capacity.
    ///
    /// This is useful when you know approximately how many handles you'll need,
    /// as it avoids repeated reallocations during bulk allocation.
    ///
    /// Note that this only pre-allocates memory for the internal vectors;
    /// it does not create any handles. Use [`allocate`](Self::allocate) to
    /// actually create handles.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of slots to pre-allocate
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Entity;
    ///
    /// // Pre-allocate space for 1000 entities
    /// let mut allocator: HandleAllocator<Entity> = HandleAllocator::with_capacity(1000);
    ///
    /// // No handles allocated yet, but memory is reserved
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
            _marker: PhantomData,
        }
    }

    /// Allocates a new handle.
    ///
    /// If there are slots in the free list, one is reused with an incremented
    /// generation. Otherwise, a new slot is created with generation 1.
    ///
    /// # Returns
    ///
    /// A new, valid handle that can be used to reference a resource.
    ///
    /// # Panics
    ///
    /// Panics if the number of slots exceeds `u32::MAX - 1` (unlikely in practice).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Shader;
    ///
    /// let mut allocator: HandleAllocator<Shader> = HandleAllocator::new();
    ///
    /// let h1 = allocator.allocate();
    /// let h2 = allocator.allocate();
    ///
    /// assert_ne!(h1, h2);
    /// assert!(h1.is_valid());
    /// assert!(h2.is_valid());
    /// ```
    pub fn allocate(&mut self) -> Handle<T> {
        if let Some(index) = self.free_list.pop() {
            // Reuse a slot from the free list
            // Generation was already incremented during deallocation
            let generation = self.generations[index as usize];
            Handle::new(index, generation)
        } else {
            // Allocate a new slot
            let index = self.generations.len();

            // Ensure we don't exceed u32::MAX - 1 (reserve MAX for INVALID)
            assert!(
                index < u32::MAX as usize,
                "HandleAllocator exceeded maximum capacity"
            );

            // New slots start at generation 1 (0 is reserved for never-allocated/INVALID)
            self.generations.push(1);

            Handle::new(index as u32, 1)
        }
    }

    /// Deallocates a handle, making it invalid.
    ///
    /// The slot's generation is incremented, invalidating any handles that
    /// reference the old generation. The slot is added to the free list for
    /// future reuse.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to deallocate
    ///
    /// # Returns
    ///
    /// - `true` if the handle was valid and successfully deallocated
    /// - `false` if the handle was invalid (wrong generation, out of bounds, or INVALID)
    ///
    /// # Double Deallocation
    ///
    /// Attempting to deallocate the same handle twice returns `false` on the
    /// second attempt, as the generation will have already been incremented.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Audio;
    ///
    /// let mut allocator: HandleAllocator<Audio> = HandleAllocator::new();
    /// let handle = allocator.allocate();
    ///
    /// assert!(allocator.is_alive(handle));
    /// assert!(allocator.deallocate(handle));  // First deallocation succeeds
    /// assert!(!allocator.is_alive(handle));
    /// assert!(!allocator.deallocate(handle)); // Second deallocation fails
    /// ```
    pub fn deallocate(&mut self, handle: Handle<T>) -> bool {
        // Check if handle is valid
        if !handle.is_valid() {
            return false;
        }

        let index = handle.index() as usize;

        // Check bounds
        if index >= self.generations.len() {
            return false;
        }

        // Check generation matches (handle is still alive)
        if self.generations[index] != handle.generation() {
            return false;
        }

        // Increment generation to invalidate existing handles
        // Wrap at u32::MAX to 1 (skip 0, which is reserved)
        let new_gen = self.generations[index].wrapping_add(1);
        self.generations[index] = if new_gen == 0 { 1 } else { new_gen };

        // Add to free list for reuse
        self.free_list.push(handle.index());

        true
    }

    /// Checks if a handle refers to a currently allocated resource.
    ///
    /// A handle is "alive" if:
    /// - It is not the INVALID sentinel
    /// - Its index is within bounds
    /// - Its generation matches the current slot generation
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to check
    ///
    /// # Returns
    ///
    /// `true` if the handle is alive, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Sprite;
    ///
    /// let mut allocator: HandleAllocator<Sprite> = HandleAllocator::new();
    /// let handle = allocator.allocate();
    ///
    /// assert!(allocator.is_alive(handle));
    ///
    /// allocator.deallocate(handle);
    /// assert!(!allocator.is_alive(handle));
    /// ```
    #[inline]
    pub fn is_alive(&self, handle: Handle<T>) -> bool {
        // INVALID handles are never alive
        if !handle.is_valid() {
            return false;
        }

        let index = handle.index() as usize;

        // Check bounds and generation
        index < self.generations.len() && self.generations[index] == handle.generation()
    }

    /// Returns the number of currently allocated (alive) handles.
    ///
    /// This is the total capacity minus the number of free slots.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Entity;
    ///
    /// let mut allocator: HandleAllocator<Entity> = HandleAllocator::new();
    /// assert_eq!(allocator.len(), 0);
    ///
    /// let h1 = allocator.allocate();
    /// let h2 = allocator.allocate();
    /// assert_eq!(allocator.len(), 2);
    ///
    /// allocator.deallocate(h1);
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
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Component;
    ///
    /// let mut allocator: HandleAllocator<Component> = HandleAllocator::new();
    /// let h1 = allocator.allocate();
    /// let h2 = allocator.allocate();
    ///
    /// assert_eq!(allocator.capacity(), 2);
    ///
    /// allocator.deallocate(h1);
    /// // Capacity remains 2, even though one slot is free
    /// assert_eq!(allocator.capacity(), 2);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.generations.len()
    }

    /// Returns `true` if no handles are currently allocated.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Resource;
    ///
    /// let mut allocator: HandleAllocator<Resource> = HandleAllocator::new();
    /// assert!(allocator.is_empty());
    ///
    /// let handle = allocator.allocate();
    /// assert!(!allocator.is_empty());
    ///
    /// allocator.deallocate(handle);
    /// assert!(allocator.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all allocations, invalidating all existing handles.
    ///
    /// This increments the generation of every slot, making all previously
    /// allocated handles stale. The capacity is retained, but `len()` becomes 0.
    ///
    /// Use this for "reset to initial state" scenarios, like level transitions
    /// in a game where all entities should be destroyed.
    ///
    /// # Performance
    ///
    /// This operation is O(n) where n is the capacity (number of slots).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Entity;
    ///
    /// let mut allocator: HandleAllocator<Entity> = HandleAllocator::new();
    ///
    /// let h1 = allocator.allocate();
    /// let h2 = allocator.allocate();
    /// let h3 = allocator.allocate();
    ///
    /// assert_eq!(allocator.len(), 3);
    /// assert!(allocator.is_alive(h1));
    ///
    /// allocator.clear();
    ///
    /// // All handles are now invalid
    /// assert_eq!(allocator.len(), 0);
    /// assert!(allocator.is_empty());
    /// assert!(!allocator.is_alive(h1));
    /// assert!(!allocator.is_alive(h2));
    /// assert!(!allocator.is_alive(h3));
    ///
    /// // Capacity is retained
    /// assert_eq!(allocator.capacity(), 3);
    ///
    /// // New allocations get incremented generations
    /// let h4 = allocator.allocate();
    /// assert_eq!(h4.generation(), 2);  // Was 1, now 2
    /// ```
    pub fn clear(&mut self) {
        // Increment all generations to invalidate existing handles
        for gen in &mut self.generations {
            let new_gen = gen.wrapping_add(1);
            *gen = if new_gen == 0 { 1 } else { new_gen };
        }

        // Rebuild free list with all slots
        self.free_list.clear();
        self.free_list.reserve(self.generations.len());
        for i in (0..self.generations.len()).rev() {
            self.free_list.push(i as u32);
        }
    }

    /// Shrinks the free list to fit its current contents.
    ///
    /// This reduces memory usage by releasing excess capacity in the free list.
    /// Note that this does NOT shrink the generations vector, as that would
    /// invalidate the capacity guarantee and require more complex bookkeeping.
    ///
    /// Call this after a batch of deallocations if you want to reduce memory
    /// pressure and don't expect to allocate more handles soon.
    ///
    /// # Performance
    ///
    /// This operation may reallocate the free list, which is O(n) where n is
    /// the number of free slots.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleAllocator;
    ///
    /// struct Resource;
    ///
    /// let mut allocator: HandleAllocator<Resource> = HandleAllocator::new();
    ///
    /// // Allocate many handles
    /// let handles: Vec<_> = (0..100).map(|_| allocator.allocate()).collect();
    ///
    /// // Deallocate most of them
    /// for h in handles.iter().skip(10) {
    ///     allocator.deallocate(*h);
    /// }
    ///
    /// // Free list now has 90 entries with possibly more capacity
    /// // Shrink to reduce memory
    /// allocator.shrink_to_fit();
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.free_list.shrink_to_fit();
    }
}

impl<T> Default for HandleAllocator<T> {
    /// Creates a new, empty handle allocator (same as `new()`).
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Debug for HandleAllocator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = std::any::type_name::<T>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        f.debug_struct(&format!("HandleAllocator<{}>", short_name))
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("free_slots", &self.free_list.len())
            .finish()
    }
}

/// A type-safe, generation-counted handle to an engine resource.
///
/// Handles are lightweight (8 bytes) identifiers that can be safely passed
/// across FFI boundaries. The generic type parameter `T` provides compile-time
/// type safety, ensuring handles for different resource types cannot be mixed.
///
/// # FFI Safety
///
/// The `#[repr(C)]` attribute ensures this struct has a predictable memory
/// layout for interoperability with C#, Python, and other languages:
/// - Offset 0: `index` (4 bytes, u32)
/// - Offset 4: `generation` (4 bytes, u32)
/// - Total size: 8 bytes, alignment: 4 bytes
///
/// The `PhantomData<T>` is a zero-sized type that doesn't affect the layout.
#[repr(C)]
pub struct Handle<T> {
    /// Index into the storage array.
    ///
    /// This is the slot number in the allocator/storage. When a handle is
    /// deallocated, its index may be reused for a new allocation.
    index: u32,

    /// Generation counter for this slot.
    ///
    /// Incremented each time the slot is deallocated. A handle is only valid
    /// if its generation matches the current generation of the slot.
    generation: u32,

    /// Marker to make `Handle<T>` generic over T without storing T.
    ///
    /// This provides compile-time type safety: `Handle<Texture>` and
    /// `Handle<Shader>` are distinct types that cannot be accidentally mixed.
    _marker: PhantomData<T>,
}

impl<T> Handle<T> {
    /// The invalid handle constant.
    ///
    /// Used to represent "no resource" or "null handle". This is distinguishable
    /// from any valid handle because:
    /// - `index` is `u32::MAX`, which exceeds any reasonable allocation count
    /// - `generation` is 0, which is never used for valid allocations
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::Handle;
    ///
    /// struct Texture;
    ///
    /// let handle: Handle<Texture> = Handle::INVALID;
    /// assert!(!handle.is_valid());
    /// assert_eq!(handle.index(), u32::MAX);
    /// assert_eq!(handle.generation(), 0);
    /// ```
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
        _marker: PhantomData,
    };

    /// Creates a new handle with the given index and generation.
    ///
    /// This is typically called by `HandleAllocator`, not by user code.
    /// Users should obtain handles through the allocator or storage APIs.
    ///
    /// # Arguments
    ///
    /// * `index` - The slot index in the storage array
    /// * `generation` - The generation counter for this slot
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::Handle;
    ///
    /// struct Shader;
    ///
    /// let handle: Handle<Shader> = Handle::new(42, 3);
    /// assert_eq!(handle.index(), 42);
    /// assert_eq!(handle.generation(), 3);
    /// ```
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// Returns the index component of this handle.
    ///
    /// The index is the slot number in the allocator/storage. It identifies
    /// which entry the handle refers to.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::Handle;
    ///
    /// struct Mesh;
    ///
    /// let handle: Handle<Mesh> = Handle::new(10, 1);
    /// assert_eq!(handle.index(), 10);
    /// ```
    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation component of this handle.
    ///
    /// The generation is incremented each time a slot is deallocated and
    /// reused. It prevents use-after-free by invalidating old handles.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::Handle;
    ///
    /// struct Audio;
    ///
    /// let handle: Handle<Audio> = Handle::new(5, 7);
    /// assert_eq!(handle.generation(), 7);
    /// ```
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Checks if this handle is valid (not the INVALID sentinel).
    ///
    /// A handle is considered valid if it is not equal to `Handle::INVALID`.
    /// Note that a "valid" handle here only means it's not the null sentinel;
    /// it may still refer to a deallocated resource (stale handle).
    ///
    /// To check if a handle refers to a live resource, use the allocator's
    /// `is_alive()` method.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::Handle;
    ///
    /// struct Sprite;
    ///
    /// let valid: Handle<Sprite> = Handle::new(0, 1);
    /// assert!(valid.is_valid());
    ///
    /// let invalid: Handle<Sprite> = Handle::INVALID;
    /// assert!(!invalid.is_valid());
    /// ```
    #[inline]
    pub const fn is_valid(&self) -> bool {
        // INVALID has index=u32::MAX and generation=0
        // A handle is invalid if it matches INVALID exactly
        !(self.index == u32::MAX && self.generation == 0)
    }

    /// Packs this handle into a single u64 value.
    ///
    /// The packed format is:
    /// - Upper 32 bits: generation
    /// - Lower 32 bits: index
    ///
    /// This is useful for FFI with languages that prefer a single integer
    /// over a struct, and for use as hash map keys.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::Handle;
    ///
    /// struct Resource;
    ///
    /// let handle: Handle<Resource> = Handle::new(42, 7);
    /// let packed = handle.to_u64();
    /// let unpacked: Handle<Resource> = Handle::from_u64(packed);
    ///
    /// assert_eq!(handle, unpacked);
    /// ```
    #[inline]
    pub const fn to_u64(&self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }

    /// Creates a handle from a packed u64 value.
    ///
    /// This is the inverse of `to_u64()`. The packed format is:
    /// - Upper 32 bits: generation
    /// - Lower 32 bits: index
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::Handle;
    ///
    /// struct Resource;
    ///
    /// let packed: u64 = (7u64 << 32) | 42u64;  // gen=7, index=42
    /// let handle: Handle<Resource> = Handle::from_u64(packed);
    ///
    /// assert_eq!(handle.index(), 42);
    /// assert_eq!(handle.generation(), 7);
    /// ```
    #[inline]
    pub const fn from_u64(packed: u64) -> Self {
        let index = packed as u32;
        let generation = (packed >> 32) as u32;
        Self::new(index, generation)
    }
}

// =============================================================================
// Trait Implementations
// =============================================================================

impl<T> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Get the type name, stripping the module path for readability
        let type_name = std::any::type_name::<T>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        write!(
            f,
            "Handle<{}>({}:{})",
            short_name, self.index, self.generation
        )
    }
}

impl<T> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // Both index AND generation must match for handles to be equal
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the packed u64 representation for consistent hashing
        // This ensures hash is consistent with PartialEq
        self.to_u64().hash(state);
    }
}

impl<T> Default for Handle<T> {
    /// Returns `Handle::INVALID`.
    ///
    /// The default handle is the invalid sentinel, representing "no resource".
    /// This is useful for struct initialization where a handle field may not
    /// yet have a valid value.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::Handle;
    ///
    /// struct Texture;
    ///
    /// let handle: Handle<Texture> = Default::default();
    /// assert!(!handle.is_valid());
    /// assert_eq!(handle, Handle::INVALID);
    /// ```
    #[inline]
    fn default() -> Self {
        Self::INVALID
    }
}

impl<T> From<Handle<T>> for u64 {
    /// Converts a handle to its packed u64 representation.
    ///
    /// Format: upper 32 bits = generation, lower 32 bits = index.
    #[inline]
    fn from(handle: Handle<T>) -> u64 {
        handle.to_u64()
    }
}

impl<T> From<u64> for Handle<T> {
    /// Creates a handle from a packed u64 representation.
    ///
    /// Format: upper 32 bits = generation, lower 32 bits = index.
    #[inline]
    fn from(packed: u64) -> Self {
        Handle::from_u64(packed)
    }
}

// =============================================================================
// HandleMap
// =============================================================================

/// A map that associates handles with values using generational indices.
///
/// `HandleMap` is a slot-map data structure that combines a `HandleAllocator`
/// with value storage. It provides:
///
/// - O(1) insertion, returning a new handle
/// - O(1) lookup by handle
/// - O(1) removal by handle
/// - Generational safety: stale handles return None, never wrong values
///
/// This is the primary storage mechanism for engine resources. Each resource
/// type (textures, shaders, entities, etc.) typically has its own `HandleMap`.
///
/// # Type Parameters
///
/// - `T`: Marker type for the handle (provides type safety)
/// - `V`: The value type stored in the map
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// // Marker type for textures
/// struct Texture;
///
/// // Value stored for each texture
/// struct TextureData {
///     width: u32,
///     height: u32,
///     path: String,
/// }
///
/// let mut textures: HandleMap<Texture, TextureData> = HandleMap::new();
///
/// // Insert a texture and get a handle
/// let handle = textures.insert(TextureData {
///     width: 1024,
///     height: 768,
///     path: "player.png".to_string(),
/// });
///
/// // Lookup by handle
/// if let Some(tex) = textures.get(handle) {
///     assert_eq!(tex.width, 1024);
/// }
///
/// // Remove by handle
/// let removed = textures.remove(handle);
/// assert!(removed.is_some());
///
/// // Handle is now stale
/// assert!(textures.get(handle).is_none());
/// ```
///
/// # Thread Safety
///
/// `HandleMap` is NOT thread-safe. For concurrent access, wrap in
/// appropriate synchronization primitives (Mutex, RwLock, etc.).
///
/// # Design Pattern: Slot Map
///
/// This is a slot map (or handle-based storage) pattern commonly used in
/// game engines for:
/// - Stable references that survive reallocation
/// - Safe deletion detection without dangling pointers
/// - FFI-safe handles that can be passed across language boundaries
pub struct HandleMap<T, V> {
    /// The handle allocator managing index and generation tracking.
    allocator: HandleAllocator<T>,

    /// Storage for values, indexed by handle index.
    ///
    /// Entries are `Some(value)` for live handles, `None` for deallocated slots.
    /// The index in this vector corresponds to `handle.index()`.
    values: Vec<Option<V>>,
}

impl<T, V> HandleMap<T, V> {
    /// Creates a new, empty handle map.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Mesh;
    /// struct MeshData { vertex_count: usize }
    ///
    /// let map: HandleMap<Mesh, MeshData> = HandleMap::new();
    /// assert!(map.is_empty());
    /// assert_eq!(map.len(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            allocator: HandleAllocator::new(),
            values: Vec::new(),
        }
    }

    /// Creates a new handle map with pre-allocated capacity.
    ///
    /// This is useful when you know approximately how many entries you'll need,
    /// as it avoids repeated reallocations during bulk insertion.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of entries to pre-allocate space for
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Entity;
    /// struct EntityData { name: String }
    ///
    /// let map: HandleMap<Entity, EntityData> = HandleMap::with_capacity(1000);
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            allocator: HandleAllocator::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    /// Inserts a value into the map and returns a handle to it.
    ///
    /// The returned handle can be used to retrieve, modify, or remove the value.
    /// The handle remains valid until the value is removed.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to insert
    ///
    /// # Returns
    ///
    /// A handle that can be used to access the inserted value.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Shader;
    /// struct ShaderData { name: String }
    ///
    /// let mut map: HandleMap<Shader, ShaderData> = HandleMap::new();
    ///
    /// let h1 = map.insert(ShaderData { name: "basic".to_string() });
    /// let h2 = map.insert(ShaderData { name: "pbr".to_string() });
    ///
    /// assert_ne!(h1, h2);
    /// assert!(h1.is_valid());
    /// assert!(h2.is_valid());
    /// assert_eq!(map.len(), 2);
    /// ```
    pub fn insert(&mut self, value: V) -> Handle<T> {
        let handle = self.allocator.allocate();
        let index = handle.index() as usize;

        // Ensure values vec is large enough
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }

        self.values[index] = Some(value);
        handle
    }

    /// Removes a value from the map by its handle.
    ///
    /// If the handle is valid and points to an existing value, the value is
    /// removed and returned. The handle becomes stale after this operation.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the value to remove
    ///
    /// # Returns
    ///
    /// - `Some(value)` if the handle was valid and the value was removed
    /// - `None` if the handle was invalid, stale, or already removed
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Audio;
    /// struct AudioClip { duration: f32 }
    ///
    /// let mut map: HandleMap<Audio, AudioClip> = HandleMap::new();
    ///
    /// let handle = map.insert(AudioClip { duration: 5.5 });
    /// assert_eq!(map.len(), 1);
    ///
    /// let removed = map.remove(handle);
    /// assert!(removed.is_some());
    /// assert_eq!(removed.unwrap().duration, 5.5);
    /// assert_eq!(map.len(), 0);
    ///
    /// // Handle is now stale
    /// assert!(map.remove(handle).is_none());
    /// ```
    pub fn remove(&mut self, handle: Handle<T>) -> Option<V> {
        // Check if handle is alive
        if !self.allocator.is_alive(handle) {
            return None;
        }

        let index = handle.index() as usize;

        // Deallocate the handle
        if !self.allocator.deallocate(handle) {
            return None;
        }

        // Take the value from storage
        self.values.get_mut(index).and_then(|slot| slot.take())
    }

    /// Returns a reference to the value associated with a handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to look up
    ///
    /// # Returns
    ///
    /// - `Some(&V)` if the handle is valid and points to a value
    /// - `None` if the handle is invalid, stale, or the value was removed
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Sprite;
    /// struct SpriteData { x: f32, y: f32 }
    ///
    /// let mut map: HandleMap<Sprite, SpriteData> = HandleMap::new();
    ///
    /// let handle = map.insert(SpriteData { x: 100.0, y: 200.0 });
    ///
    /// if let Some(sprite) = map.get(handle) {
    ///     assert_eq!(sprite.x, 100.0);
    ///     assert_eq!(sprite.y, 200.0);
    /// }
    /// ```
    #[inline]
    pub fn get(&self, handle: Handle<T>) -> Option<&V> {
        if !self.allocator.is_alive(handle) {
            return None;
        }

        let index = handle.index() as usize;
        self.values.get(index).and_then(|slot| slot.as_ref())
    }

    /// Returns a mutable reference to the value associated with a handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to look up
    ///
    /// # Returns
    ///
    /// - `Some(&mut V)` if the handle is valid and points to a value
    /// - `None` if the handle is invalid, stale, or the value was removed
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Transform;
    /// struct TransformData { x: f32, y: f32 }
    ///
    /// let mut map: HandleMap<Transform, TransformData> = HandleMap::new();
    ///
    /// let handle = map.insert(TransformData { x: 0.0, y: 0.0 });
    ///
    /// if let Some(transform) = map.get_mut(handle) {
    ///     transform.x = 50.0;
    ///     transform.y = 100.0;
    /// }
    ///
    /// assert_eq!(map.get(handle).unwrap().x, 50.0);
    /// ```
    #[inline]
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut V> {
        if !self.allocator.is_alive(handle) {
            return None;
        }

        let index = handle.index() as usize;
        self.values.get_mut(index).and_then(|slot| slot.as_mut())
    }

    /// Checks if a handle is valid and points to a value in this map.
    ///
    /// This is equivalent to `self.get(handle).is_some()` but more efficient.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to check
    ///
    /// # Returns
    ///
    /// `true` if the handle is valid and the value exists, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Entity;
    /// struct EntityData { id: u32 }
    ///
    /// let mut map: HandleMap<Entity, EntityData> = HandleMap::new();
    ///
    /// let handle = map.insert(EntityData { id: 1 });
    /// assert!(map.contains(handle));
    ///
    /// map.remove(handle);
    /// assert!(!map.contains(handle));
    /// ```
    #[inline]
    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.allocator.is_alive(handle)
    }

    /// Returns the number of values currently stored in the map.
    ///
    /// This counts only live entries, not removed slots.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Resource;
    ///
    /// let mut map: HandleMap<Resource, i32> = HandleMap::new();
    /// assert_eq!(map.len(), 0);
    ///
    /// let h1 = map.insert(10);
    /// let h2 = map.insert(20);
    /// assert_eq!(map.len(), 2);
    ///
    /// map.remove(h1);
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.allocator.len()
    }

    /// Returns `true` if the map contains no values.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Item;
    ///
    /// let mut map: HandleMap<Item, String> = HandleMap::new();
    /// assert!(map.is_empty());
    ///
    /// let handle = map.insert("item".to_string());
    /// assert!(!map.is_empty());
    ///
    /// map.remove(handle);
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.allocator.is_empty()
    }

    /// Returns the total capacity of the map.
    ///
    /// This is the number of slots allocated, including both live and removed entries.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Data;
    ///
    /// let mut map: HandleMap<Data, i32> = HandleMap::new();
    /// let h1 = map.insert(1);
    /// let h2 = map.insert(2);
    ///
    /// assert_eq!(map.capacity(), 2);
    ///
    /// map.remove(h1);
    /// // Capacity unchanged after removal
    /// assert_eq!(map.capacity(), 2);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.allocator.capacity()
    }

    /// Clears all values from the map, invalidating all handles.
    ///
    /// After this operation, all previously returned handles become stale.
    /// The capacity is retained but `len()` becomes 0.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Object;
    ///
    /// let mut map: HandleMap<Object, i32> = HandleMap::new();
    ///
    /// let h1 = map.insert(100);
    /// let h2 = map.insert(200);
    /// let h3 = map.insert(300);
    ///
    /// map.clear();
    ///
    /// assert!(map.is_empty());
    /// assert!(!map.contains(h1));
    /// assert!(!map.contains(h2));
    /// assert!(!map.contains(h3));
    ///
    /// // Capacity retained
    /// assert_eq!(map.capacity(), 3);
    /// ```
    pub fn clear(&mut self) {
        self.allocator.clear();

        // Clear all values but retain the vec capacity
        for slot in &mut self.values {
            *slot = None;
        }
    }

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// # Arguments
    ///
    /// * `additional` - The number of additional entries to reserve space for
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Component;
    ///
    /// let mut map: HandleMap<Component, i32> = HandleMap::new();
    /// map.reserve(100);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.values.reserve(additional);
    }

    /// Shrinks the capacity of the map as much as possible.
    ///
    /// This reduces memory usage by releasing excess capacity in internal
    /// data structures.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Item;
    ///
    /// let mut map: HandleMap<Item, i32> = HandleMap::with_capacity(100);
    ///
    /// for i in 0..10 {
    ///     map.insert(i);
    /// }
    ///
    /// map.shrink_to_fit();
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.allocator.shrink_to_fit();
        self.values.shrink_to_fit();
    }
}

impl<T, V> Default for HandleMap<T, V> {
    /// Creates an empty `HandleMap` (same as `new()`).
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, V: std::fmt::Debug> std::fmt::Debug for HandleMap<T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = std::any::type_name::<T>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        f.debug_struct(&format!("HandleMap<{}, ...>", short_name))
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .finish()
    }
}

// =============================================================================
// HandleMap Iterators
// =============================================================================

/// An iterator over handle-value pairs in a [`HandleMap`].
///
/// This struct is created by the [`iter`](HandleMap::iter) method on `HandleMap`.
/// It yields `(Handle<T>, &V)` pairs for all live entries.
///
/// # Iteration Order
///
/// Iteration order is based on the internal storage order (by slot index),
/// which corresponds to allocation order for slots that haven't been recycled.
/// Do not rely on any specific ordering as it may change with insertions and removals.
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// struct Texture;
///
/// let mut map: HandleMap<Texture, &str> = HandleMap::new();
/// map.insert("first");
/// map.insert("second");
/// map.insert("third");
///
/// for (handle, value) in map.iter() {
///     println!("Handle {:?} => {}", handle, value);
/// }
/// ```
pub struct HandleMapIter<'a, T, V> {
    /// Reference to the allocator for generation checking.
    allocator: &'a HandleAllocator<T>,

    /// Iterator over the values vector with indices.
    values_iter: std::iter::Enumerate<std::slice::Iter<'a, Option<V>>>,
}

impl<'a, T, V> Iterator for HandleMapIter<'a, T, V> {
    type Item = (Handle<T>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate through values, skipping None entries
        loop {
            match self.values_iter.next() {
                Some((index, Some(value))) => {
                    // Reconstruct the handle from index and current generation
                    let index = index as u32;
                    // Get the current generation for this slot
                    if let Some(&generation) = self.allocator.generations.get(index as usize) {
                        let handle = Handle::new(index, generation);
                        // Only yield if the handle is alive (matches current generation)
                        if self.allocator.is_alive(handle) {
                            return Some((handle, value));
                        }
                    }
                    // Generation mismatch or out of bounds - skip this entry
                    // This shouldn't happen if the map is consistent, but handle gracefully
                }
                Some((_, None)) => {
                    // Empty slot, continue to next
                    continue;
                }
                None => {
                    // End of iteration
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Lower bound is 0 (all remaining could be None)
        // Upper bound is remaining elements in the iterator
        let (_, upper) = self.values_iter.size_hint();
        (0, upper)
    }
}

/// A mutable iterator over handle-value pairs in a [`HandleMap`].
///
/// This struct is created by the [`iter_mut`](HandleMap::iter_mut) method on `HandleMap`.
/// It yields `(Handle<T>, &mut V)` pairs for all live entries.
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// struct Counter;
///
/// let mut map: HandleMap<Counter, i32> = HandleMap::new();
/// map.insert(1);
/// map.insert(2);
/// map.insert(3);
///
/// for (handle, value) in map.iter_mut() {
///     *value *= 2;
/// }
///
/// // All values are now doubled
/// ```
pub struct HandleMapIterMut<'a, T, V> {
    /// Pointer to the allocator for generation checking.
    /// We use a raw pointer because we can't hold a reference while iterating mutably.
    allocator_ptr: *const HandleAllocator<T>,

    /// Iterator over the values vector with indices.
    values_iter: std::iter::Enumerate<std::slice::IterMut<'a, Option<V>>>,

    /// Phantom marker for lifetime.
    _marker: PhantomData<&'a HandleAllocator<T>>,
}

impl<'a, T, V> Iterator for HandleMapIterMut<'a, T, V> {
    type Item = (Handle<T>, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate through values, skipping None entries
        loop {
            match self.values_iter.next() {
                Some((index, Some(value))) => {
                    // Reconstruct the handle from index and current generation
                    let index = index as u32;
                    // Safety: allocator_ptr is valid for the lifetime 'a
                    let allocator = unsafe { &*self.allocator_ptr };
                    if let Some(&generation) = allocator.generations.get(index as usize) {
                        let handle = Handle::new(index, generation);
                        if allocator.is_alive(handle) {
                            return Some((handle, value));
                        }
                    }
                }
                Some((_, None)) => {
                    continue;
                }
                None => {
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.values_iter.size_hint();
        (0, upper)
    }
}

/// An iterator over handles in a [`HandleMap`].
///
/// This struct is created by the [`handles`](HandleMap::handles) method on `HandleMap`.
/// It yields `Handle<T>` for all live entries.
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// struct Entity;
///
/// let mut map: HandleMap<Entity, String> = HandleMap::new();
/// let h1 = map.insert("entity1".to_string());
/// let h2 = map.insert("entity2".to_string());
///
/// let handles: Vec<_> = map.handles().collect();
/// assert!(handles.contains(&h1));
/// assert!(handles.contains(&h2));
/// ```
pub struct HandleMapHandles<'a, T, V> {
    /// The underlying iterator.
    inner: HandleMapIter<'a, T, V>,
}

impl<'a, T, V> Iterator for HandleMapHandles<'a, T, V> {
    type Item = Handle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(handle, _)| handle)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// An iterator over values in a [`HandleMap`].
///
/// This struct is created by the [`values`](HandleMap::values) method on `HandleMap`.
/// It yields `&V` for all live entries.
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// struct Score;
///
/// let mut map: HandleMap<Score, i32> = HandleMap::new();
/// map.insert(100);
/// map.insert(200);
/// map.insert(300);
///
/// let sum: i32 = map.values().sum();
/// assert_eq!(sum, 600);
/// ```
pub struct HandleMapValues<'a, T, V> {
    /// The underlying iterator.
    inner: HandleMapIter<'a, T, V>,
}

impl<'a, T, V> Iterator for HandleMapValues<'a, T, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over values in a [`HandleMap`].
///
/// This struct is created by the [`values_mut`](HandleMap::values_mut) method on `HandleMap`.
/// It yields `&mut V` for all live entries.
pub struct HandleMapValuesMut<'a, T, V> {
    /// The underlying iterator.
    inner: HandleMapIterMut<'a, T, V>,
}

impl<'a, T, V> Iterator for HandleMapValuesMut<'a, T, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T, V> HandleMap<T, V> {
    /// Returns an iterator over handle-value pairs.
    ///
    /// The iterator yields `(Handle<T>, &V)` for each live entry in the map.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Item;
    ///
    /// let mut map: HandleMap<Item, &str> = HandleMap::new();
    /// map.insert("a");
    /// map.insert("b");
    /// map.insert("c");
    ///
    /// let count = map.iter().count();
    /// assert_eq!(count, 3);
    /// ```
    #[inline]
    pub fn iter(&self) -> HandleMapIter<'_, T, V> {
        HandleMapIter {
            allocator: &self.allocator,
            values_iter: self.values.iter().enumerate(),
        }
    }

    /// Returns a mutable iterator over handle-value pairs.
    ///
    /// The iterator yields `(Handle<T>, &mut V)` for each live entry in the map.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Counter;
    ///
    /// let mut map: HandleMap<Counter, i32> = HandleMap::new();
    /// map.insert(1);
    /// map.insert(2);
    ///
    /// for (_, value) in map.iter_mut() {
    ///     *value *= 10;
    /// }
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> HandleMapIterMut<'_, T, V> {
        HandleMapIterMut {
            allocator_ptr: &self.allocator as *const HandleAllocator<T>,
            values_iter: self.values.iter_mut().enumerate(),
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over handles only.
    ///
    /// This is more efficient than `iter().map(|(h, _)| h)` if you only need handles.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Entity;
    ///
    /// let mut map: HandleMap<Entity, ()> = HandleMap::new();
    /// map.insert(());
    /// map.insert(());
    ///
    /// let handles: Vec<_> = map.handles().collect();
    /// assert_eq!(handles.len(), 2);
    /// ```
    #[inline]
    pub fn handles(&self) -> HandleMapHandles<'_, T, V> {
        HandleMapHandles { inner: self.iter() }
    }

    /// Returns an iterator over values only.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Score;
    ///
    /// let mut map: HandleMap<Score, i32> = HandleMap::new();
    /// map.insert(10);
    /// map.insert(20);
    /// map.insert(30);
    ///
    /// let total: i32 = map.values().sum();
    /// assert_eq!(total, 60);
    /// ```
    #[inline]
    pub fn values(&self) -> HandleMapValues<'_, T, V> {
        HandleMapValues { inner: self.iter() }
    }

    /// Returns a mutable iterator over values only.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Data;
    ///
    /// let mut map: HandleMap<Data, i32> = HandleMap::new();
    /// map.insert(1);
    /// map.insert(2);
    ///
    /// for value in map.values_mut() {
    ///     *value += 100;
    /// }
    /// ```
    #[inline]
    pub fn values_mut(&mut self) -> HandleMapValuesMut<'_, T, V> {
        HandleMapValuesMut {
            inner: self.iter_mut(),
        }
    }
}

/// Allows iterating over a `HandleMap` with a for loop.
impl<'a, T, V> IntoIterator for &'a HandleMap<T, V> {
    type Item = (Handle<T>, &'a V);
    type IntoIter = HandleMapIter<'a, T, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Allows mutably iterating over a `HandleMap` with a for loop.
impl<'a, T, V> IntoIterator for &'a mut HandleMap<T, V> {
    type Item = (Handle<T>, &'a mut V);
    type IntoIter = HandleMapIterMut<'a, T, V>;

    #[inline]
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

    /// Marker type for testing handles.
    struct TestResource;

    /// Another marker type to verify type safety.
    struct OtherResource;

    #[test]
    fn test_handle_new_and_accessors() {
        // Test that new() creates a handle with correct index and generation
        let handle: Handle<TestResource> = Handle::new(42, 7);

        assert_eq!(handle.index(), 42, "index should be 42");
        assert_eq!(handle.generation(), 7, "generation should be 7");
    }

    #[test]
    fn test_handle_invalid_constant() {
        // Test that INVALID has the expected values
        let invalid: Handle<TestResource> = Handle::INVALID;

        assert_eq!(
            invalid.index(),
            u32::MAX,
            "INVALID index should be u32::MAX"
        );
        assert_eq!(invalid.generation(), 0, "INVALID generation should be 0");
    }

    #[test]
    fn test_handle_is_valid() {
        // Test is_valid() for various handles
        let valid1: Handle<TestResource> = Handle::new(0, 1);
        let valid2: Handle<TestResource> = Handle::new(100, 50);
        let valid3: Handle<TestResource> = Handle::new(u32::MAX - 1, 1);
        let invalid: Handle<TestResource> = Handle::INVALID;

        assert!(valid1.is_valid(), "Handle(0, 1) should be valid");
        assert!(valid2.is_valid(), "Handle(100, 50) should be valid");
        assert!(valid3.is_valid(), "Handle(MAX-1, 1) should be valid");
        assert!(!invalid.is_valid(), "INVALID should not be valid");
    }

    #[test]
    fn test_handle_edge_cases() {
        // Test edge cases near INVALID values
        // Handle with MAX index but non-zero generation is valid
        let edge1: Handle<TestResource> = Handle::new(u32::MAX, 1);
        assert!(edge1.is_valid(), "Handle(MAX, 1) should be valid");

        // Handle with zero generation but non-MAX index is valid
        let edge2: Handle<TestResource> = Handle::new(0, 0);
        assert!(edge2.is_valid(), "Handle(0, 0) should be valid");

        // Handle with index MAX-1 and generation 0 is valid
        let edge3: Handle<TestResource> = Handle::new(u32::MAX - 1, 0);
        assert!(edge3.is_valid(), "Handle(MAX-1, 0) should be valid");
    }

    #[test]
    fn test_handle_size_and_alignment() {
        // Verify FFI-compatible size and alignment
        use std::mem::{align_of, size_of};

        assert_eq!(
            size_of::<Handle<TestResource>>(),
            8,
            "Handle should be 8 bytes"
        );
        assert_eq!(
            align_of::<Handle<TestResource>>(),
            4,
            "Handle should have 4-byte alignment"
        );

        // Different type parameters shouldn't affect size
        assert_eq!(
            size_of::<Handle<TestResource>>(),
            size_of::<Handle<OtherResource>>(),
            "Handle size should not depend on type parameter"
        );
    }

    // =========================================================================
    // Trait Tests (Step 1.2.2)
    // =========================================================================

    #[test]
    fn test_handle_clone() {
        // Test Clone implementation
        let original: Handle<TestResource> = Handle::new(42, 7);
        let cloned = original.clone();

        assert_eq!(original.index(), cloned.index());
        assert_eq!(original.generation(), cloned.generation());
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_handle_copy() {
        // Test Copy implementation (handles should be trivially copyable)
        let original: Handle<TestResource> = Handle::new(42, 7);

        // Copy by assignment
        let copied = original;

        // Original is still usable (proves Copy, not just Move)
        assert_eq!(original.index(), 42);
        assert_eq!(copied.index(), 42);
        assert_eq!(original, copied);
    }

    #[test]
    fn test_handle_debug_format() {
        // Test Debug formatting: Handle<TypeName>(index:gen)
        let handle: Handle<TestResource> = Handle::new(42, 7);
        let debug_str = format!("{:?}", handle);

        // Should contain type name, index, and generation
        assert!(
            debug_str.contains("TestResource"),
            "Debug should contain type name, got: {}",
            debug_str
        );
        assert!(
            debug_str.contains("42"),
            "Debug should contain index, got: {}",
            debug_str
        );
        assert!(
            debug_str.contains("7"),
            "Debug should contain generation, got: {}",
            debug_str
        );
        // Check format: Handle<TypeName>(index:gen)
        assert!(
            debug_str.starts_with("Handle<"),
            "Debug should start with 'Handle<', got: {}",
            debug_str
        );
    }

    #[test]
    fn test_handle_debug_invalid() {
        // Test Debug formatting for INVALID handle
        let invalid: Handle<TestResource> = Handle::INVALID;
        let debug_str = format!("{:?}", invalid);

        assert!(
            debug_str.contains(&u32::MAX.to_string()),
            "Debug of INVALID should show MAX index, got: {}",
            debug_str
        );
        assert!(
            debug_str.contains(":0)"),
            "Debug of INVALID should show generation 0, got: {}",
            debug_str
        );
    }

    #[test]
    fn test_handle_partial_eq() {
        // Test PartialEq: must compare both index AND generation
        let h1: Handle<TestResource> = Handle::new(1, 1);
        let h2: Handle<TestResource> = Handle::new(1, 1);
        let h3: Handle<TestResource> = Handle::new(1, 2); // same index, different gen
        let h4: Handle<TestResource> = Handle::new(2, 1); // different index, same gen

        assert_eq!(h1, h2, "Same index and gen should be equal");
        assert_ne!(h1, h3, "Same index, different gen should not be equal");
        assert_ne!(h1, h4, "Different index, same gen should not be equal");
    }

    #[test]
    fn test_handle_eq_reflexive_symmetric_transitive() {
        // Test Eq properties
        let a: Handle<TestResource> = Handle::new(5, 3);
        let b: Handle<TestResource> = Handle::new(5, 3);
        let c: Handle<TestResource> = Handle::new(5, 3);

        // Reflexive: a == a
        assert_eq!(a, a);

        // Symmetric: a == b implies b == a
        assert_eq!(a, b);
        assert_eq!(b, a);

        // Transitive: a == b && b == c implies a == c
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a, c);
    }

    #[test]
    fn test_handle_hash_consistency() {
        use std::collections::hash_map::DefaultHasher;

        // Test that Hash is consistent with PartialEq
        let h1: Handle<TestResource> = Handle::new(42, 7);
        let h2: Handle<TestResource> = Handle::new(42, 7);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        h1.hash(&mut hasher1);
        h2.hash(&mut hasher2);

        let hash1 = hasher1.finish();
        let hash2 = hasher2.finish();

        // Equal handles must have equal hashes
        assert_eq!(hash1, hash2, "Equal handles must have equal hashes");
    }

    #[test]
    fn test_handle_hash_in_hashmap() {
        use std::collections::HashMap;

        // Test that handles work correctly as HashMap keys
        let mut map: HashMap<Handle<TestResource>, &str> = HashMap::new();

        let h1: Handle<TestResource> = Handle::new(1, 1);
        let h2: Handle<TestResource> = Handle::new(2, 1);
        let h3: Handle<TestResource> = Handle::new(1, 2); // same index as h1, different gen

        map.insert(h1, "first");
        map.insert(h2, "second");
        map.insert(h3, "third");

        assert_eq!(map.get(&h1), Some(&"first"));
        assert_eq!(map.get(&h2), Some(&"second"));
        assert_eq!(map.get(&h3), Some(&"third"));
        assert_eq!(map.len(), 3, "All three handles should be distinct keys");

        // Lookup with equivalent handle
        let h1_copy: Handle<TestResource> = Handle::new(1, 1);
        assert_eq!(map.get(&h1_copy), Some(&"first"));
    }

    #[test]
    fn test_handle_default() {
        // Test Default returns INVALID
        let default_handle: Handle<TestResource> = Handle::default();

        assert!(!default_handle.is_valid());
        assert_eq!(default_handle, Handle::INVALID);
        assert_eq!(default_handle.index(), u32::MAX);
        assert_eq!(default_handle.generation(), 0);
    }

    #[test]
    fn test_handle_to_u64_and_from_u64() {
        // Test pack/unpack round-trip
        let original: Handle<TestResource> = Handle::new(42, 7);
        let packed = original.to_u64();
        let unpacked: Handle<TestResource> = Handle::from_u64(packed);

        assert_eq!(original, unpacked);
        assert_eq!(original.index(), unpacked.index());
        assert_eq!(original.generation(), unpacked.generation());
    }

    #[test]
    fn test_handle_u64_pack_format() {
        // Verify the packing format: upper 32 bits = generation, lower 32 = index
        let handle: Handle<TestResource> = Handle::new(0x12345678, 0xABCDEF01);
        let packed = handle.to_u64();

        // Expected: 0xABCDEF01_12345678
        let expected: u64 = 0xABCDEF01_12345678;
        assert_eq!(packed, expected, "Pack format should be gen:index");

        // Verify we can extract the parts
        let lower = packed as u32;
        let upper = (packed >> 32) as u32;
        assert_eq!(lower, 0x12345678, "Lower 32 bits should be index");
        assert_eq!(upper, 0xABCDEF01, "Upper 32 bits should be generation");
    }

    #[test]
    fn test_handle_from_trait_u64() {
        // Test From<Handle<T>> for u64
        let handle: Handle<TestResource> = Handle::new(100, 50);
        let packed: u64 = handle.into();

        assert_eq!(packed, handle.to_u64());
    }

    #[test]
    fn test_handle_into_trait_from_u64() {
        // Test From<u64> for Handle<T>
        let packed: u64 = (7u64 << 32) | 42u64;
        let handle: Handle<TestResource> = packed.into();

        assert_eq!(handle.index(), 42);
        assert_eq!(handle.generation(), 7);
    }

    #[test]
    fn test_handle_u64_edge_cases() {
        // Test edge cases for pack/unpack

        // Zero handle
        let zero: Handle<TestResource> = Handle::new(0, 0);
        assert_eq!(zero.to_u64(), 0u64);
        assert_eq!(Handle::<TestResource>::from_u64(0), zero);

        // Max values
        let max: Handle<TestResource> = Handle::new(u32::MAX, u32::MAX);
        let packed = max.to_u64();
        assert_eq!(packed, u64::MAX);
        assert_eq!(Handle::<TestResource>::from_u64(u64::MAX), max);

        // INVALID handle
        let invalid: Handle<TestResource> = Handle::INVALID;
        let invalid_packed = invalid.to_u64();
        let invalid_unpacked: Handle<TestResource> = Handle::from_u64(invalid_packed);
        assert_eq!(invalid, invalid_unpacked);
        assert!(!invalid_unpacked.is_valid());
    }

    #[test]
    fn test_handle_different_types_not_comparable() {
        // Verify that Handle<A> and Handle<B> are different types
        // This is a compile-time check - if this compiles, the types are distinct
        let _h1: Handle<TestResource> = Handle::new(1, 1);
        let _h2: Handle<OtherResource> = Handle::new(1, 1);

        // These have the same index/generation but are different types
        // We can't directly compare them (which is correct behavior)
        // This test just verifies the type system works
        fn assert_types_differ<A, B>() {}
        assert_types_differ::<Handle<TestResource>, Handle<OtherResource>>();
    }

    // =========================================================================
    // HandleAllocator Tests (Step 1.2.3)
    // =========================================================================

    #[test]
    fn test_allocator_new() {
        // Test that new() creates an empty allocator
        let allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        assert_eq!(allocator.len(), 0, "New allocator should have len 0");
        assert_eq!(
            allocator.capacity(),
            0,
            "New allocator should have capacity 0"
        );
        assert!(allocator.is_empty(), "New allocator should be empty");
    }

    #[test]
    fn test_allocator_default() {
        // Test that Default creates an empty allocator (same as new)
        let allocator: HandleAllocator<TestResource> = HandleAllocator::default();

        assert_eq!(allocator.len(), 0);
        assert!(allocator.is_empty());
    }

    #[test]
    fn test_allocator_allocate_single() {
        // Test allocating a single handle
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        let handle = allocator.allocate();

        assert!(handle.is_valid(), "Allocated handle should be valid");
        assert_eq!(handle.index(), 0, "First allocation should have index 0");
        assert_eq!(
            handle.generation(),
            1,
            "First allocation should have generation 1"
        );
        assert_eq!(allocator.len(), 1, "Allocator should have 1 handle");
        assert_eq!(allocator.capacity(), 1, "Capacity should be 1");
        assert!(!allocator.is_empty(), "Allocator should not be empty");
    }

    #[test]
    fn test_allocator_allocate_multiple() {
        // Test allocating multiple handles
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        let h1 = allocator.allocate();
        let h2 = allocator.allocate();
        let h3 = allocator.allocate();

        // All should be valid and unique
        assert!(h1.is_valid());
        assert!(h2.is_valid());
        assert!(h3.is_valid());

        assert_ne!(h1, h2, "Handles should be unique");
        assert_ne!(h2, h3, "Handles should be unique");
        assert_ne!(h1, h3, "Handles should be unique");

        // Indices should be sequential
        assert_eq!(h1.index(), 0);
        assert_eq!(h2.index(), 1);
        assert_eq!(h3.index(), 2);

        // All should have generation 1
        assert_eq!(h1.generation(), 1);
        assert_eq!(h2.generation(), 1);
        assert_eq!(h3.generation(), 1);

        assert_eq!(allocator.len(), 3);
        assert_eq!(allocator.capacity(), 3);
    }

    #[test]
    fn test_allocator_is_alive() {
        // Test is_alive for various scenarios
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        // INVALID handle should never be alive
        assert!(
            !allocator.is_alive(Handle::INVALID),
            "INVALID should not be alive"
        );

        // Allocated handle should be alive
        let handle = allocator.allocate();
        assert!(
            allocator.is_alive(handle),
            "Allocated handle should be alive"
        );

        // Fabricated handle with wrong index should not be alive
        let fake = Handle::<TestResource>::new(100, 1);
        assert!(
            !allocator.is_alive(fake),
            "Handle with out-of-bounds index should not be alive"
        );

        // Fabricated handle with wrong generation should not be alive
        let wrong_gen = Handle::<TestResource>::new(0, 99);
        assert!(
            !allocator.is_alive(wrong_gen),
            "Handle with wrong generation should not be alive"
        );
    }

    #[test]
    fn test_allocator_deallocate() {
        // Test basic deallocation
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        let handle = allocator.allocate();
        assert!(allocator.is_alive(handle));

        // Deallocate should succeed and return true
        assert!(allocator.deallocate(handle), "Deallocation should succeed");

        // Handle should no longer be alive
        assert!(
            !allocator.is_alive(handle),
            "Deallocated handle should not be alive"
        );

        // Allocator should be empty
        assert_eq!(allocator.len(), 0, "Allocator should have 0 handles");
        assert_eq!(allocator.capacity(), 1, "Capacity should still be 1");
    }

    #[test]
    fn test_allocator_deallocate_invalid() {
        // Test deallocating invalid handles
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        // Deallocating INVALID should fail
        assert!(
            !allocator.deallocate(Handle::INVALID),
            "Deallocating INVALID should fail"
        );

        // Deallocating out-of-bounds handle should fail
        let fake = Handle::<TestResource>::new(100, 1);
        assert!(
            !allocator.deallocate(fake),
            "Deallocating out-of-bounds handle should fail"
        );

        // Allocate then try to deallocate with wrong generation
        let handle = allocator.allocate();
        let wrong_gen = Handle::<TestResource>::new(handle.index(), handle.generation() + 1);
        assert!(
            !allocator.deallocate(wrong_gen),
            "Deallocating with wrong generation should fail"
        );

        // Original handle should still be alive
        assert!(allocator.is_alive(handle));
    }

    #[test]
    fn test_allocator_double_deallocate() {
        // Test that double deallocation fails gracefully
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        let handle = allocator.allocate();

        // First deallocation succeeds
        assert!(allocator.deallocate(handle));

        // Second deallocation fails (generation mismatch)
        assert!(
            !allocator.deallocate(handle),
            "Double deallocation should fail"
        );
    }

    #[test]
    fn test_allocator_slot_reuse() {
        // Test that deallocated slots are reused
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        // Allocate and deallocate
        let h1 = allocator.allocate();
        assert_eq!(h1.index(), 0);
        assert_eq!(h1.generation(), 1);

        allocator.deallocate(h1);

        // Allocate again - should reuse slot 0 with incremented generation
        let h2 = allocator.allocate();
        assert_eq!(h2.index(), 0, "Should reuse slot 0");
        assert_eq!(h2.generation(), 2, "Generation should be incremented");

        // Capacity should still be 1 (slot was reused)
        assert_eq!(allocator.capacity(), 1);

        // h1 should be dead, h2 should be alive
        assert!(!allocator.is_alive(h1), "Old handle should be dead");
        assert!(allocator.is_alive(h2), "New handle should be alive");
    }

    #[test]
    fn test_allocator_generation_prevents_aba() {
        // Test that generational indices prevent ABA problem
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        // Allocate slot 0
        let h1 = allocator.allocate();
        assert!(allocator.is_alive(h1));

        // Deallocate slot 0
        allocator.deallocate(h1);

        // Allocate slot 0 again (different generation)
        let h2 = allocator.allocate();
        assert_eq!(h1.index(), h2.index(), "Same slot should be reused");
        assert_ne!(
            h1.generation(),
            h2.generation(),
            "Generations should differ"
        );

        // h1 is stale (references old generation)
        assert!(!allocator.is_alive(h1), "Old handle should be stale");
        assert!(allocator.is_alive(h2), "New handle should be alive");

        // Attempting to use h1 for operations should fail
        assert!(
            !allocator.deallocate(h1),
            "Deallocation with stale handle should fail"
        );
    }

    #[test]
    fn test_allocator_generation_wrapping() {
        // Test that generation increments correctly on reuse
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        // Allocate slot 0 with generation 1
        let h1 = allocator.allocate();
        assert_eq!(h1.index(), 0);
        assert_eq!(h1.generation(), 1);

        // Deallocate and re-allocate multiple times to increment generation
        allocator.deallocate(h1);
        let h2 = allocator.allocate();
        assert_eq!(h2.index(), 0, "Should reuse slot 0");
        assert_eq!(h2.generation(), 2, "Generation should be 2");

        allocator.deallocate(h2);
        let h3 = allocator.allocate();
        assert_eq!(h3.index(), 0, "Should reuse slot 0");
        assert_eq!(h3.generation(), 3, "Generation should be 3");

        allocator.deallocate(h3);
        let h4 = allocator.allocate();
        assert_eq!(h4.index(), 0, "Should reuse slot 0");
        assert_eq!(h4.generation(), 4, "Generation should be 4");

        // Proper test: deallocate then allocate sequentially
        allocator.deallocate(h4);
        for expected_gen in 5..=10 {
            let h = allocator.allocate();
            assert_eq!(h.index(), 0, "Should reuse slot 0");
            assert_eq!(h.generation(), expected_gen, "Generation should increment");
            allocator.deallocate(h);
        }
    }

    #[test]
    fn test_allocator_len_and_capacity() {
        // Test len() and capacity() behavior
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        // Empty state
        assert_eq!(allocator.len(), 0);
        assert_eq!(allocator.capacity(), 0);

        // Allocate some handles
        let h1 = allocator.allocate();
        let h2 = allocator.allocate();
        let h3 = allocator.allocate();

        assert_eq!(allocator.len(), 3);
        assert_eq!(allocator.capacity(), 3);

        // Deallocate one
        allocator.deallocate(h2);

        assert_eq!(allocator.len(), 2, "len should decrease on deallocation");
        assert_eq!(
            allocator.capacity(),
            3,
            "capacity should not change on deallocation"
        );

        // Allocate one more (should reuse h2's slot)
        let h4 = allocator.allocate();

        assert_eq!(allocator.len(), 3);
        assert_eq!(
            allocator.capacity(),
            3,
            "capacity should not increase when reusing"
        );

        // Allocate another (new slot)
        let _h5 = allocator.allocate();

        assert_eq!(allocator.len(), 4);
        assert_eq!(allocator.capacity(), 4);

        // Deallocate all
        allocator.deallocate(h1);
        allocator.deallocate(h4);
        allocator.deallocate(h3);
        let h5_deallocate = allocator.allocate(); // h5 was moved, allocate fresh for test
        allocator.deallocate(h5_deallocate);

        // Actually, let's restart to make this clearer
        let mut allocator2: HandleAllocator<TestResource> = HandleAllocator::new();

        let handles: Vec<_> = (0..5).map(|_| allocator2.allocate()).collect();
        assert_eq!(allocator2.len(), 5);
        assert_eq!(allocator2.capacity(), 5);

        for h in &handles {
            allocator2.deallocate(*h);
        }

        assert_eq!(allocator2.len(), 0, "All handles deallocated");
        assert_eq!(
            allocator2.capacity(),
            5,
            "Capacity unchanged after deallocation"
        );
        assert!(allocator2.is_empty());
    }

    #[test]
    fn test_allocator_debug_format() {
        // Test Debug formatting
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();
        allocator.allocate();
        allocator.allocate();
        let h = allocator.allocate();
        allocator.deallocate(h);

        let debug_str = format!("{:?}", allocator);

        assert!(
            debug_str.contains("HandleAllocator"),
            "Debug should contain type name"
        );
        assert!(debug_str.contains("len"), "Debug should show len");
        assert!(debug_str.contains("capacity"), "Debug should show capacity");
    }

    // =========================================================================
    // Capacity Management Tests (Step 1.2.4)
    // =========================================================================

    #[test]
    fn test_allocator_with_capacity() {
        // Test with_capacity creates allocator with reserved space
        let allocator: HandleAllocator<TestResource> = HandleAllocator::with_capacity(100);

        assert_eq!(
            allocator.len(),
            0,
            "with_capacity should not allocate handles"
        );
        assert!(
            allocator.is_empty(),
            "with_capacity should leave allocator empty"
        );
        assert_eq!(
            allocator.capacity(),
            0,
            "capacity() reports active slots, not reserved"
        );

        // Verify we can allocate up to capacity without reallocation
        // (Can't directly test Vec capacity, but we can verify behavior)
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::with_capacity(100);
        for _ in 0..100 {
            allocator.allocate();
        }
        assert_eq!(allocator.len(), 100);
        assert_eq!(allocator.capacity(), 100);
    }

    #[test]
    fn test_allocator_with_capacity_zero() {
        // Test with_capacity(0) is equivalent to new()
        let allocator: HandleAllocator<TestResource> = HandleAllocator::with_capacity(0);

        assert_eq!(allocator.len(), 0);
        assert!(allocator.is_empty());
    }

    #[test]
    fn test_allocator_clear_basic() {
        // Test basic clear functionality
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        let h1 = allocator.allocate();
        let h2 = allocator.allocate();
        let h3 = allocator.allocate();

        assert_eq!(allocator.len(), 3);
        assert!(allocator.is_alive(h1));
        assert!(allocator.is_alive(h2));
        assert!(allocator.is_alive(h3));

        allocator.clear();

        // All handles should be invalid
        assert_eq!(allocator.len(), 0);
        assert!(allocator.is_empty());
        assert!(!allocator.is_alive(h1));
        assert!(!allocator.is_alive(h2));
        assert!(!allocator.is_alive(h3));

        // Capacity should be retained
        assert_eq!(allocator.capacity(), 3);
    }

    #[test]
    fn test_allocator_clear_and_reallocate() {
        // Test that clear increments generations properly
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        let h1 = allocator.allocate();
        assert_eq!(h1.generation(), 1);

        allocator.clear();

        // New allocation should have incremented generation
        let h2 = allocator.allocate();
        assert_eq!(h2.index(), h1.index(), "Should reuse same slot");
        assert_eq!(
            h2.generation(),
            2,
            "Generation should be incremented after clear"
        );

        // Old handle still not alive
        assert!(!allocator.is_alive(h1));
        assert!(allocator.is_alive(h2));
    }

    #[test]
    fn test_allocator_clear_empty() {
        // Test clearing an empty allocator
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        allocator.clear();

        assert_eq!(allocator.len(), 0);
        assert!(allocator.is_empty());
        assert_eq!(allocator.capacity(), 0);
    }

    #[test]
    fn test_allocator_clear_with_some_deallocated() {
        // Test clear when some handles are already deallocated
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        let h1 = allocator.allocate();
        let h2 = allocator.allocate();
        let h3 = allocator.allocate();

        // Deallocate middle one
        allocator.deallocate(h2);

        assert_eq!(allocator.len(), 2);

        allocator.clear();

        // All should be invalid
        assert_eq!(allocator.len(), 0);
        assert!(!allocator.is_alive(h1));
        assert!(!allocator.is_alive(h2)); // Already was invalid
        assert!(!allocator.is_alive(h3));

        // All slots should be in free list
        assert_eq!(allocator.capacity(), 3);
    }

    #[test]
    fn test_allocator_shrink_to_fit() {
        // Test shrink_to_fit reduces free list memory
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        // Allocate many handles
        let handles: Vec<_> = (0..100).map(|_| allocator.allocate()).collect();

        // Deallocate most of them
        for h in handles.iter().skip(10) {
            allocator.deallocate(*h);
        }

        assert_eq!(allocator.len(), 10);

        // Shrink should work without panic
        allocator.shrink_to_fit();

        // Functionality should be preserved
        assert_eq!(allocator.len(), 10);
        assert_eq!(allocator.capacity(), 100);

        // Can still allocate (from free list)
        let new_handle = allocator.allocate();
        assert!(allocator.is_alive(new_handle));
    }

    #[test]
    fn test_allocator_shrink_to_fit_empty() {
        // Test shrink_to_fit on empty allocator
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        allocator.shrink_to_fit(); // Should not panic

        assert_eq!(allocator.len(), 0);
        assert!(allocator.is_empty());
    }

    #[test]
    fn test_allocator_stress_100k_allocations() {
        // Stress test: 100K allocations to verify performance and correctness
        const COUNT: usize = 100_000;

        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::with_capacity(COUNT);

        // Phase 1: Allocate all
        let handles: Vec<_> = (0..COUNT).map(|_| allocator.allocate()).collect();

        assert_eq!(allocator.len(), COUNT);
        assert_eq!(allocator.capacity(), COUNT);

        // Verify all are alive and unique
        for (i, h) in handles.iter().enumerate() {
            assert!(allocator.is_alive(*h), "Handle {} should be alive", i);
            assert_eq!(h.index(), i as u32, "Handle {} should have index {}", i, i);
        }

        // Phase 2: Deallocate every other handle
        for (i, h) in handles.iter().enumerate() {
            if i % 2 == 0 {
                assert!(allocator.deallocate(*h));
            }
        }

        assert_eq!(allocator.len(), COUNT / 2);

        // Phase 3: Verify deallocated handles are not alive
        for (i, h) in handles.iter().enumerate() {
            if i % 2 == 0 {
                assert!(
                    !allocator.is_alive(*h),
                    "Deallocated handle {} should not be alive",
                    i
                );
            } else {
                assert!(allocator.is_alive(*h), "Handle {} should still be alive", i);
            }
        }

        // Phase 4: Reallocate - should reuse free slots
        let new_handles: Vec<_> = (0..COUNT / 2).map(|_| allocator.allocate()).collect();

        assert_eq!(allocator.len(), COUNT);
        assert_eq!(
            allocator.capacity(),
            COUNT,
            "Capacity should not grow when reusing slots"
        );

        // Verify new handles are alive
        for (i, h) in new_handles.iter().enumerate() {
            assert!(allocator.is_alive(*h), "New handle {} should be alive", i);
        }

        // Phase 5: Clear and verify
        allocator.clear();

        assert_eq!(allocator.len(), 0);
        assert!(allocator.is_empty());
        assert_eq!(allocator.capacity(), COUNT);

        // All handles should be invalid
        for h in handles.iter().take(10) {
            assert!(!allocator.is_alive(*h));
        }
        for h in new_handles.iter().take(10) {
            assert!(!allocator.is_alive(*h));
        }

        // Can still allocate after clear
        let after_clear = allocator.allocate();
        assert!(allocator.is_alive(after_clear));
        // Generation will be at least 2 (was 1 for fresh slots, or 2 for reallocated slots)
        // After clear, all generations are incremented by 1
        assert!(
            after_clear.generation() >= 2,
            "Generation should be at least 2 after clear, got {}",
            after_clear.generation()
        );
    }

    #[test]
    fn test_allocator_clear_multiple_times() {
        // Test clearing multiple times increments generations correctly
        let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

        let h1 = allocator.allocate();
        assert_eq!(h1.generation(), 1);

        allocator.clear();
        let h2 = allocator.allocate();
        assert_eq!(h2.generation(), 2);

        allocator.clear();
        let h3 = allocator.allocate();
        assert_eq!(h3.generation(), 3);

        allocator.clear();
        let h4 = allocator.allocate();
        assert_eq!(h4.generation(), 4);

        // Only the last one is alive
        assert!(!allocator.is_alive(h1));
        assert!(!allocator.is_alive(h2));
        assert!(!allocator.is_alive(h3));
        assert!(allocator.is_alive(h4));
    }

    // =========================================================================
    // HandleMap Tests (Step 1.2.5)
    // =========================================================================

    #[test]
    fn test_handle_map_new() {
        // Test that new() creates an empty map
        let map: HandleMap<TestResource, i32> = HandleMap::new();

        assert_eq!(map.len(), 0, "New map should have len 0");
        assert!(map.is_empty(), "New map should be empty");
        assert_eq!(map.capacity(), 0, "New map should have capacity 0");
    }

    #[test]
    fn test_handle_map_default() {
        // Test that Default is same as new()
        let map: HandleMap<TestResource, String> = HandleMap::default();

        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_handle_map_with_capacity() {
        // Test with_capacity pre-allocates
        let map: HandleMap<TestResource, i32> = HandleMap::with_capacity(100);

        assert_eq!(map.len(), 0, "with_capacity should not insert values");
        assert!(map.is_empty());
    }

    #[test]
    fn test_handle_map_insert_single() {
        // Test inserting a single value
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        let handle = map.insert(42);

        assert!(handle.is_valid(), "Returned handle should be valid");
        assert_eq!(map.len(), 1, "Map should have 1 entry");
        assert!(!map.is_empty(), "Map should not be empty");
        assert_eq!(map.capacity(), 1);
    }

    #[test]
    fn test_handle_map_insert_multiple() {
        // Test inserting multiple values
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        let h1 = map.insert(10);
        let h2 = map.insert(20);
        let h3 = map.insert(30);

        // All should be valid and unique
        assert!(h1.is_valid());
        assert!(h2.is_valid());
        assert!(h3.is_valid());

        assert_ne!(h1, h2, "Handles should be unique");
        assert_ne!(h2, h3, "Handles should be unique");
        assert_ne!(h1, h3, "Handles should be unique");

        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_handle_map_get() {
        // Test retrieving values by handle
        let mut map: HandleMap<TestResource, String> = HandleMap::new();

        let handle = map.insert("hello".to_string());

        let value = map.get(handle);
        assert!(value.is_some(), "get should return Some for valid handle");
        assert_eq!(value.unwrap(), "hello");
    }

    #[test]
    fn test_handle_map_get_invalid_handle() {
        // Test get with invalid/stale handles
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        // Get with INVALID handle
        assert!(
            map.get(Handle::INVALID).is_none(),
            "get with INVALID should return None"
        );

        // Get with fabricated handle
        let fake = Handle::<TestResource>::new(100, 1);
        assert!(
            map.get(fake).is_none(),
            "get with out-of-bounds handle should return None"
        );

        // Insert and then get with wrong generation
        let handle = map.insert(42);
        let wrong_gen = Handle::<TestResource>::new(handle.index(), handle.generation() + 1);
        assert!(
            map.get(wrong_gen).is_none(),
            "get with wrong generation should return None"
        );
    }

    #[test]
    fn test_handle_map_get_mut() {
        // Test mutable access
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        let handle = map.insert(100);

        // Modify the value
        if let Some(value) = map.get_mut(handle) {
            *value = 200;
        }

        // Verify modification
        assert_eq!(map.get(handle), Some(&200));
    }

    #[test]
    fn test_handle_map_get_mut_invalid() {
        // Test get_mut with invalid handles
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        assert!(map.get_mut(Handle::INVALID).is_none());

        let handle = map.insert(42);
        map.remove(handle);
        assert!(
            map.get_mut(handle).is_none(),
            "get_mut on removed handle should return None"
        );
    }

    #[test]
    fn test_handle_map_contains() {
        // Test contains method
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        let handle = map.insert(42);

        assert!(
            map.contains(handle),
            "contains should return true for valid handle"
        );
        assert!(
            !map.contains(Handle::INVALID),
            "contains should return false for INVALID"
        );

        map.remove(handle);
        assert!(
            !map.contains(handle),
            "contains should return false after removal"
        );
    }

    #[test]
    fn test_handle_map_remove() {
        // Test removing values
        let mut map: HandleMap<TestResource, String> = HandleMap::new();

        let handle = map.insert("to_remove".to_string());
        assert_eq!(map.len(), 1);

        let removed = map.remove(handle);
        assert!(removed.is_some(), "remove should return Some");
        assert_eq!(removed.unwrap(), "to_remove");
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_handle_map_remove_returns_none_for_invalid() {
        // Test remove with invalid handles
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        // Remove INVALID
        assert!(map.remove(Handle::INVALID).is_none());

        // Remove fabricated handle
        let fake = Handle::<TestResource>::new(100, 1);
        assert!(map.remove(fake).is_none());

        // Double remove
        let handle = map.insert(42);
        assert!(map.remove(handle).is_some());
        assert!(
            map.remove(handle).is_none(),
            "Second remove should return None"
        );
    }

    #[test]
    fn test_handle_map_remove_drops_value() {
        // Test that removed values are actually dropped
        use std::cell::RefCell;
        use std::rc::Rc;

        let drop_counter = Rc::new(RefCell::new(0));

        struct DropTracker {
            counter: Rc<RefCell<i32>>,
        }

        impl Drop for DropTracker {
            fn drop(&mut self) {
                *self.counter.borrow_mut() += 1;
            }
        }

        let mut map: HandleMap<TestResource, DropTracker> = HandleMap::new();

        let handle = map.insert(DropTracker {
            counter: drop_counter.clone(),
        });

        assert_eq!(*drop_counter.borrow(), 0, "Not dropped yet");

        let removed = map.remove(handle);
        assert_eq!(*drop_counter.borrow(), 0, "Still held by removed");

        drop(removed);
        assert_eq!(
            *drop_counter.borrow(),
            1,
            "Dropped after remove result dropped"
        );
    }

    #[test]
    fn test_handle_map_slot_reuse() {
        // Test that removed slots are reused
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        let h1 = map.insert(100);
        assert_eq!(h1.index(), 0);
        assert_eq!(h1.generation(), 1);

        map.remove(h1);

        // Next insert should reuse slot 0 with incremented generation
        let h2 = map.insert(200);
        assert_eq!(h2.index(), 0, "Should reuse slot 0");
        assert_eq!(h2.generation(), 2, "Generation should be incremented");

        // Verify values
        assert!(map.get(h1).is_none(), "Old handle should be stale");
        assert_eq!(map.get(h2), Some(&200), "New handle should work");

        // Capacity should not have grown
        assert_eq!(map.capacity(), 1);
    }

    #[test]
    fn test_handle_map_generation_safety() {
        // Test that generational indices prevent ABA problem
        let mut map: HandleMap<TestResource, &str> = HandleMap::new();

        // Insert value A at slot 0
        let h_a = map.insert("A");
        assert_eq!(map.get(h_a), Some(&"A"));

        // Remove A
        map.remove(h_a);

        // Insert value B (reuses slot 0)
        let h_b = map.insert("B");
        assert_eq!(h_a.index(), h_b.index(), "Same slot reused");

        // Old handle h_a should NOT access B (generation mismatch)
        assert!(map.get(h_a).is_none(), "Stale handle should return None");
        assert_eq!(map.get(h_b), Some(&"B"), "New handle should work");
    }

    #[test]
    fn test_handle_map_clear() {
        // Test clearing the map
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        let h1 = map.insert(10);
        let h2 = map.insert(20);
        let h3 = map.insert(30);

        assert_eq!(map.len(), 3);

        map.clear();

        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        // All handles should be invalid
        assert!(!map.contains(h1));
        assert!(!map.contains(h2));
        assert!(!map.contains(h3));

        assert!(map.get(h1).is_none());
        assert!(map.get(h2).is_none());
        assert!(map.get(h3).is_none());

        // Capacity retained
        assert_eq!(map.capacity(), 3);
    }

    #[test]
    fn test_handle_map_clear_and_reinsert() {
        // Test reinserting after clear
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        let h1 = map.insert(100);
        assert_eq!(h1.generation(), 1);

        map.clear();

        let h2 = map.insert(200);
        assert_eq!(h2.index(), h1.index(), "Should reuse slot");
        assert_eq!(h2.generation(), 2, "Generation should be incremented");

        assert!(map.get(h1).is_none());
        assert_eq!(map.get(h2), Some(&200));
    }

    #[test]
    fn test_handle_map_len_and_capacity() {
        // Test len() and capacity() behavior
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 0);

        let h1 = map.insert(1);
        let h2 = map.insert(2);
        let h3 = map.insert(3);

        assert_eq!(map.len(), 3);
        assert_eq!(map.capacity(), 3);

        // Remove one
        map.remove(h2);

        assert_eq!(map.len(), 2, "len should decrease");
        assert_eq!(map.capacity(), 3, "capacity should not decrease");

        // Insert one (reuses slot)
        let _h4 = map.insert(4);

        assert_eq!(map.len(), 3);
        assert_eq!(map.capacity(), 3, "capacity unchanged when reusing");

        // Insert another (new slot)
        let _h5 = map.insert(5);

        assert_eq!(map.len(), 4);
        assert_eq!(map.capacity(), 4);

        // Clean up to verify
        map.remove(h1);
        map.remove(h3);

        assert_eq!(map.len(), 2);
        assert_eq!(map.capacity(), 4);
    }

    #[test]
    fn test_handle_map_debug() {
        // Test Debug formatting
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();
        map.insert(1);
        map.insert(2);

        let debug_str = format!("{:?}", map);

        assert!(
            debug_str.contains("HandleMap"),
            "Debug should contain type name"
        );
        assert!(debug_str.contains("len"), "Debug should show len");
        assert!(debug_str.contains("capacity"), "Debug should show capacity");
    }

    #[test]
    fn test_handle_map_reserve() {
        // Test reserve functionality
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        map.reserve(100);

        // Can insert without reallocation
        for i in 0..100 {
            map.insert(i);
        }

        assert_eq!(map.len(), 100);
    }

    #[test]
    fn test_handle_map_shrink_to_fit() {
        // Test shrink_to_fit
        let mut map: HandleMap<TestResource, i32> = HandleMap::with_capacity(100);

        for i in 0..10 {
            map.insert(i);
        }

        map.shrink_to_fit();

        // Functionality preserved
        assert_eq!(map.len(), 10);
    }

    #[test]
    fn test_handle_map_stress() {
        // Stress test with many operations
        const COUNT: usize = 10_000;

        let mut map: HandleMap<TestResource, usize> = HandleMap::with_capacity(COUNT);

        // Phase 1: Insert many
        let handles: Vec<_> = (0..COUNT).map(|i| map.insert(i)).collect();

        assert_eq!(map.len(), COUNT);

        // Verify all values
        for (i, h) in handles.iter().enumerate() {
            assert_eq!(map.get(*h), Some(&i), "Value {} should match", i);
        }

        // Phase 2: Remove half
        for h in handles.iter().step_by(2) {
            map.remove(*h);
        }

        assert_eq!(map.len(), COUNT / 2);

        // Phase 3: Verify removals
        for (i, h) in handles.iter().enumerate() {
            if i % 2 == 0 {
                assert!(map.get(*h).is_none(), "Removed value {} should be None", i);
            } else {
                assert_eq!(map.get(*h), Some(&i), "Kept value {} should exist", i);
            }
        }

        // Phase 4: Insert again (reuses slots)
        let new_handles: Vec<_> = (0..COUNT / 2).map(|i| map.insert(i + COUNT)).collect();

        assert_eq!(map.len(), COUNT);
        assert_eq!(
            map.capacity(),
            COUNT,
            "Should reuse slots, not grow capacity"
        );

        // Verify new values
        for (i, h) in new_handles.iter().enumerate() {
            assert_eq!(map.get(*h), Some(&(i + COUNT)));
        }

        // Phase 5: Clear
        map.clear();

        assert!(map.is_empty());
        assert_eq!(map.capacity(), COUNT);

        // All handles should be stale
        for h in handles.iter().take(10) {
            assert!(map.get(*h).is_none());
        }
    }

    #[test]
    fn test_handle_map_values_with_complex_types() {
        // Test with complex value types
        #[derive(Debug, Clone, PartialEq)]
        struct ComplexData {
            id: u64,
            name: String,
            values: Vec<f32>,
        }

        let mut map: HandleMap<TestResource, ComplexData> = HandleMap::new();

        let h1 = map.insert(ComplexData {
            id: 1,
            name: "first".to_string(),
            values: vec![1.0, 2.0, 3.0],
        });

        let h2 = map.insert(ComplexData {
            id: 2,
            name: "second".to_string(),
            values: vec![4.0, 5.0],
        });

        // Access and verify
        assert_eq!(map.get(h1).unwrap().id, 1);
        assert_eq!(map.get(h1).unwrap().name, "first");
        assert_eq!(map.get(h2).unwrap().values.len(), 2);

        // Modify
        if let Some(data) = map.get_mut(h1) {
            data.values.push(4.0);
        }

        assert_eq!(map.get(h1).unwrap().values.len(), 4);

        // Remove and verify
        let removed = map.remove(h1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "first");

        assert!(map.get(h1).is_none());
        assert!(map.get(h2).is_some());
    }

    // =========================================================================
    // HandleMap Iterator Tests (Step 1.2.6)
    // =========================================================================

    #[test]
    fn test_handle_map_iter_basic() {
        // Test basic iteration over handle-value pairs
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        let h1 = map.insert(10);
        let h2 = map.insert(20);
        let h3 = map.insert(30);

        // Collect all pairs
        let pairs: Vec<_> = map.iter().collect();

        assert_eq!(pairs.len(), 3, "Should iterate over 3 entries");

        // Verify all handles are present
        let handles: Vec<_> = pairs.iter().map(|(h, _)| *h).collect();
        assert!(handles.contains(&h1));
        assert!(handles.contains(&h2));
        assert!(handles.contains(&h3));

        // Verify all values are present
        let values: Vec<_> = pairs.iter().map(|(_, v)| **v).collect();
        assert!(values.contains(&10));
        assert!(values.contains(&20));
        assert!(values.contains(&30));
    }

    #[test]
    fn test_handle_map_iter_empty() {
        // Test iteration over empty map
        let map: HandleMap<TestResource, i32> = HandleMap::new();

        let pairs: Vec<_> = map.iter().collect();

        assert!(pairs.is_empty(), "Empty map should yield no items");
    }

    #[test]
    fn test_handle_map_iter_skips_removed() {
        // Test that iteration skips removed entries
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        let h1 = map.insert(10);
        let h2 = map.insert(20);
        let h3 = map.insert(30);

        // Remove the middle entry
        map.remove(h2);

        // Iterate and collect
        let pairs: Vec<_> = map.iter().collect();

        assert_eq!(pairs.len(), 2, "Should only iterate over 2 entries");

        let handles: Vec<_> = pairs.iter().map(|(h, _)| *h).collect();
        assert!(handles.contains(&h1), "Should contain h1");
        assert!(!handles.contains(&h2), "Should NOT contain removed h2");
        assert!(handles.contains(&h3), "Should contain h3");
    }

    #[test]
    fn test_handle_map_iter_mut_basic() {
        // Test mutable iteration
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        map.insert(1);
        map.insert(2);
        map.insert(3);

        // Double all values
        for (_, value) in map.iter_mut() {
            *value *= 2;
        }

        // Verify values are doubled
        let values: Vec<_> = map.values().cloned().collect();
        assert!(values.contains(&2), "1*2 should be 2");
        assert!(values.contains(&4), "2*2 should be 4");
        assert!(values.contains(&6), "3*2 should be 6");
    }

    #[test]
    fn test_handle_map_iter_mut_modifies_in_place() {
        // Test that modifications are visible via handles
        let mut map: HandleMap<TestResource, String> = HandleMap::new();

        let h1 = map.insert("hello".to_string());
        let h2 = map.insert("world".to_string());

        // Append to all values
        for (_, value) in map.iter_mut() {
            value.push_str("!");
        }

        assert_eq!(map.get(h1).unwrap(), "hello!");
        assert_eq!(map.get(h2).unwrap(), "world!");
    }

    #[test]
    fn test_handle_map_handles_iterator() {
        // Test handles() iterator
        let mut map: HandleMap<TestResource, &str> = HandleMap::new();

        let h1 = map.insert("a");
        let h2 = map.insert("b");
        let h3 = map.insert("c");

        let handles: Vec<_> = map.handles().collect();

        assert_eq!(handles.len(), 3);
        assert!(handles.contains(&h1));
        assert!(handles.contains(&h2));
        assert!(handles.contains(&h3));
    }

    #[test]
    fn test_handle_map_values_iterator() {
        // Test values() iterator
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        map.insert(100);
        map.insert(200);
        map.insert(300);

        // Use values iterator to sum
        let sum: i32 = map.values().sum();
        assert_eq!(sum, 600);

        // Collect values
        let mut values: Vec<_> = map.values().cloned().collect();
        values.sort();
        assert_eq!(values, vec![100, 200, 300]);
    }

    #[test]
    fn test_handle_map_values_mut_iterator() {
        // Test values_mut() iterator
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        map.insert(1);
        map.insert(2);
        map.insert(3);

        // Add 100 to all values
        for value in map.values_mut() {
            *value += 100;
        }

        let mut values: Vec<_> = map.values().cloned().collect();
        values.sort();
        assert_eq!(values, vec![101, 102, 103]);
    }

    #[test]
    fn test_handle_map_into_iterator_ref() {
        // Test IntoIterator for &HandleMap (for loop support)
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        map.insert(10);
        map.insert(20);

        let mut count = 0;
        for (handle, value) in &map {
            assert!(handle.is_valid());
            assert!(*value == 10 || *value == 20);
            count += 1;
        }

        assert_eq!(count, 2);
    }

    #[test]
    fn test_handle_map_into_iterator_mut_ref() {
        // Test IntoIterator for &mut HandleMap (mutable for loop support)
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        map.insert(1);
        map.insert(2);
        map.insert(3);

        // Triple all values using for loop
        for (_, value) in &mut map {
            *value *= 3;
        }

        let mut values: Vec<_> = map.values().cloned().collect();
        values.sort();
        assert_eq!(values, vec![3, 6, 9]);
    }

    #[test]
    fn test_handle_map_iter_with_gaps() {
        // Test iteration with multiple gaps from removals
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        // Insert 10 values
        let handles: Vec<_> = (0..10).map(|i| map.insert(i)).collect();

        // Remove every other one (0, 2, 4, 6, 8)
        for i in (0..10).step_by(2) {
            map.remove(handles[i]);
        }

        // Should have 5 values remaining (1, 3, 5, 7, 9)
        let remaining: Vec<_> = map.values().cloned().collect();
        assert_eq!(remaining.len(), 5);

        let mut sorted = remaining.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 3, 5, 7, 9]);
    }

    #[test]
    fn test_handle_map_iter_count() {
        // Test that iter().count() matches len()
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        for i in 0..100 {
            map.insert(i);
        }

        assert_eq!(map.iter().count(), 100);
        assert_eq!(map.iter().count(), map.len());

        // Remove some
        let handles: Vec<_> = map.handles().take(30).collect();
        for h in handles {
            map.remove(h);
        }

        assert_eq!(map.iter().count(), 70);
        assert_eq!(map.iter().count(), map.len());
    }

    #[test]
    fn test_handle_map_iter_size_hint() {
        // Test size_hint is reasonable
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        for i in 0..10 {
            map.insert(i);
        }

        let iter = map.iter();
        let (lower, upper) = iter.size_hint();

        // Lower bound is 0 (conservative)
        assert_eq!(lower, 0);

        // Upper bound should be at most the capacity
        assert!(upper.is_some());
        assert!(upper.unwrap() <= 10);
    }

    #[test]
    fn test_handle_map_iter_after_clear() {
        // Test iteration after clear
        let mut map: HandleMap<TestResource, i32> = HandleMap::new();

        map.insert(1);
        map.insert(2);
        map.insert(3);

        map.clear();

        let count = map.iter().count();
        assert_eq!(count, 0, "Iteration after clear should yield nothing");

        // Insert new values
        map.insert(100);
        map.insert(200);

        let count = map.iter().count();
        assert_eq!(count, 2, "Should iterate over new values");
    }

    #[test]
    fn test_handle_map_iter_stress() {
        // Stress test iteration with many operations
        const COUNT: usize = 1000;

        let mut map: HandleMap<TestResource, usize> = HandleMap::new();

        // Insert values
        let handles: Vec<_> = (0..COUNT).map(|i| map.insert(i)).collect();

        // Verify iteration count
        assert_eq!(map.iter().count(), COUNT);

        // Remove half
        for (i, h) in handles.iter().enumerate() {
            if i % 2 == 0 {
                map.remove(*h);
            }
        }

        // Verify iteration count
        assert_eq!(map.iter().count(), COUNT / 2);

        // Verify remaining values
        let remaining: std::collections::HashSet<_> = map.values().cloned().collect();
        for i in 0..COUNT {
            if i % 2 == 0 {
                assert!(
                    !remaining.contains(&i),
                    "Removed value {} should not be present",
                    i
                );
            } else {
                assert!(remaining.contains(&i), "Kept value {} should be present", i);
            }
        }
    }
}
