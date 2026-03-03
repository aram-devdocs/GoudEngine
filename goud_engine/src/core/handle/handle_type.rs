//! The core `Handle<T>` type and its trait implementations.

use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

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
