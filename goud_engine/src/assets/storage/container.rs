//! [`AssetStorage`]: type-erased container for all asset types.

use crate::assets::{
    Asset, AssetHandle, AssetId, AssetInfo, AssetPath, AssetState, UntypedAssetHandle,
};
use std::collections::HashMap;
use std::fmt;

use super::any_storage::AnyAssetStorage;
use super::entry::AssetEntry;
use super::typed::TypedAssetStorage;

// =============================================================================
// AssetStorage
// =============================================================================

/// Type-erased container for all asset types.
///
/// `AssetStorage` is the main asset storage container that holds typed storages
/// for each registered asset type. It provides both typed and untyped access
/// patterns.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetStorage, AssetPath};
///
/// struct Texture { width: u32 }
/// impl Asset for Texture {}
///
/// struct Audio { duration: f32 }
/// impl Asset for Audio {}
///
/// let mut storage = AssetStorage::new();
///
/// // Insert different asset types
/// let tex_handle = storage.insert(Texture { width: 256 });
/// let audio_handle = storage.insert(Audio { duration: 2.5 });
///
/// // Type-safe access
/// assert_eq!(storage.get::<Texture>(&tex_handle).unwrap().width, 256);
/// assert_eq!(storage.get::<Audio>(&audio_handle).unwrap().duration, 2.5);
///
/// // Count by type
/// assert_eq!(storage.len::<Texture>(), 1);
/// assert_eq!(storage.len::<Audio>(), 1);
/// ```
pub struct AssetStorage {
    /// Type-erased storage for each asset type.
    storages: HashMap<AssetId, Box<dyn AnyAssetStorage>>,
}

