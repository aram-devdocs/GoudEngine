//! Typed and weak asset handle types.
//!
//! Provides [`AssetHandle<A>`] and [`WeakAssetHandle<A>`] for type-safe,
//! generational references to assets.

use crate::assets::{Asset, AssetId, AssetType};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use super::untyped::UntypedAssetHandle;

// =============================================================================
// AssetHandle<A>
// =============================================================================

/// A typed handle to an asset of type `A`.
///
/// `AssetHandle` is the primary way to reference loaded assets. It provides:
///
/// - **Type Safety**: `AssetHandle<Texture>` and `AssetHandle<Audio>` are distinct types
/// - **Generation Counting**: Prevents use-after-free when assets are unloaded
/// - **FFI Compatibility**: Can be passed across language boundaries
///
/// # Handle States
///
/// An asset handle can be in several states:
/// - **Invalid**: The sentinel value `INVALID`, representing "no asset"
/// - **Valid but Loading**: Points to an asset slot where loading is in progress
/// - **Valid and Loaded**: Points to a fully loaded, usable asset
/// - **Stale**: The asset was unloaded, handle generation no longer matches
///
/// Use `AssetServer::get_state()` to check the current state of an asset.
///
/// # FFI Layout
///
/// ```text
/// Offset 0:  index      (u32, 4 bytes)
/// Offset 4:  generation (u32, 4 bytes)
/// Offset 8:  _marker    (PhantomData, 0 bytes)
/// Total:     8 bytes, alignment 4
/// ```
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetHandle};
///
/// struct Shader { /* ... */ }
/// impl Asset for Shader {}
///
/// // Create an invalid handle (default)
/// let handle: AssetHandle<Shader> = AssetHandle::INVALID;
/// assert!(!handle.is_valid());
///
/// // Create a valid handle (normally done by AssetServer)
/// let handle: AssetHandle<Shader> = AssetHandle::new(42, 1);
/// assert!(handle.is_valid());
/// assert_eq!(handle.index(), 42);
/// assert_eq!(handle.generation(), 1);
/// ```
#[repr(C)]
pub struct AssetHandle<A: Asset> {
    /// Slot index in the asset storage.
    index: u32,

    /// Generation counter for stale handle detection.
    generation: u32,

    /// Zero-sized marker for type safety.
    _marker: PhantomData<A>,
}

impl<A: Asset> AssetHandle<A> {
    /// The invalid handle constant, representing "no asset".
    ///
    /// This is the default value for `AssetHandle` and is guaranteed to never
    /// match any valid asset handle.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle};
    ///
    /// struct Mesh;
    /// impl Asset for Mesh {}
    ///
    /// let handle: AssetHandle<Mesh> = AssetHandle::INVALID;
    /// assert!(!handle.is_valid());
    /// assert_eq!(handle.index(), u32::MAX);
    /// assert_eq!(handle.generation(), 0);
    /// ```
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
        _marker: PhantomData,
    };

    /// Creates a new asset handle with the given index and generation.
    ///
    /// This is typically called by the asset system, not by user code.
    ///
    /// # Arguments
    ///
    /// * `index` - Slot index in the asset storage
    /// * `generation` - Generation counter for this slot
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle};
    ///
    /// struct Audio;
    /// impl Asset for Audio {}
    ///
    /// let handle: AssetHandle<Audio> = AssetHandle::new(10, 3);
    /// assert_eq!(handle.index(), 10);
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
    /// The index is the slot number in the asset storage array.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation component of this handle.
    ///
    /// The generation is incremented each time a slot is reused,
    /// preventing stale handles from accessing wrong assets.
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns `true` if this handle is not the `INVALID` sentinel.
    ///
    /// Note: A "valid" handle may still be stale if the asset was unloaded.
    /// Use `AssetServer::is_alive()` for definitive liveness checks.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle};
    ///
    /// struct Font;
    /// impl Asset for Font {}
    ///
    /// let valid: AssetHandle<Font> = AssetHandle::new(0, 1);
    /// assert!(valid.is_valid());
    ///
    /// let invalid: AssetHandle<Font> = AssetHandle::INVALID;
    /// assert!(!invalid.is_valid());
    /// ```
    #[inline]
    pub const fn is_valid(&self) -> bool {
        !(self.index == u32::MAX && self.generation == 0)
    }

    /// Converts this handle to a type-erased `UntypedAssetHandle`.
    ///
    /// The untyped handle preserves the asset type ID for runtime type checking.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle, AssetId};
    ///
    /// struct Texture;
    /// impl Asset for Texture {}
    ///
    /// let typed: AssetHandle<Texture> = AssetHandle::new(5, 2);
    /// let untyped = typed.untyped();
    ///
    /// assert_eq!(untyped.index(), 5);
    /// assert_eq!(untyped.generation(), 2);
    /// assert_eq!(untyped.asset_id(), AssetId::of::<Texture>());
    /// ```
    #[inline]
    pub fn untyped(&self) -> UntypedAssetHandle {
        UntypedAssetHandle::new(self.index, self.generation, AssetId::of::<A>())
    }

    /// Packs this handle into a single u64 value for FFI.
    ///
    /// Format: upper 32 bits = generation, lower 32 bits = index.
    ///
    /// Note: This does NOT preserve the asset type; use `untyped()` if you
    /// need type information across FFI boundaries.
    #[inline]
    pub const fn to_u64(&self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }

    /// Creates a handle from a packed u64 value.
    ///
    /// Format: upper 32 bits = generation, lower 32 bits = index.
    #[inline]
    pub const fn from_u64(packed: u64) -> Self {
        let index = packed as u32;
        let generation = (packed >> 32) as u32;
        Self::new(index, generation)
    }

    /// Returns the asset type ID for this handle's asset type.
    ///
    /// This is a convenience method equivalent to `AssetId::of::<A>()`.
    #[inline]
    pub fn asset_id() -> AssetId {
        AssetId::of::<A>()
    }

    /// Returns the asset type category for this handle's asset type.
    ///
    /// This is a convenience method equivalent to `A::asset_type()`.
    #[inline]
    pub fn asset_type() -> AssetType {
        A::asset_type()
    }
}

