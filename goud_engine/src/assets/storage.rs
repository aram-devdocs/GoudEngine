//! Asset storage system for caching and managing loaded assets.
//!
//! This module provides the infrastructure for storing loaded assets:
//!
//! - [`TypedAssetStorage<A>`]: Storage for a single asset type
//! - [`AssetStorage`]: Type-erased container for all asset types
//! - [`AssetEntry<A>`]: Individual asset entry with metadata
//!
//! # Design Philosophy
//!
//! The asset storage system is designed with these goals:
//!
//! 1. **Type Safety**: Typed access through handles prevents misuse
//! 2. **Efficient Lookup**: O(1) access by handle, O(1) amortized by path
//! 3. **Memory Efficient**: Slot reuse through generational indices
//! 4. **Thread Safe**: Storage is `Send + Sync` for parallel access
//!
//! # Storage Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        AssetStorage                              │
//! │  ┌───────────────────────────────────────────────────────────┐  │
//! │  │  HashMap<AssetId, Box<dyn AnyAssetStorage>>               │  │
//! │  │  ┌─────────────────────┐  ┌─────────────────────┐        │  │
//! │  │  │TypedAssetStorage<T1>│  │TypedAssetStorage<T2>│  ...   │  │
//! │  │  │  - allocator        │  │  - allocator        │        │  │
//! │  │  │  - entries[]        │  │  - entries[]        │        │  │
//! │  │  │  - path_index       │  │  - path_index       │        │  │
//! │  │  └─────────────────────┘  └─────────────────────┘        │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```
//! use goud_engine::assets::{Asset, AssetStorage, AssetState, AssetPath};
//!
//! // Define asset types
//! struct Texture { width: u32, height: u32 }
//! impl Asset for Texture {}
//!
//! struct Audio { duration: f32 }
//! impl Asset for Audio {}
//!
//! // Create storage
//! let mut storage = AssetStorage::new();
//!
//! // Insert assets
//! let tex_handle = storage.insert(Texture { width: 256, height: 256 });
//! let audio_handle = storage.insert(Audio { duration: 2.5 });
//!
//! // Access assets
//! let tex = storage.get::<Texture>(&tex_handle);
//! assert!(tex.is_some());
//!
//! // Path-based lookup (after associating path)
//! storage.set_path(&tex_handle, AssetPath::new("textures/player.png"));
//! let found = storage.get_handle_by_path::<Texture>("textures/player.png");
//! assert_eq!(found, Some(tex_handle));
//! ```

use crate::assets::{
    Asset, AssetHandle, AssetHandleAllocator, AssetId, AssetInfo, AssetPath, AssetState,
    UntypedAssetHandle,
};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;

// =============================================================================
// AssetEntry
// =============================================================================

/// An individual asset entry in storage with metadata.
///
/// `AssetEntry` wraps an asset value together with its loading state,
/// optional path association, and other metadata.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetEntry, AssetState, AssetPath};
///
/// struct Texture { width: u32 }
/// impl Asset for Texture {}
///
/// // Create a loaded entry
/// let entry = AssetEntry::loaded(Texture { width: 256 });
/// assert!(entry.is_loaded());
/// assert_eq!(entry.asset().unwrap().width, 256);
///
/// // Create entry with path
/// let entry = AssetEntry::with_path(
///     Texture { width: 512 },
///     AssetPath::new("textures/player.png"),
/// );
/// assert_eq!(entry.path().map(|p| p.as_str()), Some("textures/player.png"));
/// ```
#[derive(Debug, Clone)]
pub struct AssetEntry<A: Asset> {
    /// The asset data, if loaded.
    asset: Option<A>,

    /// Current loading state.
    state: AssetState,

    /// Optional path this asset was loaded from.
    path: Option<AssetPath<'static>>,
}

impl<A: Asset> AssetEntry<A> {
    /// Creates a new entry in the `NotLoaded` state.
    ///
    /// Use this when you want to reserve a handle before the asset is loaded.
    #[inline]
    pub fn empty() -> Self {
        Self {
            asset: None,
            state: AssetState::NotLoaded,
            path: None,
        }
    }

    /// Creates a new entry with a loading state.
    ///
    /// # Arguments
    ///
    /// * `progress` - Initial loading progress (0.0 to 1.0)
    #[inline]
    pub fn loading(progress: f32) -> Self {
        Self {
            asset: None,
            state: AssetState::Loading { progress },
            path: None,
        }
    }

    /// Creates a new entry with a loaded asset.
    #[inline]
    pub fn loaded(asset: A) -> Self {
        Self {
            asset: Some(asset),
            state: AssetState::Loaded,
            path: None,
        }
    }

    /// Creates a new entry with a loaded asset and path.
    pub fn with_path(asset: A, path: AssetPath<'static>) -> Self {
        Self {
            asset: Some(asset),
            state: AssetState::Loaded,
            path: Some(path),
        }
    }

