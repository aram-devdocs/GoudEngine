//! [`TypedAssetStorage`]: storage container for assets of a single type.

use crate::assets::{Asset, AssetHandle, AssetHandleAllocator, AssetPath, AssetState};
use std::collections::HashMap;
use std::fmt;

use super::entry::AssetEntry;
use super::ref_count::RefCountMap;

// =============================================================================
// TypedAssetStorage
// =============================================================================

/// Storage container for assets of a single type.
///
/// Manages assets with generation-based handles, path deduplication,
/// state tracking, and external reference counting for deferred unloading.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, TypedAssetStorage, AssetPath};
///
/// struct Texture { width: u32 }
/// impl Asset for Texture {}
///
/// let mut storage: TypedAssetStorage<Texture> = TypedAssetStorage::new();
/// let handle = storage.insert(Texture { width: 256 });
/// assert!(storage.is_alive(&handle));
/// assert_eq!(storage.get(&handle).unwrap().width, 256);
/// ```
pub struct TypedAssetStorage<A: Asset> {
    /// Handle allocator for this asset type.
    allocator: AssetHandleAllocator<A>,

    /// Asset entries, indexed by handle index.
    entries: Vec<Option<AssetEntry<A>>>,

    /// Mapping from path strings to handles for deduplication.
    path_index: HashMap<String, AssetHandle<A>>,

    /// External reference counts keyed by (index, generation).
    ref_counts: RefCountMap,
}

impl<A: Asset> TypedAssetStorage<A> {
    /// Creates a new, empty storage.
    #[inline]
    pub fn new() -> Self {
        Self {
            allocator: AssetHandleAllocator::new(),
            entries: Vec::new(),
            path_index: HashMap::new(),
            ref_counts: RefCountMap::new(),
        }
    }

