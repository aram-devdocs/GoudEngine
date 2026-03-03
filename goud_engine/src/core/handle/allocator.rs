//! Handle allocation with generation counting and free-list recycling.

use std::marker::PhantomData;

use super::Handle;

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
    pub(crate) generations: Vec<u32>,

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