    /// Creates a new failed entry with an error message.
    #[inline]
    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            asset: None,
            state: AssetState::Failed {
                error: error.into(),
            },
            path: None,
        }
    }

    /// Returns a reference to the asset if loaded.
    #[inline]
    pub fn asset(&self) -> Option<&A> {
        self.asset.as_ref()
    }

    /// Returns a mutable reference to the asset if loaded.
    #[inline]
    pub fn asset_mut(&mut self) -> Option<&mut A> {
        self.asset.as_mut()
    }

    /// Takes the asset out of this entry, leaving `None`.
    #[inline]
    pub fn take_asset(&mut self) -> Option<A> {
        let asset = self.asset.take();
        if asset.is_some() {
            self.state = AssetState::Unloaded;
        }
        asset
    }

    /// Returns a reference to the current state.
    #[inline]
    pub fn state(&self) -> &AssetState {
        &self.state
    }

    /// Returns the path this asset was loaded from, if any.
    #[inline]
    pub fn path(&self) -> Option<&AssetPath<'static>> {
        self.path.as_ref()
    }

    /// Sets the path for this entry.
    #[inline]
    pub fn set_path(&mut self, path: AssetPath<'static>) {
        self.path = Some(path);
    }

    /// Clears the path for this entry.
    #[inline]
    pub fn clear_path(&mut self) {
        self.path = None;
    }

    /// Returns `true` if the asset is fully loaded.
    #[inline]
    pub fn is_loaded(&self) -> bool {
        self.state.is_ready() && self.asset.is_some()
    }

    /// Returns `true` if the asset is currently loading.
    #[inline]
    pub fn is_loading(&self) -> bool {
        self.state.is_loading()
    }

    /// Returns `true` if loading failed.
    #[inline]
    pub fn is_failed(&self) -> bool {
        self.state.is_failed()
    }

    /// Sets the asset and marks as loaded.
    pub fn set_loaded(&mut self, asset: A) {
        self.asset = Some(asset);
        self.state = AssetState::Loaded;
    }

    /// Updates the loading progress.
    pub fn set_progress(&mut self, progress: f32) {
        self.state = AssetState::Loading { progress };
    }

    /// Marks the entry as failed.
    pub fn set_failed(&mut self, error: impl Into<String>) {
        self.asset = None;
        self.state = AssetState::Failed {
            error: error.into(),
        };
    }

    /// Marks the entry as unloaded and removes the asset.
    pub fn set_unloaded(&mut self) {
        self.asset = None;
        self.state = AssetState::Unloaded;
    }
}

impl<A: Asset> Default for AssetEntry<A> {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

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
}

// =============================================================================
// TypedAssetStorage
// =============================================================================

/// Storage container for assets of a single type.
///
/// `TypedAssetStorage` manages a collection of assets with:
/// - Generation-based handle allocation
/// - Path-to-handle mapping for deduplication
/// - State tracking for each asset
///
/// # Thread Safety
///
/// `TypedAssetStorage` is `Send + Sync` when the asset type `A` is `Send + Sync`,
/// which is guaranteed by the `Asset` trait bounds.
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
///
/// // Insert an asset
/// let handle = storage.insert(Texture { width: 256 });
/// assert!(storage.is_alive(&handle));
///
/// // Access the asset
/// let tex = storage.get(&handle);
/// assert_eq!(tex.unwrap().width, 256);
///
/// // Path-based lookup
/// storage.set_path(&handle, AssetPath::new("textures/player.png"));
/// let found = storage.get_handle_by_path("textures/player.png");
/// assert!(found.is_some());
/// ```
pub struct TypedAssetStorage<A: Asset> {
    /// Handle allocator for this asset type.
    allocator: AssetHandleAllocator<A>,

    /// Asset entries, indexed by handle index.
    entries: Vec<Option<AssetEntry<A>>>,

    /// Mapping from path strings to handles for deduplication.
    path_index: HashMap<String, AssetHandle<A>>,
}

impl<A: Asset> TypedAssetStorage<A> {
    /// Creates a new, empty storage.
    #[inline]
    pub fn new() -> Self {
        Self {
            allocator: AssetHandleAllocator::new(),
            entries: Vec::new(),
            path_index: HashMap::new(),
        }
    }

