//! [`AnyAssetStorage`] trait and its implementation for [`TypedAssetStorage`].

use crate::assets::{Asset, AssetId, AssetInfo, AssetPath, AssetState, UntypedAssetHandle};
use std::any::Any;

use super::typed::TypedAssetStorage;
use crate::assets::AssetHandle;

// =============================================================================
// AnyAssetStorage Trait
// =============================================================================

/// Trait for type-erased asset storage operations.
///
/// This trait enables storing different `TypedAssetStorage<T>` instances
/// in a single collection while maintaining type safety through `AssetId`.
pub trait AnyAssetStorage: Send + Sync {
    /// Returns the `AssetId` for the asset type stored.
    fn asset_id(&self) -> AssetId;

    /// Returns the `AssetInfo` for the stored asset type.
    fn asset_info(&self) -> AssetInfo;

    /// Returns the number of assets currently stored.
    fn len(&self) -> usize;

    /// Returns `true` if no assets are stored.
    fn is_empty(&self) -> bool;

    /// Returns the total capacity.
    fn capacity(&self) -> usize;

    /// Clears all assets from storage.
    fn clear(&mut self);

    /// Checks if a handle (by index and generation) is alive.
    fn is_alive_raw(&self, index: u32, generation: u32) -> bool;

    /// Removes an asset by untyped handle.
    fn remove_untyped(&mut self, handle: &UntypedAssetHandle) -> bool;

    /// Returns the state for an untyped handle.
    fn get_state_untyped(&self, handle: &UntypedAssetHandle) -> Option<AssetState>;

    /// Returns the path for an untyped handle.
    fn get_path_untyped(&self, handle: &UntypedAssetHandle) -> Option<AssetPath<'static>>;

    /// Returns as `Any` for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Returns as mutable `Any` for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Sets a loaded asset from a type-erased boxed value.
    ///
    /// The `boxed` value must downcast to the correct asset type.
    /// Returns `true` if the asset was successfully set.
    fn set_loaded_raw(
        &mut self,
        index: u32,
        generation: u32,
        boxed: Box<dyn std::any::Any + Send>,
    ) -> bool;

    /// Marks an asset as failed with a type-erased error message.
    ///
    /// Returns `true` if the entry was found and updated.
    fn set_failed_raw(&mut self, index: u32, generation: u32, error: String) -> bool;
}

// Implement AnyAssetStorage for TypedAssetStorage
impl<A: Asset> AnyAssetStorage for TypedAssetStorage<A> {
    fn asset_id(&self) -> AssetId {
        AssetId::of::<A>()
    }

    fn asset_info(&self) -> AssetInfo {
        AssetInfo::of::<A>()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn is_alive_raw(&self, index: u32, generation: u32) -> bool {
        let handle = AssetHandle::<A>::new(index, generation);
        self.is_alive_raw_inner(handle)
    }

    fn remove_untyped(&mut self, handle: &UntypedAssetHandle) -> bool {
        if handle.asset_id() != AssetId::of::<A>() {
            return false;
        }

        let typed = AssetHandle::<A>::new(handle.index(), handle.generation());
        self.remove(&typed).is_some()
    }

    fn get_state_untyped(&self, handle: &UntypedAssetHandle) -> Option<AssetState> {
        if handle.asset_id() != AssetId::of::<A>() {
            return None;
        }

        let typed = AssetHandle::<A>::new(handle.index(), handle.generation());
        self.get_state(&typed)
    }

    fn get_path_untyped(&self, handle: &UntypedAssetHandle) -> Option<AssetPath<'static>> {
        if handle.asset_id() != AssetId::of::<A>() {
            return None;
        }

        let typed = AssetHandle::<A>::new(handle.index(), handle.generation());
        self.get_entry(&typed).and_then(|e| e.path().cloned())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_loaded_raw(
        &mut self,
        index: u32,
        generation: u32,
        boxed: Box<dyn std::any::Any + Send>,
    ) -> bool {
        let handle = AssetHandle::<A>::new(index, generation);
        match boxed.downcast::<A>() {
            Ok(asset) => self.set_loaded(&handle, *asset),
            Err(_) => self.set_failed_raw(index, generation, "Type mismatch during async load downcast".to_string()),
        }
    }

    fn set_failed_raw(&mut self, index: u32, generation: u32, error: String) -> bool {
        let handle = AssetHandle::<A>::new(index, generation);
        match self.get_entry_mut(&handle) {
            Some(entry) => {
                entry.set_failed(error);
                true
            }
            None => false,
        }
    }
}