// Trait implementations for AssetHandle<A>

impl<A: Asset> Clone for AssetHandle<A> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Asset> Copy for AssetHandle<A> {}

impl<A: Asset> Default for AssetHandle<A> {
    /// Returns `AssetHandle::INVALID`.
    #[inline]
    fn default() -> Self {
        Self::INVALID
    }
}

impl<A: Asset> fmt::Debug for AssetHandle<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = A::asset_type_name();
        if self.is_valid() {
            write!(
                f,
                "AssetHandle<{}>({}:{})",
                type_name, self.index, self.generation
            )
        } else {
            write!(f, "AssetHandle<{}>(INVALID)", type_name)
        }
    }
}

impl<A: Asset> fmt::Display for AssetHandle<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            write!(f, "{}:{}", self.index, self.generation)
        } else {
            write!(f, "INVALID")
        }
    }
}

impl<A: Asset> PartialEq for AssetHandle<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<A: Asset> Eq for AssetHandle<A> {}

impl<A: Asset> Hash for AssetHandle<A> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_u64().hash(state);
    }
}

impl<A: Asset> PartialOrd for AssetHandle<A> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Asset> Ord for AssetHandle<A> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_u64().cmp(&other.to_u64())
    }
}

// Conversions
impl<A: Asset> From<AssetHandle<A>> for u64 {
    #[inline]
    fn from(handle: AssetHandle<A>) -> u64 {
        handle.to_u64()
    }
}

impl<A: Asset> From<u64> for AssetHandle<A> {
    #[inline]
    fn from(packed: u64) -> Self {
        Self::from_u64(packed)
    }
}

// =============================================================================
// WeakAssetHandle<A>
// =============================================================================

/// A weak reference to an asset that doesn't prevent unloading.
///
/// Unlike `AssetHandle`, a `WeakAssetHandle` does not contribute to the
/// reference count of an asset. This is useful for caches and lookup tables
/// where you want to reference assets without keeping them loaded.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetHandle, WeakAssetHandle};
///
/// struct Texture;
/// impl Asset for Texture {}
///
/// let strong: AssetHandle<Texture> = AssetHandle::new(1, 1);
/// let weak: WeakAssetHandle<Texture> = WeakAssetHandle::from_handle(&strong);
///
/// assert!(weak.is_valid());
/// assert_eq!(weak.index(), 1);
/// assert_eq!(weak.generation(), 1);
///
/// // Upgrade to strong handle for access (liveness must be checked separately)
/// let upgraded = weak.upgrade();
/// assert_eq!(upgraded.index(), 1);
/// ```
#[repr(C)]
pub struct WeakAssetHandle<A: Asset> {
    /// Slot index.
    index: u32,

    /// Generation counter.
    generation: u32,

    /// Zero-sized marker.
    _marker: PhantomData<A>,
}

impl<A: Asset> WeakAssetHandle<A> {
    /// The invalid weak handle constant.
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
        _marker: PhantomData,
    };

    /// Creates a weak handle from a strong handle.
    #[inline]
    pub fn from_handle(handle: &AssetHandle<A>) -> Self {
        Self {
            index: handle.index,
            generation: handle.generation,
            _marker: PhantomData,
        }
    }

    /// Creates a new weak handle with the given index and generation.
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// Returns the index component.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation component.
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns `true` if not the INVALID sentinel.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        !(self.index == u32::MAX && self.generation == 0)
    }

    /// Upgrades to a strong handle.
    ///
    /// Note: This does NOT check if the asset is still alive. Use
    /// `AssetServer::is_alive()` to check liveness before using the handle.
    #[inline]
    pub fn upgrade(&self) -> AssetHandle<A> {
        AssetHandle::new(self.index, self.generation)
    }
}

impl<A: Asset> Clone for WeakAssetHandle<A> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Asset> Copy for WeakAssetHandle<A> {}

impl<A: Asset> Default for WeakAssetHandle<A> {
    #[inline]
    fn default() -> Self {
        Self::INVALID
    }
}

impl<A: Asset> fmt::Debug for WeakAssetHandle<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = A::asset_type_name();
        if self.is_valid() {
            write!(
                f,
                "WeakAssetHandle<{}>({}:{})",
                type_name, self.index, self.generation
            )
        } else {
            write!(f, "WeakAssetHandle<{}>(INVALID)", type_name)
        }
    }
}

impl<A: Asset> PartialEq for WeakAssetHandle<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<A: Asset> Eq for WeakAssetHandle<A> {}

impl<A: Asset> Hash for WeakAssetHandle<A> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<A: Asset> From<&AssetHandle<A>> for WeakAssetHandle<A> {
    #[inline]
    fn from(handle: &AssetHandle<A>) -> Self {
        Self::from_handle(handle)
    }
}