    /// Creates a new storage with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            allocator: AssetHandleAllocator::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
            path_index: HashMap::new(),
        }
    }

    /// Inserts an asset and returns its handle.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, TypedAssetStorage};
    ///
    /// struct Texture { width: u32 }
    /// impl Asset for Texture {}
    ///
    /// let mut storage: TypedAssetStorage<Texture> = TypedAssetStorage::new();
    /// let handle = storage.insert(Texture { width: 256 });
    ///
    /// assert!(handle.is_valid());
    /// assert!(storage.is_alive(&handle));
    /// ```
    pub fn insert(&mut self, asset: A) -> AssetHandle<A> {
        let handle = self.allocator.allocate();
        let index = handle.index() as usize;

        // Ensure entries vec is large enough
        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, || None);
        }

        self.entries[index] = Some(AssetEntry::loaded(asset));
        handle
    }

    /// Inserts an asset with an associated path.
    ///
    /// If an asset with the same path already exists and is alive,
    /// returns the existing handle instead of inserting a new one.
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
    ///
    /// // First insert
    /// let h1 = storage.insert_with_path(
    ///     Texture { width: 256 },
    ///     AssetPath::new("textures/player.png"),
    /// );
    ///
    /// // Same path returns existing handle (asset not replaced)
    /// let h2 = storage.insert_with_path(
    ///     Texture { width: 512 },  // This asset is ignored
    ///     AssetPath::new("textures/player.png"),
    /// );
    ///
    /// assert_eq!(h1, h2);
    /// assert_eq!(storage.get(&h1).unwrap().width, 256);  // Original asset
    /// ```
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

        handle
    }

    /// Reserves a handle for later use (e.g., async loading).
    ///
    /// Creates an empty entry that can be populated later.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, TypedAssetStorage};
    ///
    /// struct Texture { width: u32 }
    /// impl Asset for Texture {}
    ///
    /// let mut storage: TypedAssetStorage<Texture> = TypedAssetStorage::new();
    ///
    /// // Reserve handle before loading
    /// let handle = storage.reserve();
    /// assert!(storage.is_alive(&handle));
    /// assert!(storage.get(&handle).is_none());  // Not loaded yet
    ///
    /// // Later, set the loaded asset
    /// storage.set_loaded(&handle, Texture { width: 256 });
    /// assert!(storage.get(&handle).is_some());
    /// ```
    pub fn reserve(&mut self) -> AssetHandle<A> {
        let handle = self.allocator.allocate();
        let index = handle.index() as usize;

        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, || None);
        }

        self.entries[index] = Some(AssetEntry::empty());
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

    /// Removes an asset and returns it if present.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, TypedAssetStorage};
    ///
    /// struct Texture { width: u32 }
    /// impl Asset for Texture {}
    ///
    /// let mut storage: TypedAssetStorage<Texture> = TypedAssetStorage::new();
    /// let handle = storage.insert(Texture { width: 256 });
    ///
    /// let removed = storage.remove(&handle);
    /// assert!(removed.is_some());
    /// assert_eq!(removed.unwrap().width, 256);
    /// assert!(!storage.is_alive(&handle));
    /// ```
    pub fn remove(&mut self, handle: &AssetHandle<A>) -> Option<A> {
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

    /// Returns the state for a handle.
    pub fn get_state(&self, handle: &AssetHandle<A>) -> Option<AssetState> {
        self.get_entry(handle).map(|e| e.state().clone())
    }

    /// Gets a handle by path.
    ///
    /// Returns `None` if no asset is associated with the path.
    pub fn get_handle_by_path(&self, path: &str) -> Option<AssetHandle<A>> {
        self.path_index.get(path).copied().filter(|h| self.allocator.is_alive(*h))
    }

    /// Gets an asset by path.
    pub fn get_by_path(&self, path: &str) -> Option<&A> {
        self.get_handle_by_path(path)
            .and_then(|h| self.get(&h))
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
        self.allocator.is_alive(handle)
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
        self.get_entry(&typed)
            .and_then(|e| e.path().cloned())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

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
    pub fn insert_with_path<A: Asset>(
        &mut self,
        asset: A,
        path: AssetPath<'_>,
    ) -> AssetHandle<A> {
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
        self.get_storage::<A>()
            .into_iter()
            .flat_map(|s| s.iter())
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetType;

    // Test asset types
    #[derive(Clone, Debug, PartialEq)]
    struct TestTexture {
        width: u32,
        height: u32,
    }

    impl Asset for TestTexture {
        fn asset_type_name() -> &'static str {
            "TestTexture"
        }

        fn asset_type() -> AssetType {
            AssetType::Texture
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct TestAudio {
        duration: f32,
    }

    impl Asset for TestAudio {
        fn asset_type_name() -> &'static str {
            "TestAudio"
        }

        fn asset_type() -> AssetType {
            AssetType::Audio
        }
    }

    #[derive(Clone, Debug, PartialEq, Default)]
    struct SimpleAsset {
        value: i32,
    }

    impl Asset for SimpleAsset {}

    // =========================================================================
    // AssetEntry Tests
    // =========================================================================

    mod asset_entry {
        use super::*;

        #[test]
        fn test_empty() {
            let entry: AssetEntry<TestTexture> = AssetEntry::empty();
            assert!(entry.asset().is_none());
            assert!(!entry.is_loaded());
            assert!(!entry.is_loading());
            assert!(!entry.is_failed());
            assert!(entry.path().is_none());
        }

        #[test]
        fn test_loading() {
            let entry: AssetEntry<TestTexture> = AssetEntry::loading(0.5);
            assert!(entry.is_loading());
            assert!(!entry.is_loaded());
            assert_eq!(entry.state().progress(), Some(0.5));
        }

        #[test]
        fn test_loaded() {
            let entry = AssetEntry::loaded(TestTexture {
                width: 256,
                height: 256,
            });
            assert!(entry.is_loaded());
            assert!(!entry.is_loading());
            assert_eq!(entry.asset().unwrap().width, 256);
        }

        #[test]
        fn test_with_path() {
            let entry = AssetEntry::with_path(
                TestTexture {
                    width: 512,
                    height: 512,
                },
                AssetPath::new("textures/player.png"),
            );
            assert!(entry.is_loaded());
            assert_eq!(
                entry.path().map(|p| p.as_str()),
                Some("textures/player.png")
            );
        }

        #[test]
        fn test_failed() {
            let entry: AssetEntry<TestTexture> = AssetEntry::failed("File not found");
            assert!(entry.is_failed());
            assert!(!entry.is_loaded());
            assert_eq!(entry.state().error(), Some("File not found"));
        }

        #[test]
        fn test_asset_mut() {
            let mut entry = AssetEntry::loaded(TestTexture {
                width: 256,
                height: 256,
            });
            if let Some(asset) = entry.asset_mut() {
                asset.width = 512;
            }
            assert_eq!(entry.asset().unwrap().width, 512);
        }

        #[test]
        fn test_take_asset() {
            let mut entry = AssetEntry::loaded(TestTexture {
                width: 256,
                height: 256,
            });
            let asset = entry.take_asset();
            assert!(asset.is_some());
            assert!(entry.asset().is_none());
            assert_eq!(entry.state().discriminant(), AssetState::Unloaded.discriminant());
        }

        #[test]
        fn test_set_path() {
            let mut entry = AssetEntry::loaded(TestTexture {
                width: 256,
                height: 256,
            });
            entry.set_path(AssetPath::from_string("new/path.png".to_string()));
            assert_eq!(entry.path().map(|p| p.as_str()), Some("new/path.png"));
        }

        #[test]
        fn test_clear_path() {
            let mut entry = AssetEntry::with_path(
                TestTexture {
                    width: 256,
                    height: 256,
                },
                AssetPath::new("textures/player.png"),
            );
            entry.clear_path();
            assert!(entry.path().is_none());
        }

        #[test]
        fn test_set_loaded() {
            let mut entry: AssetEntry<TestTexture> = AssetEntry::loading(0.5);
            entry.set_loaded(TestTexture {
                width: 256,
                height: 256,
            });
            assert!(entry.is_loaded());
            assert_eq!(entry.asset().unwrap().width, 256);
        }

        #[test]
        fn test_set_progress() {
            let mut entry: AssetEntry<TestTexture> = AssetEntry::loading(0.0);
            entry.set_progress(0.75);
            assert_eq!(entry.state().progress(), Some(0.75));
        }

        #[test]
        fn test_set_failed() {
            let mut entry = AssetEntry::loaded(TestTexture {
                width: 256,
                height: 256,
            });
            entry.set_failed("Error occurred");
            assert!(entry.is_failed());
            assert!(entry.asset().is_none());
        }

        #[test]
        fn test_set_unloaded() {
            let mut entry = AssetEntry::loaded(TestTexture {
                width: 256,
                height: 256,
            });
            entry.set_unloaded();
            assert!(entry.asset().is_none());
            assert_eq!(entry.state().discriminant(), AssetState::Unloaded.discriminant());
        }

        #[test]
        fn test_default() {
            let entry: AssetEntry<TestTexture> = AssetEntry::default();
            assert!(entry.asset().is_none());
            assert!(!entry.is_loaded());
        }

        #[test]
        fn test_clone() {
            let entry = AssetEntry::loaded(TestTexture {
                width: 256,
                height: 256,
            });
            let cloned = entry.clone();
            assert_eq!(entry.asset(), cloned.asset());
        }

        #[test]
        fn test_debug() {
            let entry = AssetEntry::loaded(TestTexture {
                width: 256,
                height: 256,
            });
            let debug_str = format!("{:?}", entry);
            assert!(debug_str.contains("AssetEntry"));
        }
    }

    // =========================================================================
    // TypedAssetStorage Tests
    // =========================================================================

    mod typed_asset_storage {
        use super::*;

        #[test]
        fn test_new() {
            let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            assert!(storage.is_empty());
            assert_eq!(storage.len(), 0);
        }

        #[test]
        fn test_with_capacity() {
            let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::with_capacity(100);
            assert!(storage.is_empty());
        }

        #[test]
        fn test_insert() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            let handle = storage.insert(TestTexture {
                width: 256,
                height: 256,
            });

            assert!(handle.is_valid());
            assert!(storage.is_alive(&handle));
            assert_eq!(storage.len(), 1);
        }

        #[test]
        fn test_insert_multiple() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let h1 = storage.insert(TestTexture { width: 256, height: 256 });
            let h2 = storage.insert(TestTexture { width: 512, height: 512 });
            let h3 = storage.insert(TestTexture { width: 1024, height: 1024 });

            assert_ne!(h1, h2);
            assert_ne!(h2, h3);
            assert_eq!(storage.len(), 3);
        }

        #[test]
        fn test_insert_with_path() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let h1 = storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );

            assert!(storage.is_alive(&h1));
            assert!(storage.has_path("textures/player.png"));
        }

        #[test]
        fn test_insert_with_path_deduplication() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let h1 = storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );

            let h2 = storage.insert_with_path(
                TestTexture { width: 512, height: 512 }, // Different asset
                AssetPath::new("textures/player.png"),   // Same path
            );

            // Should return existing handle
            assert_eq!(h1, h2);
            assert_eq!(storage.len(), 1);
            // Original asset preserved
            assert_eq!(storage.get(&h1).unwrap().width, 256);
        }

        #[test]
        fn test_reserve() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.reserve();
            assert!(storage.is_alive(&handle));
            assert!(storage.get(&handle).is_none()); // Not loaded yet
            assert_eq!(storage.len(), 1);
        }

        #[test]
        fn test_reserve_with_path() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.reserve_with_path(AssetPath::new("textures/player.png"));
            assert!(storage.is_alive(&handle));
            assert!(storage.has_path("textures/player.png"));
            assert!(storage.get(&handle).is_none());
        }

        #[test]
        fn test_set_loaded() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.reserve();
            assert!(storage.get(&handle).is_none());

            let result = storage.set_loaded(&handle, TestTexture { width: 256, height: 256 });
            assert!(result);
            assert_eq!(storage.get(&handle).unwrap().width, 256);
        }

        #[test]
        fn test_set_loaded_invalid_handle() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let result = storage.set_loaded(
                &AssetHandle::INVALID,
                TestTexture { width: 256, height: 256 },
            );
            assert!(!result);
        }

        #[test]
        fn test_remove() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let removed = storage.remove(&handle);

            assert!(removed.is_some());
            assert_eq!(removed.unwrap().width, 256);
            assert!(!storage.is_alive(&handle));
            assert_eq!(storage.len(), 0);
        }

        #[test]
        fn test_remove_with_path() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );

            storage.remove(&handle);
            assert!(!storage.has_path("textures/player.png"));
        }

        #[test]
        fn test_remove_invalid() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let removed = storage.remove(&AssetHandle::INVALID);
            assert!(removed.is_none());
        }

        #[test]
        fn test_get() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let asset = storage.get(&handle);

            assert!(asset.is_some());
            assert_eq!(asset.unwrap().width, 256);
        }

        #[test]
        fn test_get_invalid() {
            let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            assert!(storage.get(&AssetHandle::INVALID).is_none());
        }

        #[test]
        fn test_get_stale() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            storage.remove(&handle);

            assert!(storage.get(&handle).is_none());
        }

        #[test]
        fn test_get_mut() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            if let Some(asset) = storage.get_mut(&handle) {
                asset.width = 512;
            }

            assert_eq!(storage.get(&handle).unwrap().width, 512);
        }

        #[test]
        fn test_get_entry() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let entry = storage.get_entry(&handle);

            assert!(entry.is_some());
            assert!(entry.unwrap().is_loaded());
        }

        #[test]
        fn test_get_state() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let state = storage.get_state(&handle);

            assert!(state.is_some());
            assert!(state.unwrap().is_ready());
        }

        #[test]
        fn test_get_handle_by_path() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );

            let found = storage.get_handle_by_path("textures/player.png");
            assert_eq!(found, Some(handle));

            assert!(storage.get_handle_by_path("nonexistent.png").is_none());
        }

        #[test]
        fn test_get_by_path() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );

            let asset = storage.get_by_path("textures/player.png");
            assert!(asset.is_some());
            assert_eq!(asset.unwrap().width, 256);
        }

        #[test]
        fn test_set_path() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let result = storage.set_path(&handle, AssetPath::new("textures/player.png"));

            assert!(result);
            assert!(storage.has_path("textures/player.png"));
            assert_eq!(storage.get_handle_by_path("textures/player.png"), Some(handle));
        }

        #[test]
        fn test_set_path_replaces_old() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("old/path.png"),
            );

            storage.set_path(&handle, AssetPath::new("new/path.png"));

            assert!(!storage.has_path("old/path.png"));
            assert!(storage.has_path("new/path.png"));
        }

        #[test]
        fn test_clear_path() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let handle = storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );

            storage.clear_path(&handle);
            assert!(!storage.has_path("textures/player.png"));
        }

        #[test]
        fn test_clear() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let h1 = storage.insert(TestTexture { width: 256, height: 256 });
            let h2 = storage.insert(TestTexture { width: 512, height: 512 });

            storage.clear();

            assert!(!storage.is_alive(&h1));
            assert!(!storage.is_alive(&h2));
            assert_eq!(storage.len(), 0);
        }

        #[test]
        fn test_iter() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            storage.insert(TestTexture { width: 256, height: 256 });
            storage.insert(TestTexture { width: 512, height: 512 });

            let pairs: Vec<_> = storage.iter().collect();
            assert_eq!(pairs.len(), 2);
        }

        #[test]
        fn test_handles() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let h1 = storage.insert(TestTexture { width: 256, height: 256 });
            let h2 = storage.insert(TestTexture { width: 512, height: 512 });

            let handles: Vec<_> = storage.handles().collect();
            assert!(handles.contains(&h1));
            assert!(handles.contains(&h2));
        }

        #[test]
        fn test_paths() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/a.png"),
            );
            storage.insert_with_path(
                TestTexture { width: 512, height: 512 },
                AssetPath::new("textures/b.png"),
            );

            let paths: Vec<_> = storage.paths().collect();
            assert!(paths.contains(&"textures/a.png"));
            assert!(paths.contains(&"textures/b.png"));
            assert_eq!(storage.path_count(), 2);
        }

        #[test]
        fn test_default() {
            let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::default();
            assert!(storage.is_empty());
        }

        #[test]
        fn test_debug() {
            let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            let debug_str = format!("{:?}", storage);
            assert!(debug_str.contains("TypedAssetStorage"));
            assert!(debug_str.contains("TestTexture"));
        }

        #[test]
        fn test_slot_reuse() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            // Insert and remove to create free slot
            let h1 = storage.insert(TestTexture { width: 256, height: 256 });
            storage.remove(&h1);

            // New handle should have same index but different generation
            let h2 = storage.insert(TestTexture { width: 512, height: 512 });
            assert_eq!(h1.index(), h2.index());
            assert_ne!(h1.generation(), h2.generation());
        }

        #[test]
        fn test_stale_path_index() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

            let h1 = storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );

            storage.remove(&h1);

            // Path should not return stale handle
            assert!(storage.get_handle_by_path("textures/player.png").is_none());

            // Can insert with same path again
            let h2 = storage.insert_with_path(
                TestTexture { width: 512, height: 512 },
                AssetPath::new("textures/player.png"),
            );

            assert!(storage.get_handle_by_path("textures/player.png").is_some());
            assert_ne!(h1, h2);
        }
    }

    // =========================================================================
    // AssetStorage Tests
    // =========================================================================

    mod asset_storage {
        use super::*;

        #[test]
        fn test_new() {
            let storage = AssetStorage::new();
            assert!(storage.is_empty());
            assert_eq!(storage.type_count(), 0);
        }

        #[test]
        fn test_insert() {
            let mut storage = AssetStorage::new();
            let handle = storage.insert(TestTexture { width: 256, height: 256 });

            assert!(handle.is_valid());
            assert!(storage.is_alive(&handle));
            assert_eq!(storage.type_count(), 1);
        }

        #[test]
        fn test_insert_multiple_types() {
            let mut storage = AssetStorage::new();

            let tex_handle = storage.insert(TestTexture { width: 256, height: 256 });
            let audio_handle = storage.insert(TestAudio { duration: 2.5 });

            assert!(storage.is_alive(&tex_handle));
            assert!(storage.is_alive(&audio_handle));
            assert_eq!(storage.type_count(), 2);
            assert_eq!(storage.total_len(), 2);
        }

        #[test]
        fn test_insert_with_path() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );

            assert!(storage.has_path::<TestTexture>("textures/player.png"));
            assert_eq!(
                storage.get_handle_by_path::<TestTexture>("textures/player.png"),
                Some(handle)
            );
        }

        #[test]
        fn test_reserve() {
            let mut storage = AssetStorage::new();

            let handle = storage.reserve::<TestTexture>();
            assert!(storage.is_alive(&handle));
            assert!(storage.get::<TestTexture>(&handle).is_none());
        }

        #[test]
        fn test_set_loaded() {
            let mut storage = AssetStorage::new();

            let handle = storage.reserve::<TestTexture>();
            storage.set_loaded(&handle, TestTexture { width: 256, height: 256 });

            assert_eq!(storage.get::<TestTexture>(&handle).unwrap().width, 256);
        }

        #[test]
        fn test_remove() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let removed = storage.remove::<TestTexture>(&handle);

            assert!(removed.is_some());
            assert!(!storage.is_alive(&handle));
        }

        #[test]
        fn test_remove_untyped() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let untyped = handle.untyped();

            let result = storage.remove_untyped(&untyped);
            assert!(result);
            assert!(!storage.is_alive(&handle));
        }

        #[test]
        fn test_get() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let asset = storage.get::<TestTexture>(&handle);

            assert!(asset.is_some());
            assert_eq!(asset.unwrap().width, 256);
        }

        #[test]
        fn test_get_mut() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            if let Some(asset) = storage.get_mut::<TestTexture>(&handle) {
                asset.width = 512;
            }

            assert_eq!(storage.get::<TestTexture>(&handle).unwrap().width, 512);
        }

        #[test]
        fn test_get_entry() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let entry = storage.get_entry::<TestTexture>(&handle);

            assert!(entry.is_some());
            assert!(entry.unwrap().is_loaded());
        }

        #[test]
        fn test_is_alive_untyped() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let untyped = handle.untyped();

            assert!(storage.is_alive_untyped(&untyped));

            storage.remove_untyped(&untyped);
            assert!(!storage.is_alive_untyped(&untyped));
        }

        #[test]
        fn test_get_state() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let state = storage.get_state::<TestTexture>(&handle);

            assert!(state.is_some());
            assert!(state.unwrap().is_ready());
        }

        #[test]
        fn test_get_state_untyped() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let untyped = handle.untyped();

            let state = storage.get_state_untyped(&untyped);
            assert!(state.is_some());
            assert!(state.unwrap().is_ready());
        }

        #[test]
        fn test_get_by_path() {
            let mut storage = AssetStorage::new();

            storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );

            let asset = storage.get_by_path::<TestTexture>("textures/player.png");
            assert!(asset.is_some());
            assert_eq!(asset.unwrap().width, 256);
        }

        #[test]
        fn test_set_path() {
            let mut storage = AssetStorage::new();

            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            storage.set_path(&handle, AssetPath::new("textures/player.png"));

            assert!(storage.has_path::<TestTexture>("textures/player.png"));
        }

        #[test]
        fn test_len() {
            let mut storage = AssetStorage::new();

            storage.insert(TestTexture { width: 256, height: 256 });
            storage.insert(TestTexture { width: 512, height: 512 });
            storage.insert(TestAudio { duration: 2.5 });

            assert_eq!(storage.len::<TestTexture>(), 2);
            assert_eq!(storage.len::<TestAudio>(), 1);
            assert_eq!(storage.total_len(), 3);
        }

        #[test]
        fn test_is_empty_type() {
            let mut storage = AssetStorage::new();

            assert!(storage.is_empty_type::<TestTexture>());

            storage.insert(TestTexture { width: 256, height: 256 });
            assert!(!storage.is_empty_type::<TestTexture>());
            assert!(storage.is_empty_type::<TestAudio>());
        }

        #[test]
        fn test_clear_type() {
            let mut storage = AssetStorage::new();

            let tex_handle = storage.insert(TestTexture { width: 256, height: 256 });
            storage.insert(TestAudio { duration: 2.5 });

            storage.clear_type::<TestTexture>();

            assert!(!storage.is_alive(&tex_handle));
            assert_eq!(storage.len::<TestTexture>(), 0);
            assert_eq!(storage.len::<TestAudio>(), 1);
        }

        #[test]
        fn test_clear() {
            let mut storage = AssetStorage::new();

            storage.insert(TestTexture { width: 256, height: 256 });
            storage.insert(TestAudio { duration: 2.5 });

            storage.clear();

            assert_eq!(storage.total_len(), 0);
        }

        #[test]
        fn test_has_type() {
            let mut storage = AssetStorage::new();

            assert!(!storage.has_type::<TestTexture>());

            storage.insert(TestTexture { width: 256, height: 256 });
            assert!(storage.has_type::<TestTexture>());
        }

        #[test]
        fn test_registered_types() {
            let mut storage = AssetStorage::new();

            storage.insert(TestTexture { width: 256, height: 256 });
            storage.insert(TestAudio { duration: 2.5 });

            let types = storage.registered_types();
            assert_eq!(types.len(), 2);
        }

        #[test]
        fn test_get_storage() {
            let mut storage = AssetStorage::new();

            storage.insert(TestTexture { width: 256, height: 256 });

            let typed_storage = storage.get_storage::<TestTexture>();
            assert!(typed_storage.is_some());
            assert_eq!(typed_storage.unwrap().len(), 1);
        }

        #[test]
        fn test_get_or_create_storage() {
            let mut storage = AssetStorage::new();

            // Should create storage on first access
            let typed = storage.get_or_create_storage::<TestTexture>();
            typed.insert(TestTexture { width: 256, height: 256 });

            assert_eq!(storage.len::<TestTexture>(), 1);
        }

        #[test]
        fn test_iter() {
            let mut storage = AssetStorage::new();

            storage.insert(TestTexture { width: 256, height: 256 });
            storage.insert(TestTexture { width: 512, height: 512 });

            let pairs: Vec<_> = storage.iter::<TestTexture>().collect();
            assert_eq!(pairs.len(), 2);
        }

        #[test]
        fn test_handles() {
            let mut storage = AssetStorage::new();

            let h1 = storage.insert(TestTexture { width: 256, height: 256 });
            let h2 = storage.insert(TestTexture { width: 512, height: 512 });

            let handles: Vec<_> = storage.handles::<TestTexture>().collect();
            assert!(handles.contains(&h1));
            assert!(handles.contains(&h2));
        }

        #[test]
        fn test_default() {
            let storage = AssetStorage::default();
            assert!(storage.is_empty());
        }

        #[test]
        fn test_debug() {
            let storage = AssetStorage::new();
            let debug_str = format!("{:?}", storage);
            assert!(debug_str.contains("AssetStorage"));
        }

        #[test]
        fn test_type_isolation() {
            let mut storage = AssetStorage::new();

            // Insert same index for different types
            let tex_handle = storage.insert(TestTexture { width: 256, height: 256 });
            let audio_handle = storage.insert(TestAudio { duration: 2.5 });

            // Should not interfere with each other
            storage.remove::<TestTexture>(&tex_handle);

            assert!(!storage.is_alive(&tex_handle));
            assert!(storage.is_alive(&audio_handle));
        }

        #[test]
        fn test_stress_multiple_types() {
            let mut storage = AssetStorage::new();

            // Insert many assets of multiple types
            for i in 0..1000 {
                storage.insert(TestTexture {
                    width: i,
                    height: i,
                });
                storage.insert(TestAudio { duration: i as f32 });
            }

            assert_eq!(storage.len::<TestTexture>(), 1000);
            assert_eq!(storage.len::<TestAudio>(), 1000);
            assert_eq!(storage.total_len(), 2000);
        }
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_typed_storage_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<TypedAssetStorage<TestTexture>>();
        }

        #[test]
        fn test_typed_storage_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<TypedAssetStorage<TestTexture>>();
        }

        #[test]
        fn test_asset_storage_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetStorage>();
        }

        #[test]
        fn test_asset_storage_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<AssetStorage>();
        }

        #[test]
        fn test_asset_entry_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetEntry<TestTexture>>();
        }

        #[test]
        fn test_asset_entry_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<AssetEntry<TestTexture>>();
        }
    }

    // =========================================================================
    // AnyAssetStorage Tests
    // =========================================================================

    mod any_asset_storage {
        use super::*;

        #[test]
        fn test_asset_id() {
            let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            let any_storage: &dyn AnyAssetStorage = &storage;

            assert_eq!(any_storage.asset_id(), AssetId::of::<TestTexture>());
        }

        #[test]
        fn test_asset_info() {
            let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            let any_storage: &dyn AnyAssetStorage = &storage;

            let info = any_storage.asset_info();
            assert_eq!(info.name, "TestTexture");
            assert_eq!(info.asset_type, AssetType::Texture);
        }

        #[test]
        fn test_len_and_is_empty() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            let any_storage: &dyn AnyAssetStorage = &storage;

            assert!(any_storage.is_empty());
            assert_eq!(any_storage.len(), 0);

            storage.insert(TestTexture { width: 256, height: 256 });
            let any_storage: &dyn AnyAssetStorage = &storage;

            assert!(!any_storage.is_empty());
            assert_eq!(any_storage.len(), 1);
        }

        #[test]
        fn test_is_alive_raw() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            let handle = storage.insert(TestTexture { width: 256, height: 256 });

            let any_storage: &dyn AnyAssetStorage = &storage;
            assert!(any_storage.is_alive_raw(handle.index(), handle.generation()));
            assert!(!any_storage.is_alive_raw(handle.index(), handle.generation() + 1));
        }

        #[test]
        fn test_remove_untyped() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let untyped = handle.untyped();

            let any_storage: &mut dyn AnyAssetStorage = &mut storage;
            assert!(any_storage.remove_untyped(&untyped));
            assert!(!any_storage.is_alive_raw(handle.index(), handle.generation()));
        }

        #[test]
        fn test_remove_untyped_wrong_type() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            storage.insert(TestTexture { width: 256, height: 256 });

            // Create untyped handle with wrong type
            let wrong_untyped = UntypedAssetHandle::new(0, 1, AssetId::of::<TestAudio>());

            let any_storage: &mut dyn AnyAssetStorage = &mut storage;
            assert!(!any_storage.remove_untyped(&wrong_untyped));
        }

        #[test]
        fn test_get_state_untyped() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            let handle = storage.insert(TestTexture { width: 256, height: 256 });
            let untyped = handle.untyped();

            let any_storage: &dyn AnyAssetStorage = &storage;
            let state = any_storage.get_state_untyped(&untyped);
            assert!(state.is_some());
            assert!(state.unwrap().is_ready());
        }

        #[test]
        fn test_get_path_untyped() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            let handle = storage.insert_with_path(
                TestTexture { width: 256, height: 256 },
                AssetPath::new("textures/player.png"),
            );
            let untyped = handle.untyped();

            let any_storage: &dyn AnyAssetStorage = &storage;
            let path = any_storage.get_path_untyped(&untyped);
            assert!(path.is_some());
            assert_eq!(path.unwrap().as_str(), "textures/player.png");
        }

        #[test]
        fn test_as_any_downcast() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            storage.insert(TestTexture { width: 256, height: 256 });

            let any_storage: &dyn AnyAssetStorage = &storage;
            let downcasted = any_storage
                .as_any()
                .downcast_ref::<TypedAssetStorage<TestTexture>>();

            assert!(downcasted.is_some());
            assert_eq!(downcasted.unwrap().len(), 1);
        }

        #[test]
        fn test_clear() {
            let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
            storage.insert(TestTexture { width: 256, height: 256 });

            let any_storage: &mut dyn AnyAssetStorage = &mut storage;
            any_storage.clear();

            assert!(any_storage.is_empty());
        }
    }
}
