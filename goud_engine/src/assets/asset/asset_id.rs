//! Runtime type identifier for asset types.

use super::trait_def::Asset;
use std::any::TypeId;
use std::fmt;
use std::hash::Hash;

/// Unique identifier for an asset type at runtime.
///
/// `AssetId` wraps a `TypeId` to identify asset types. This is used internally
/// for asset storage, loader registration, and type-safe access.
///
/// Unlike [`Handle`](crate::core::handle::Handle), which identifies a specific
/// asset instance, `AssetId` identifies an asset *type*.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetId};
///
/// struct Texture { /* ... */ }
/// impl Asset for Texture {}
///
/// struct Mesh { /* ... */ }
/// impl Asset for Mesh {}
///
/// let tex_id = AssetId::of::<Texture>();
/// let mesh_id = AssetId::of::<Mesh>();
///
/// assert_ne!(tex_id, mesh_id);
/// assert_eq!(tex_id, AssetId::of::<Texture>());
/// ```
///
/// # FFI Considerations
///
/// `AssetId` is NOT FFI-safe (contains `TypeId`). For FFI, use the
/// `AssetType` enum or string-based type names.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetId(TypeId);

impl AssetId {
    /// Returns the `AssetId` for a specific asset type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetId};
    ///
    /// struct MyAsset;
    /// impl Asset for MyAsset {}
    ///
    /// let id = AssetId::of::<MyAsset>();
    /// ```
    #[inline]
    pub fn of<T: Asset>() -> Self {
        Self(TypeId::of::<T>())
    }

    /// Returns the `AssetId` for any `'static` type.
    ///
    /// This is useful for internal operations where the type may not
    /// implement `Asset` yet (e.g., during registration).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetId;
    ///
    /// let id = AssetId::of_raw::<String>();
    /// ```
    #[inline]
    pub fn of_raw<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }

    /// Returns the underlying `TypeId`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetId};
    /// use std::any::TypeId;
    ///
    /// struct MyAsset;
    /// impl Asset for MyAsset {}
    ///
    /// let id = AssetId::of::<MyAsset>();
    /// assert_eq!(id.type_id(), TypeId::of::<MyAsset>());
    /// ```
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.0
    }
}

impl fmt::Debug for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetId({:?})", self.0)
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetId({:?})", self.0)
    }
}
