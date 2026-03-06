//! Generational handle allocator for asset slots.

use crate::assets::Asset;
use std::fmt;
use std::marker::PhantomData;

use super::typed::AssetHandle;

// =============================================================================
// AssetHandleAllocator
// =============================================================================

/// Allocator for asset handles with generation counting and slot reuse.
///
/// `AssetHandleAllocator` manages the allocation and deallocation of asset handles,
/// similar to [`HandleAllocator`](crate::core::handle::HandleAllocator) but specialized
/// for asset-specific use cases.
///
/// # Features
///
/// - **Generation Counting**: Prevents use-after-free by invalidating stale handles
/// - **Slot Reuse**: Deallocated slots are recycled via a free list
/// - **Type Safety**: Each allocator is generic over the asset type
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetHandleAllocator};
///
/// struct Texture;
/// impl Asset for Texture {}
///
/// let mut allocator: AssetHandleAllocator<Texture> = AssetHandleAllocator::new();
///
/// // Allocate handles
/// let h1 = allocator.allocate();
/// let h2 = allocator.allocate();
///
/// assert!(allocator.is_alive(h1));
/// assert!(allocator.is_alive(h2));
///
/// // Deallocate
/// allocator.deallocate(h1);
/// assert!(!allocator.is_alive(h1));
///
/// // Slot is reused with new generation
/// let h3 = allocator.allocate();
/// assert_ne!(h1, h3); // Different generations
/// ```
pub struct AssetHandleAllocator<A: Asset> {
    /// Generation counter for each slot.
    /// Generation starts at 1 (0 reserved for INVALID).
    generations: Vec<u32>,

    /// Free list of available slot indices.
    free_list: Vec<u32>,

    /// Phantom marker for type parameter.
    _marker: PhantomData<A>,
}

impl<A: Asset> AssetHandleAllocator<A> {
    /// Creates a new, empty allocator.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Audio;
    /// impl Asset for Audio {}
    ///
    /// let allocator: AssetHandleAllocator<Audio> = AssetHandleAllocator::new();
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

    /// Creates a new allocator with pre-allocated capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Number of slots to pre-allocate
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Mesh;
    /// impl Asset for Mesh {}
    ///
    /// let allocator: AssetHandleAllocator<Mesh> = AssetHandleAllocator::with_capacity(1000);
    /// assert!(allocator.is_empty());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            generations: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Allocates a new handle.
    ///
    /// Reuses slots from the free list when available, otherwise allocates new slots.
    ///
    /// # Returns
    ///
    /// A new, valid `AssetHandle<A>`.
    ///
    /// # Panics
    ///
    /// Panics if the number of slots exceeds `u32::MAX - 1`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Shader;
    /// impl Asset for Shader {}
    ///
    /// let mut allocator: AssetHandleAllocator<Shader> = AssetHandleAllocator::new();
    /// let handle = allocator.allocate();
    ///
    /// assert!(handle.is_valid());
    /// assert!(allocator.is_alive(handle));
    /// ```
    pub fn allocate(&mut self) -> AssetHandle<A> {
        if let Some(index) = self.free_list.pop() {
            // Reuse slot
            let generation = self.generations[index as usize];
            AssetHandle::new(index, generation)
        } else {
            // Allocate new slot
            let index = self.generations.len();
            assert!(
                index < u32::MAX as usize,
                "AssetHandleAllocator exceeded maximum capacity"
            );

            // New slots start at generation 1
            self.generations.push(1);
            AssetHandle::new(index as u32, 1)
        }
    }

    /// Deallocates a handle, making it stale.
    ///
    /// The slot's generation is incremented, invalidating any handles that
    /// reference the old generation. The slot is added to the free list.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to deallocate
    ///
    /// # Returns
    ///
    /// `true` if the handle was valid and successfully deallocated,
    /// `false` if the handle was invalid or stale.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Font;
    /// impl Asset for Font {}
    ///
    /// let mut allocator: AssetHandleAllocator<Font> = AssetHandleAllocator::new();
    /// let handle = allocator.allocate();
    ///
    /// assert!(allocator.deallocate(handle));
    /// assert!(!allocator.is_alive(handle));
    /// assert!(!allocator.deallocate(handle)); // Already deallocated
    /// ```
    pub fn deallocate(&mut self, handle: AssetHandle<A>) -> bool {
        if !handle.is_valid() {
            return false;
        }

        let index = handle.index() as usize;

        // Check bounds
        if index >= self.generations.len() {
            return false;
        }

        // Check generation matches
        if self.generations[index] != handle.generation() {
            return false;
        }

        // Increment generation (wrap to 1 if overflows to 0)
        let new_gen = self.generations[index].wrapping_add(1);
        self.generations[index] = if new_gen == 0 { 1 } else { new_gen };

        // Add to free list
        self.free_list.push(handle.index());

        true
    }

    /// Checks if a handle is still alive (not deallocated).
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to check
    ///
    /// # Returns
    ///
    /// `true` if the handle is valid and its generation matches the current slot generation.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Material;
    /// impl Asset for Material {}
    ///
    /// let mut allocator: AssetHandleAllocator<Material> = AssetHandleAllocator::new();
    /// let handle = allocator.allocate();
    ///
    /// assert!(allocator.is_alive(handle));
    ///
    /// allocator.deallocate(handle);
    /// assert!(!allocator.is_alive(handle));
    /// ```
    #[inline]
    pub fn is_alive(&self, handle: AssetHandle<A>) -> bool {
        if !handle.is_valid() {
            return false;
        }

        let index = handle.index() as usize;
        index < self.generations.len() && self.generations[index] == handle.generation()
    }

    /// Returns the number of currently allocated (alive) handles.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Sprite;
    /// impl Asset for Sprite {}
    ///
    /// let mut allocator: AssetHandleAllocator<Sprite> = AssetHandleAllocator::new();
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

    /// Returns the total capacity (number of slots).
    ///
    /// This includes both active and free slots.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.generations.len()
    }

    /// Returns `true` if no handles are currently allocated.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all allocations, invalidating all existing handles.
    ///
    /// This increments all generations and rebuilds the free list.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Animation;
    /// impl Asset for Animation {}
    ///
    /// let mut allocator: AssetHandleAllocator<Animation> = AssetHandleAllocator::new();
    ///
    /// let h1 = allocator.allocate();
    /// let h2 = allocator.allocate();
    /// assert_eq!(allocator.len(), 2);
    ///
    /// allocator.clear();
    /// assert_eq!(allocator.len(), 0);
    /// assert!(!allocator.is_alive(h1));
    /// assert!(!allocator.is_alive(h2));
    /// ```
    pub fn clear(&mut self) {
        // Increment all generations
        for gen in &mut self.generations {
            let new_gen = gen.wrapping_add(1);
            *gen = if new_gen == 0 { 1 } else { new_gen };
        }

        // Rebuild free list
        self.free_list.clear();
        self.free_list.reserve(self.generations.len());
        for i in (0..self.generations.len()).rev() {
            self.free_list.push(i as u32);
        }
    }

    /// Shrinks the free list to fit its contents.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.free_list.shrink_to_fit();
    }

    /// Returns the current generation for a slot index.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn generation_at(&self, index: u32) -> Option<u32> {
        self.generations.get(index as usize).copied()
    }
}

impl<A: Asset> Default for AssetHandleAllocator<A> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Asset> fmt::Debug for AssetHandleAllocator<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = A::asset_type_name();
        f.debug_struct(&format!("AssetHandleAllocator<{}>", type_name))
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("free_slots", &self.free_list.len())
            .finish()
    }
}