    /// Creates a new storage with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            allocator: AssetHandleAllocator::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
            path_index: HashMap::new(),
            ref_counts: RefCountMap::new(),
        }
    }

    /// Inserts an asset and returns its handle (ref count starts at 1).
    pub fn insert(&mut self, asset: A) -> AssetHandle<A> {
        let handle = self.allocator.allocate();
        let index = handle.index() as usize;

        // Ensure entries vec is large enough
        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, || None);
        }

        self.entries[index] = Some(AssetEntry::loaded(asset));
        self.ref_counts.insert(handle.index(), handle.generation());
        handle
    }

    /// Inserts an asset with an associated path.
    ///
    /// If an asset with the same path already exists and is alive,
    /// returns the existing handle instead of inserting a new one.
    pub fn insert_with_path(&mut self, asset: A, path: AssetPath<'_>) -> AssetHandle<A> {
        let path_string = path.as_str().to_string();

        // Check if path already exists and handle is alive
        if let Some(&existing) = self.path_index.get(&path_string) {
            if self.allocator.is_alive(existing) {
                return existing;
            }
            // Handle is stale, remove from index
            self.path_index.remove(&path_string);
        }

        // Insert new asset
        let handle = self.allocator.allocate();
        let index = handle.index() as usize;

        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, || None);
        }

        self.entries[index] = Some(AssetEntry::with_path(asset, path.into_owned()));
        self.path_index.insert(path_string, handle);
        self.ref_counts.insert(handle.index(), handle.generation());

        handle
    }

    /// Reserves a handle for later use (e.g., async loading).
    ///
    /// Creates an empty entry that can be populated later via [`Self::set_loaded`].
    /// The handle starts with a reference count of 1.
    pub fn reserve(&mut self) -> AssetHandle<A> {
        let handle = self.allocator.allocate();
        let index = handle.index() as usize;

        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, || None);
        }

        self.entries[index] = Some(AssetEntry::empty());
        self.ref_counts.insert(handle.index(), handle.generation());
        handle
    }

    /// Reserves a handle with an associated path.
    pub fn reserve_with_path(&mut self, path: AssetPath<'_>) -> AssetHandle<A> {
        let path_string = path.as_str().to_string();

        // Check if path already exists and handle is alive
        if let Some(&existing) = self.path_index.get(&path_string) {
            if self.allocator.is_alive(existing) {
                return existing;
            }
            self.path_index.remove(&path_string);
        }

        let handle = self.allocator.allocate();
        let index = handle.index() as usize;

        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, || None);
        }

        let mut entry = AssetEntry::empty();
        entry.set_path(path.into_owned());
        self.entries[index] = Some(entry);
        self.path_index.insert(path_string, handle);
        self.ref_counts.insert(handle.index(), handle.generation());

        handle
    }

    /// Sets a loaded asset for a previously reserved handle.
    ///
    /// Returns `true` if successful, `false` if handle is invalid/stale.
    pub fn set_loaded(&mut self, handle: &AssetHandle<A>, asset: A) -> bool {
        if !self.allocator.is_alive(*handle) {
            return false;
        }

        let index = handle.index() as usize;
        if let Some(entry) = self.entries.get_mut(index).and_then(|e| e.as_mut()) {
            entry.set_loaded(asset);
            true
        } else {
            false
        }
    }

    /// Removes an asset unconditionally, ignoring reference counts.
    ///
    /// For ref-count-aware removal, use [`Self::try_remove`].
    pub fn remove(&mut self, handle: &AssetHandle<A>) -> Option<A> {
        self.ref_counts.remove(handle.index(), handle.generation());
        self.remove_inner(handle)
    }

    /// Removes an asset only if its reference count is zero.
    ///
    /// Returns `None` if the handle is invalid, stale, or still referenced.
    pub fn try_remove(&mut self, handle: &AssetHandle<A>) -> Option<A> {
        if !self.allocator.is_alive(*handle) {
            return None;
        }
        let count = self.ref_counts.get(handle.index(), handle.generation());
        if count > 0 {
            return None;
        }
        self.ref_counts.remove(handle.index(), handle.generation());
        self.remove_inner(handle)
    }

    /// Unconditional removal (alias used by `unload`).
    pub fn force_remove(&mut self, handle: &AssetHandle<A>) -> Option<A> {
        self.remove(handle)
    }

    /// Shared removal logic.
    fn remove_inner(&mut self, handle: &AssetHandle<A>) -> Option<A> {
        if !self.allocator.is_alive(*handle) {
            return None;
        }

        let index = handle.index() as usize;

        // Remove from path index if has path
        if let Some(Some(entry)) = self.entries.get(index) {
            if let Some(path) = entry.path() {
                self.path_index.remove(path.as_str());
            }
        }

        // Take the entry
        let entry = self.entries.get_mut(index).and_then(|e| e.take());
        self.allocator.deallocate(*handle);

        entry.and_then(|mut e| e.take_asset())
    }

    /// Increments the reference count for a handle.
    ///
    /// Returns the new count, or `None` if the handle is invalid.
    pub fn retain(&mut self, handle: &AssetHandle<A>) -> Option<u32> {
        if !self.allocator.is_alive(*handle) {
            return None;
        }
        self.ref_counts
            .increment(handle.index(), handle.generation())
    }

    /// Decrements the reference count for a handle.
    ///
    /// Returns the new count, or `None` if the handle is invalid.
    pub fn release(&mut self, handle: &AssetHandle<A>) -> Option<u32> {
        if !self.allocator.is_alive(*handle) {
            return None;
        }
        self.ref_counts
            .decrement(handle.index(), handle.generation())
    }

    /// Returns the current reference count for a handle.
    pub fn ref_count(&self, handle: &AssetHandle<A>) -> u32 {
        if !self.allocator.is_alive(*handle) {
            return 0;
        }
        self.ref_counts.get(handle.index(), handle.generation())
    }

    /// Gets a reference to an asset.
    ///
    /// Returns `None` if the handle is invalid, stale, or the asset is not loaded.
    #[inline]
    pub fn get(&self, handle: &AssetHandle<A>) -> Option<&A> {
        if !self.allocator.is_alive(*handle) {
            return None;
        }

        let index = handle.index() as usize;
        self.entries
            .get(index)
            .and_then(|e| e.as_ref())
            .and_then(|e| e.asset())
    }

    /// Gets a mutable reference to an asset.
    #[inline]
    pub fn get_mut(&mut self, handle: &AssetHandle<A>) -> Option<&mut A> {
        if !self.allocator.is_alive(*handle) {
            return None;
        }

        let index = handle.index() as usize;
        self.entries
            .get_mut(index)
            .and_then(|e| e.as_mut())
            .and_then(|e| e.asset_mut())
    }

    /// Gets a reference to the entry for a handle.
    #[inline]
    pub fn get_entry(&self, handle: &AssetHandle<A>) -> Option<&AssetEntry<A>> {
        if !self.allocator.is_alive(*handle) {
            return None;
        }

        let index = handle.index() as usize;
        self.entries.get(index).and_then(|e| e.as_ref())
    }

    /// Gets a mutable reference to the entry for a handle.
    #[inline]
    pub fn get_entry_mut(&mut self, handle: &AssetHandle<A>) -> Option<&mut AssetEntry<A>> {
        if !self.allocator.is_alive(*handle) {
            return None;
        }

        let index = handle.index() as usize;
        self.entries.get_mut(index).and_then(|e| e.as_mut())
    }

    /// Checks if a handle is still alive.
    #[inline]
    pub fn is_alive(&self, handle: &AssetHandle<A>) -> bool {
        self.allocator.is_alive(*handle)
    }

    /// Checks liveness by raw index and generation (used by `AnyAssetStorage`).
    #[inline]
    pub(super) fn is_alive_raw_inner(&self, handle: AssetHandle<A>) -> bool {
        self.allocator.is_alive(handle)
    }

    /// Returns the state for a handle.
    pub fn get_state(&self, handle: &AssetHandle<A>) -> Option<AssetState> {
        self.get_entry(handle).map(|e| e.state().clone())
    }

    /// Gets a handle by path.
    ///
    /// Returns `None` if no asset is associated with the path.
    pub fn get_handle_by_path(&self, path: &str) -> Option<AssetHandle<A>> {
        self.path_index
            .get(path)
            .copied()
            .filter(|h| self.allocator.is_alive(*h))
    }

    /// Gets an asset by path.
    pub fn get_by_path(&self, path: &str) -> Option<&A> {
        self.get_handle_by_path(path).and_then(|h| self.get(&h))
    }

    /// Checks if a path is registered.
    pub fn has_path(&self, path: &str) -> bool {
        self.get_handle_by_path(path).is_some()
    }

    /// Sets the path for a handle.
    ///
    /// If the handle already has a path, the old path mapping is removed.
    pub fn set_path(&mut self, handle: &AssetHandle<A>, path: AssetPath<'_>) -> bool {
        if !self.allocator.is_alive(*handle) {
            return false;
        }

        let index = handle.index() as usize;

        // Remove old path mapping
        if let Some(Some(entry)) = self.entries.get(index) {
            if let Some(old_path) = entry.path() {
                self.path_index.remove(old_path.as_str());
            }
        }

        // Set new path
        if let Some(Some(entry)) = self.entries.get_mut(index) {
            let owned_path = path.into_owned();
            self.path_index
                .insert(owned_path.as_str().to_string(), *handle);
            entry.set_path(owned_path);
            true
        } else {
            false
        }
    }

    /// Clears the path for a handle.
    pub fn clear_path(&mut self, handle: &AssetHandle<A>) -> bool {
        if !self.allocator.is_alive(*handle) {
            return false;
        }

        let index = handle.index() as usize;
        if let Some(Some(entry)) = self.entries.get_mut(index) {
            if let Some(path) = entry.path() {
                self.path_index.remove(path.as_str());
            }
            entry.clear_path();
            true
        } else {
            false
        }
    }

    /// Returns the number of currently stored assets.
    #[inline]
    pub fn len(&self) -> usize {
        self.allocator.len()
    }

    /// Returns `true` if no assets are stored.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.allocator.is_empty()
    }

    /// Returns the total capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.allocator.capacity()
    }

    /// Clears all assets from storage.
    pub fn clear(&mut self) {
        self.allocator.clear();
        self.entries.clear();
        self.path_index.clear();
        self.ref_counts.clear();
    }

    /// Returns an iterator over all valid (handle, asset) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (AssetHandle<A>, &A)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(move |(index, entry)| {
                let entry = entry.as_ref()?;
                let asset = entry.asset()?;
                let gen = self.allocator.generation_at(index as u32)?;
                let handle = AssetHandle::new(index as u32, gen);
                if self.allocator.is_alive(handle) {
                    Some((handle, asset))
                } else {
                    None
                }
            })
    }

    /// Returns an iterator over all valid handles.
    pub fn handles(&self) -> impl Iterator<Item = AssetHandle<A>> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter_map(move |(index, entry)| {
                if entry.is_some() {
                    let gen = self.allocator.generation_at(index as u32)?;
                    let handle = AssetHandle::new(index as u32, gen);
                    if self.allocator.is_alive(handle) {
                        return Some(handle);
                    }
                }
                None
            })
    }

    /// Returns an iterator over all paths.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.path_index.keys().map(|s| s.as_str())
    }

    /// Returns the number of registered paths.
    pub fn path_count(&self) -> usize {
        self.path_index.len()
    }

    /// Shrinks internal storage to fit current usage.
    pub fn shrink_to_fit(&mut self) {
        self.allocator.shrink_to_fit();
        self.entries.shrink_to_fit();
        self.path_index.shrink_to_fit();
    }
}

impl<A: Asset> Default for TypedAssetStorage<A> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Asset> fmt::Debug for TypedAssetStorage<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!("TypedAssetStorage<{}>", A::asset_type_name()))
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("path_count", &self.path_count())
            .finish()
    }
}