impl AssetStorage {
    /// Creates a new, empty asset storage.
    #[inline]
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    /// Creates a new asset storage with pre-allocated capacity for storage map.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storages: HashMap::with_capacity(capacity),
        }
    }

    /// Inserts an asset and returns its handle.
    ///
    /// Automatically creates the typed storage if it doesn't exist.
    pub fn insert<A: Asset>(&mut self, asset: A) -> AssetHandle<A> {
        self.get_or_create_storage::<A>().insert(asset)
    }

    /// Inserts an asset with an associated path.
    ///
    /// If an asset with the same path exists and is alive, returns the existing handle.
    pub fn insert_with_path<A: Asset>(&mut self, asset: A, path: AssetPath<'_>) -> AssetHandle<A> {
        self.get_or_create_storage::<A>()
            .insert_with_path(asset, path)
    }

    /// Reserves a handle for later loading.
    pub fn reserve<A: Asset>(&mut self) -> AssetHandle<A> {
        self.get_or_create_storage::<A>().reserve()
    }

    /// Reserves a handle with an associated path.
    pub fn reserve_with_path<A: Asset>(&mut self, path: AssetPath<'_>) -> AssetHandle<A> {
        self.get_or_create_storage::<A>().reserve_with_path(path)
    }

    /// Sets a loaded asset for a reserved handle.
    pub fn set_loaded<A: Asset>(&mut self, handle: &AssetHandle<A>, asset: A) -> bool {
        self.get_or_create_storage::<A>().set_loaded(handle, asset)
    }

    /// Removes an asset and returns it.
    pub fn remove<A: Asset>(&mut self, handle: &AssetHandle<A>) -> Option<A> {
        self.get_storage_mut::<A>().and_then(|s| s.remove(handle))
    }

    /// Removes an asset by untyped handle.
    pub fn remove_untyped(&mut self, handle: &UntypedAssetHandle) -> bool {
        self.storages
            .get_mut(&handle.asset_id())
            .map(|s| s.remove_untyped(handle))
            .unwrap_or(false)
    }

    /// Gets a reference to an asset.
    pub fn get<A: Asset>(&self, handle: &AssetHandle<A>) -> Option<&A> {
        self.get_storage::<A>().and_then(|s| s.get(handle))
    }

    /// Gets a mutable reference to an asset.
    pub fn get_mut<A: Asset>(&mut self, handle: &AssetHandle<A>) -> Option<&mut A> {
        self.get_storage_mut::<A>().and_then(|s| s.get_mut(handle))
    }

    /// Gets the entry for a handle.
    pub fn get_entry<A: Asset>(&self, handle: &AssetHandle<A>) -> Option<&AssetEntry<A>> {
        self.get_storage::<A>().and_then(|s| s.get_entry(handle))
    }

    /// Gets the mutable entry for a handle.
    pub fn get_entry_mut<A: Asset>(
        &mut self,
        handle: &AssetHandle<A>,
    ) -> Option<&mut AssetEntry<A>> {
        self.get_storage_mut::<A>()
            .and_then(|s| s.get_entry_mut(handle))
    }

    /// Checks if a handle is alive.
    pub fn is_alive<A: Asset>(&self, handle: &AssetHandle<A>) -> bool {
        self.get_storage::<A>()
            .map(|s| s.is_alive(handle))
            .unwrap_or(false)
    }

    /// Checks if an untyped handle is alive.
    pub fn is_alive_untyped(&self, handle: &UntypedAssetHandle) -> bool {
        self.storages
            .get(&handle.asset_id())
            .map(|s| s.is_alive_raw(handle.index(), handle.generation()))
            .unwrap_or(false)
    }

    /// Gets the state for a handle.
    pub fn get_state<A: Asset>(&self, handle: &AssetHandle<A>) -> Option<AssetState> {
        self.get_storage::<A>().and_then(|s| s.get_state(handle))
    }

    /// Gets the state for an untyped handle.
    pub fn get_state_untyped(&self, handle: &UntypedAssetHandle) -> Option<AssetState> {
        self.storages
            .get(&handle.asset_id())
            .and_then(|s| s.get_state_untyped(handle))
    }

    /// Gets a handle by path.
    pub fn get_handle_by_path<A: Asset>(&self, path: &str) -> Option<AssetHandle<A>> {
        self.get_storage::<A>()
            .and_then(|s| s.get_handle_by_path(path))
    }

    /// Gets an asset by path.
    pub fn get_by_path<A: Asset>(&self, path: &str) -> Option<&A> {
        self.get_storage::<A>().and_then(|s| s.get_by_path(path))
    }

    /// Checks if a path exists for an asset type.
    pub fn has_path<A: Asset>(&self, path: &str) -> bool {
        self.get_storage::<A>()
            .map(|s| s.has_path(path))
            .unwrap_or(false)
    }

    /// Sets the path for a handle.
    pub fn set_path<A: Asset>(&mut self, handle: &AssetHandle<A>, path: AssetPath<'_>) -> bool {
        self.get_storage_mut::<A>()
            .map(|s| s.set_path(handle, path))
            .unwrap_or(false)
    }

    /// Returns the number of assets of a specific type.
    pub fn len<A: Asset>(&self) -> usize {
        self.get_storage::<A>().map(|s| s.len()).unwrap_or(0)
    }

    /// Returns `true` if no assets of a specific type are stored.
    pub fn is_empty_type<A: Asset>(&self) -> bool {
        self.get_storage::<A>()
            .map(|s| s.is_empty())
            .unwrap_or(true)
    }

    /// Returns the total number of assets across all types.
    pub fn total_len(&self) -> usize {
        self.storages.values().map(|s| s.len()).sum()
    }

    /// Returns `true` if no assets are stored.
    pub fn is_empty(&self) -> bool {
        self.storages.values().all(|s| s.is_empty())
    }

    /// Returns the number of registered asset types.
    pub fn type_count(&self) -> usize {
        self.storages.len()
    }

    /// Clears all assets of a specific type.
    pub fn clear_type<A: Asset>(&mut self) {
        if let Some(storage) = self.get_storage_mut::<A>() {
            storage.clear();
        }
    }

    /// Clears all assets from all storages.
    pub fn clear(&mut self) {
        for storage in self.storages.values_mut() {
            storage.clear();
        }
    }

    /// Returns `true` if storage exists for a type.
    pub fn has_type<A: Asset>(&self) -> bool {
        self.storages.contains_key(&AssetId::of::<A>())
    }

    /// Returns asset info for all registered types.
    pub fn registered_types(&self) -> Vec<AssetInfo> {
        self.storages.values().map(|s| s.asset_info()).collect()
    }

    /// Gets the typed storage for an asset type.
    pub fn get_storage<A: Asset>(&self) -> Option<&TypedAssetStorage<A>> {
        self.storages
            .get(&AssetId::of::<A>())
            .and_then(|s| s.as_any().downcast_ref())
    }

    /// Gets mutable typed storage for an asset type.
    pub fn get_storage_mut<A: Asset>(&mut self) -> Option<&mut TypedAssetStorage<A>> {
        self.storages
            .get_mut(&AssetId::of::<A>())
            .and_then(|s| s.as_any_mut().downcast_mut())
    }

    /// Sets a loaded asset from a type-erased boxed value using raw handle parts.
    ///
    /// Used by async loading to finalize results on the main thread.
    pub fn set_loaded_raw(
        &mut self,
        asset_id: AssetId,
        index: u32,
        generation: u32,
        boxed: Box<dyn std::any::Any + Send>,
    ) -> bool {
        self.storages
            .get_mut(&asset_id)
            .map(|s| s.set_loaded_raw(index, generation, boxed))
            .unwrap_or(false)
    }

    /// Replaces a loaded asset by path with a new type-erased value.
    ///
    /// Searches all storage types for an asset with the given path and
    /// replaces it with the new value. Used for hot-reload.
    pub fn replace_erased(&mut self, path: &str, boxed: Box<dyn std::any::Any + Send>) -> bool {
        // First find which storage has this path
        let target_id = self
            .storages
            .iter()
            .find(|(_, s)| s.has_path_erased(path))
            .map(|(id, _)| *id);

        if let Some(id) = target_id {
            if let Some(storage) = self.storages.get_mut(&id) {
                return storage.replace_by_path(path, boxed);
            }
        }
        false
    }

    /// Marks an asset as failed using raw handle parts.
    ///
    /// Used by async loading to report errors on the main thread.
    pub fn set_failed_raw(
        &mut self,
        asset_id: AssetId,
        index: u32,
        generation: u32,
        error: String,
    ) -> bool {
        self.storages
            .get_mut(&asset_id)
            .map(|s| s.set_failed_raw(index, generation, error))
            .unwrap_or(false)
    }

    /// Gets or creates typed storage for an asset type.
    pub fn get_or_create_storage<A: Asset>(&mut self) -> &mut TypedAssetStorage<A> {
        let id = AssetId::of::<A>();
        self.storages
            .entry(id)
            .or_insert_with(|| Box::new(TypedAssetStorage::<A>::new()))
            .as_any_mut()
            .downcast_mut()
            .expect("storage type mismatch")
    }

    /// Iterates over all assets of a specific type.
    pub fn iter<A: Asset>(&self) -> impl Iterator<Item = (AssetHandle<A>, &A)> {
        self.get_storage::<A>().into_iter().flat_map(|s| s.iter())
    }

    /// Returns handles for all assets of a specific type.
    pub fn handles<A: Asset>(&self) -> impl Iterator<Item = AssetHandle<A>> + '_ {
        self.get_storage::<A>()
            .into_iter()
            .flat_map(|s| s.handles())
    }
}

impl Default for AssetStorage {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for AssetStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetStorage")
            .field("type_count", &self.type_count())
            .field("total_assets", &self.total_len())
            .finish()
    }
}
