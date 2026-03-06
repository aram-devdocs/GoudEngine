//! Type-erased asset handle for heterogeneous asset collections.

use crate::assets::{Asset, AssetId};
use std::fmt;
use std::hash::{Hash, Hasher};

use super::typed::AssetHandle;

// =============================================================================
// UntypedAssetHandle
// =============================================================================

/// A type-erased handle to any asset type.
///
/// `UntypedAssetHandle` allows storing handles to different asset types in the
/// same collection, at the cost of runtime type checking instead of compile-time.
///
/// # Use Cases
///
/// - Asset registries that store mixed asset types
/// - Dynamic asset loading where type is determined at runtime
/// - FFI where type information must be preserved explicitly
///
/// # FFI Layout
///
/// ```text
/// Offset 0:  index      (u32, 4 bytes)
/// Offset 4:  generation (u32, 4 bytes)
/// Offset 8:  asset_id   (AssetId, varies by platform)
/// ```
///
/// For FFI, use `to_packed()` which provides a consistent representation.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetHandle, AssetId, UntypedAssetHandle};
///
/// struct Texture;
/// impl Asset for Texture {}
///
/// struct Audio;
/// impl Asset for Audio {}
///
/// // Create typed handles
/// let tex_handle: AssetHandle<Texture> = AssetHandle::new(1, 1);
/// let audio_handle: AssetHandle<Audio> = AssetHandle::new(2, 1);
///
/// // Convert to untyped for storage in a single collection
/// let handles: Vec<UntypedAssetHandle> = vec![
///     tex_handle.untyped(),
///     audio_handle.untyped(),
/// ];
///
/// // Runtime type checking
/// assert_eq!(handles[0].asset_id(), AssetId::of::<Texture>());
/// assert_eq!(handles[1].asset_id(), AssetId::of::<Audio>());
///
/// // Convert back to typed (with type check)
/// let recovered: Option<AssetHandle<Texture>> = handles[0].typed::<Texture>();
/// assert!(recovered.is_some());
///
/// // Wrong type returns None
/// let wrong: Option<AssetHandle<Audio>> = handles[0].typed::<Audio>();
/// assert!(wrong.is_none());
/// ```
#[derive(Clone, Copy)]
pub struct UntypedAssetHandle {
    /// Slot index in the asset storage.
    index: u32,

    /// Generation counter for stale handle detection.
    generation: u32,

    /// The asset type this handle refers to.
    asset_id: AssetId,
}

impl UntypedAssetHandle {
    /// The invalid untyped handle constant.
    ///
    /// Uses `AssetId::of_raw::<()>()` as a placeholder type ID.
    pub fn invalid() -> Self {
        Self {
            index: u32::MAX,
            generation: 0,
            asset_id: AssetId::of_raw::<()>(),
        }
    }

    /// Creates a new untyped handle.
    ///
    /// # Arguments
    ///
    /// * `index` - Slot index in asset storage
    /// * `generation` - Generation counter
    /// * `asset_id` - The asset type identifier
    #[inline]
    pub const fn new(index: u32, generation: u32, asset_id: AssetId) -> Self {
        Self {
            index,
            generation,
            asset_id,
        }
    }

    /// Creates an untyped handle from a typed handle.
    ///
    /// Equivalent to calling `typed_handle.untyped()`.
    #[inline]
    pub fn from_typed<A: Asset>(handle: AssetHandle<A>) -> Self {
        handle.untyped()
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

    /// Returns the asset type identifier.
    #[inline]
    pub const fn asset_id(&self) -> AssetId {
        self.asset_id
    }

    /// Returns `true` if this is not the invalid sentinel.
    #[inline]
    pub fn is_valid(&self) -> bool {
        !(self.index == u32::MAX && self.generation == 0)
    }

    /// Attempts to convert this untyped handle to a typed handle.
    ///
    /// Returns `Some(handle)` if the asset type matches, `None` otherwise.
    ///
    /// # Type Safety
    ///
    /// This performs a runtime type check using `AssetId`. If you're certain
    /// of the type, you can use `typed_unchecked()` instead.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle, UntypedAssetHandle};
    ///
    /// struct Texture;
    /// impl Asset for Texture {}
    ///
    /// struct Audio;
    /// impl Asset for Audio {}
    ///
    /// let typed: AssetHandle<Texture> = AssetHandle::new(1, 1);
    /// let untyped = typed.untyped();
    ///
    /// // Correct type succeeds
    /// assert!(untyped.typed::<Texture>().is_some());
    ///
    /// // Wrong type fails
    /// assert!(untyped.typed::<Audio>().is_none());
    /// ```
    #[inline]
    pub fn typed<A: Asset>(&self) -> Option<AssetHandle<A>> {
        if self.asset_id == AssetId::of::<A>() {
            Some(AssetHandle::new(self.index, self.generation))
        } else {
            None
        }
    }

    /// Converts this untyped handle to a typed handle without checking the type.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this handle was created from an asset of type `A`.
    /// Using a handle with the wrong type leads to undefined behavior when
    /// accessing the asset data.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle, UntypedAssetHandle};
    ///
    /// struct Texture;
    /// impl Asset for Texture {}
    ///
    /// let typed: AssetHandle<Texture> = AssetHandle::new(1, 1);
    /// let untyped = typed.untyped();
    ///
    /// // SAFETY: We know this was created from a Texture handle
    /// let recovered: AssetHandle<Texture> = unsafe { untyped.typed_unchecked() };
    /// assert_eq!(typed, recovered);
    /// ```
    #[inline]
    pub unsafe fn typed_unchecked<A: Asset>(&self) -> AssetHandle<A> {
        AssetHandle::new(self.index, self.generation)
    }

    /// Checks if this handle's type matches the given asset type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle, UntypedAssetHandle};
    ///
    /// struct Texture;
    /// impl Asset for Texture {}
    ///
    /// struct Audio;
    /// impl Asset for Audio {}
    ///
    /// let typed: AssetHandle<Texture> = AssetHandle::new(1, 1);
    /// let untyped = typed.untyped();
    ///
    /// assert!(untyped.is_type::<Texture>());
    /// assert!(!untyped.is_type::<Audio>());
    /// ```
    #[inline]
    pub fn is_type<A: Asset>(&self) -> bool {
        self.asset_id == AssetId::of::<A>()
    }

    /// Packs index and generation into a u64 (does not include type info).
    #[inline]
    pub const fn to_u64(&self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }
}

impl Default for UntypedAssetHandle {
    #[inline]
    fn default() -> Self {
        Self::invalid()
    }
}

impl fmt::Debug for UntypedAssetHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            write!(
                f,
                "UntypedAssetHandle({}:{}, {:?})",
                self.index, self.generation, self.asset_id
            )
        } else {
            write!(f, "UntypedAssetHandle(INVALID)")
        }
    }
}

impl fmt::Display for UntypedAssetHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            write!(f, "{}:{}", self.index, self.generation)
        } else {
            write!(f, "INVALID")
        }
    }
}

impl PartialEq for UntypedAssetHandle {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.generation == other.generation
            && self.asset_id == other.asset_id
    }
}

impl Eq for UntypedAssetHandle {}

impl Hash for UntypedAssetHandle {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
        self.asset_id.hash(state);
    }
}
