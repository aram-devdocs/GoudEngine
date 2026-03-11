//! Asset loading and access operations for `AssetServer`.

use super::core::AssetServer;
use crate::assets::{Asset, AssetHandle, AssetLoadError, AssetPath, AssetState, LoadContext};
#[cfg(feature = "native")]
use crate::assets::{HotReloadConfig, HotReloadWatcher};
use std::path::Path;

impl AssetServer {
    /// Loads an asset from a path (relative to asset root), returning a handle immediately.
    ///
    /// The asset loads synchronously (blocking). The handle is valid even if loading
    /// fails -- check with `get_load_state()`.
    pub fn load<A: Asset>(&mut self, path: impl AsRef<Path>) -> AssetHandle<A> {
        let asset_path = AssetPath::new(path.as_ref().to_str().unwrap_or_default());

        // Check if already loaded
        if let Some(handle) = self.storage.get_handle_by_path::<A>(asset_path.as_str()) {
            return handle;
        }

        // Reserve a handle for this asset
        let handle = self.storage.reserve_with_path::<A>(asset_path.clone());

        // Load the asset synchronously
        match self.load_asset_sync::<A>(&asset_path) {
            Ok((asset, dependencies)) => {
                self.storage.set_loaded(&handle, asset);
                // Record dependencies in the graph
                let asset_str = asset_path.as_str().to_string();
                for dep in &dependencies {
                    if let Err(e) = self.dependency_graph.add_dependency(&asset_str, dep) {
                        log::warn!("Dependency cycle detected loading '{}': {}", asset_str, e);
                    }
                }
            }
            Err(error) => {
                log::warn!("Failed to load asset '{}': {}", asset_path.as_str(), error);
                // Attempt fallback substitution
                if let Some(fallback) = self.fallbacks.get_cloned::<A>() {
                    if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
                        entry.set_loaded(fallback);
                        entry.set_fallback(true);
                    }
                } else if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
                    entry.set_failed(error.to_string());
                }
            }
        }

        handle
    }

    /// Runs raw bytes through the registered loader for the given asset path's extension.
    ///
    /// Returns the loaded asset and any dependencies declared by the loader.
    pub(super) fn parse_bytes<A: Asset>(
        &self,
        path: &AssetPath,
        bytes: &[u8],
    ) -> Result<(A, Vec<String>), AssetLoadError> {
        let extension = path
            .extension()
            .ok_or_else(|| AssetLoadError::unsupported_format(""))?;

        // Try compound extension first (e.g. "mat.json", "anim.json"), then simple
        let loader = self
            .find_loader_for_path(path, extension)
            .ok_or_else(|| AssetLoadError::unsupported_format(extension))?;

        let mut context = LoadContext::new(path.clone().into_owned());
        let boxed = loader.load_erased(bytes, &mut context)?;

        let dependencies = context.into_dependencies();

        let asset = boxed
            .downcast::<A>()
            .map(|boxed| *boxed)
            .map_err(|_| AssetLoadError::custom("Type mismatch after loading"))?;

        Ok((asset, dependencies))
    }

    /// Reads a file via the virtual filesystem and parses it into an asset.
    fn load_asset_sync<A: Asset>(
        &self,
        path: &AssetPath,
    ) -> Result<(A, Vec<String>), AssetLoadError> {
        let bytes = self.vfs.read(path.as_str())?;
        self.parse_bytes::<A>(path, &bytes)
    }

    /// Loads an asset from pre-fetched bytes, returning a handle.
    ///
    /// This is the platform-agnostic entry point for loading assets when you
    /// already have the raw bytes (e.g., from a web fetch, embedded resource,
    /// or custom I/O layer). The bytes are run through the registered loader
    /// matched by the path's file extension.
    ///
    /// Returns an existing handle if an asset with the same path is already loaded.
    ///
    /// # Arguments
    ///
    /// * `path` - Logical asset path (used for loader lookup and deduplication)
    /// * `bytes` - Raw asset bytes to parse
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetServer, AssetLoader, LoadContext, AssetLoadError};
    ///
    /// #[derive(Clone)]
    /// struct JsonAsset { data: String }
    /// impl Asset for JsonAsset {}
    ///
    /// #[derive(Clone)]
    /// struct JsonLoader;
    /// impl AssetLoader for JsonLoader {
    ///     type Asset = JsonAsset;
    ///     type Settings = ();
    ///     fn extensions(&self) -> &[&str] { &["json"] }
    ///     fn load<'a>(
    ///         &'a self, bytes: &'a [u8], _: &'a (), _: &'a mut LoadContext,
    ///     ) -> Result<Self::Asset, AssetLoadError> {
    ///         Ok(JsonAsset { data: String::from_utf8_lossy(bytes).into_owned() })
    ///     }
    /// }
    ///
    /// let mut server = AssetServer::new();
    /// server.register_loader(JsonLoader);
    ///
    /// let bytes = br#"{"key": "value"}"#;
    /// let handle = server.load_from_bytes::<JsonAsset>("config.json", bytes);
    /// assert!(server.is_loaded(&handle));
    /// ```
    pub fn load_from_bytes<A: Asset>(
        &mut self,
        path: impl AsRef<Path>,
        bytes: &[u8],
    ) -> AssetHandle<A> {
        let asset_path = AssetPath::new(path.as_ref().to_str().unwrap_or_default());

        if let Some(handle) = self.storage.get_handle_by_path::<A>(asset_path.as_str()) {
            return handle;
        }

        let handle = self.storage.reserve_with_path::<A>(asset_path.clone());

        match self.parse_bytes::<A>(&asset_path, bytes) {
            Ok((asset, dependencies)) => {
                self.storage.set_loaded(&handle, asset);
                // Record dependencies in the graph
                let asset_str = asset_path.as_str().to_string();
                for dep in &dependencies {
                    if let Err(e) = self.dependency_graph.add_dependency(&asset_str, dep) {
                        log::warn!("Dependency cycle detected loading '{}': {}", asset_str, e);
                    }
                }
            }
            Err(error) => {
                log::warn!(
                    "Failed to load asset from bytes '{}': {}",
                    asset_path.as_str(),
                    error
                );
                if let Some(fallback) = self.fallbacks.get_cloned::<A>() {
                    if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
                        entry.set_loaded(fallback);
                        entry.set_fallback(true);
                    }
                } else if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
                    entry.set_failed(error.to_string());
                }
            }
        }

        handle
    }

    /// Gets a reference to a loaded asset.
    ///
    /// Returns `None` if the asset is not loaded or the handle is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetServer};
    ///
    /// struct MyAsset { value: i32 }
    /// impl Asset for MyAsset {}
    ///
    /// let mut server = AssetServer::new();
    /// let handle = server.load::<MyAsset>("data/config.json");
    ///
    /// if let Some(asset) = server.get(&handle) {
    ///     println!("Asset value: {}", asset.value);
    /// }
    /// ```
    #[inline]
    pub fn get<A: Asset>(&self, handle: &AssetHandle<A>) -> Option<&A> {
        self.storage.get(handle)
    }

    /// Gets a mutable reference to a loaded asset.
    ///
    /// Returns `None` if the asset is not loaded or the handle is invalid.
    #[inline]
    pub fn get_mut<A: Asset>(&mut self, handle: &AssetHandle<A>) -> Option<&mut A> {
        self.storage.get_mut(handle)
    }

    /// Returns true if the handle points to a valid, loaded asset.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetServer, AssetHandle};
    ///
    /// struct MyAsset;
    /// impl Asset for MyAsset {}
    ///
    /// let server = AssetServer::new();
    /// let handle = AssetHandle::<MyAsset>::INVALID;
    ///
    /// assert!(!server.is_loaded(&handle));
    /// ```
    pub fn is_loaded<A: Asset>(&self, handle: &AssetHandle<A>) -> bool {
        self.storage
            .get_state(handle)
            .map(|s| s.is_ready())
            .unwrap_or(false)
    }

    /// Returns the loading state for a handle, or `None` if not tracked.
    #[inline]
    pub fn get_load_state<A: Asset>(&self, handle: &AssetHandle<A>) -> Option<AssetState> {
        self.storage.get_state(handle)
    }

    /// Unloads an asset, freeing its memory.
    ///
    /// The handle becomes invalid after this call.
    ///
    /// # Returns
    ///
    /// The unloaded asset if it was loaded, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetServer};
    ///
    /// struct Texture { width: u32 }
    /// impl Asset for Texture {}
    ///
    /// let mut server = AssetServer::new();
    /// let handle = server.load::<Texture>("texture.png");
    ///
    /// // Later, unload to free memory
    /// let texture = server.unload(&handle);
    /// assert!(!server.is_loaded(&handle));
    /// ```
    pub fn unload<A: Asset>(&mut self, handle: &AssetHandle<A>) -> Option<A> {
        // Clean up dependency graph for unloaded asset
        if let Some(entry) = self.storage.get_entry(handle) {
            if let Some(path) = entry.path() {
                let path_str = path.as_str().to_string();
                self.dependency_graph.remove_asset(&path_str);
            }
        }
        self.storage.force_remove(handle)
    }

    /// Returns a reference to the dependency graph.
    ///
    /// Useful for inspecting asset relationships or computing
    /// cascade reload orders externally.
    pub fn dependency_graph(&self) -> &crate::assets::dependency::DependencyGraph {
        &self.dependency_graph
    }

    /// Returns a mutable reference to the dependency graph.
    pub fn dependency_graph_mut(&mut self) -> &mut crate::assets::dependency::DependencyGraph {
        &mut self.dependency_graph
    }

    /// Reloads an asset by its path, using the type-erased loader.
    ///
    /// This is used by the hot-reload watcher to reload assets without knowing
    /// their concrete type at compile time.
    ///
    /// # Returns
    ///
    /// `true` if the asset was successfully reloaded, `false` if the path
    /// has no registered loader or the reload failed.
    #[cfg(feature = "native")]
    pub fn reload_by_path(&mut self, path: &str) -> bool {
        let asset_path = AssetPath::new(path);
        let extension = match asset_path.extension() {
            Some(ext) => ext.to_string(),
            None => return false,
        };

        // Check if we have a loader for this extension
        let loader = match self.loaders.get(&extension) {
            Some(l) => l.clone_boxed(),
            None => return false,
        };

        // Read file via virtual filesystem
        let bytes = match self.vfs.read(path) {
            Ok(b) => b,
            Err(_) => return false,
        };

        // Parse using erased loader
        let mut context = LoadContext::new(asset_path.into_owned());
        match loader.load_erased(&bytes, &mut context) {
            Ok(boxed_asset) => {
                // Update in storage if the asset exists
                self.storage.replace_erased(path, boxed_asset);

                // Update dependency graph with new dependencies from reload
                let new_deps = context.into_dependencies();
                let path_str = path.to_string();
                self.dependency_graph.remove_asset(&path_str);
                for dep in &new_deps {
                    if let Err(e) = self.dependency_graph.add_dependency(&path_str, dep) {
                        log::warn!(
                            "Dependency cycle detected during hot-reload of '{}': {}",
                            path,
                            e
                        );
                    }
                }

                true
            }
            Err(_) => false,
        }
    }

    /// Returns the cascade reload order for a changed asset path.
    ///
    /// This delegates to
    /// [`DependencyGraph::get_cascade_order`](crate::assets::dependency::DependencyGraph::get_cascade_order)
    /// and
    /// returns the list of asset paths that should be reloaded, in
    /// the correct order, when `changed_path` changes.
    pub fn get_cascade_order(&self, changed_path: &str) -> Vec<String> {
        self.dependency_graph.get_cascade_order(changed_path)
    }

    /// Finds a loader for a path, trying compound extensions first.
    ///
    /// For a path like `"brick.mat.json"`, tries `"mat.json"` first,
    /// then falls back to `"json"`.
    fn find_loader_for_path(
        &self,
        path: &AssetPath,
        simple_ext: &str,
    ) -> Option<&dyn crate::assets::ErasedAssetLoader> {
        // Try compound extension (everything after first dot in filename)
        if let Some(file_name) = path.file_name() {
            if let Some(first_dot) = file_name.find('.') {
                let compound_ext = &file_name[first_dot + 1..];
                if compound_ext != simple_ext {
                    if let Some(loader) = self.loaders.get(compound_ext) {
                        return Some(loader.as_ref());
                    }
                }
            }
        }
        // Fall back to simple extension
        self.loaders.get(simple_ext).map(|b| b.as_ref())
    }

    /// Returns the number of loaded assets of a specific type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetServer};
    ///
    /// struct Texture;
    /// impl Asset for Texture {}
    ///
    /// let server = AssetServer::new();
    /// assert_eq!(server.loaded_count::<Texture>(), 0);
    /// ```
    #[inline]
    pub fn loaded_count<A: Asset>(&self) -> usize {
        self.storage.len::<A>()
    }

    /// Returns the total number of loaded assets across all types.
    #[inline]
    pub fn total_loaded_count(&self) -> usize {
        self.storage.total_len()
    }

    /// Returns the number of registered asset types.
    #[inline]
    pub fn registered_type_count(&self) -> usize {
        self.storage.type_count()
    }

    /// Clears all loaded assets of a specific type.
    ///
    /// This frees memory but does not affect registered loaders.
    pub fn clear_type<A: Asset>(&mut self) {
        self.storage.clear_type::<A>();
    }

    /// Clears all loaded assets from all types.
    ///
    /// This frees memory but does not affect registered loaders.
    pub fn clear(&mut self) {
        self.storage.clear();
    }

    /// Returns an iterator over all loaded asset handles of a specific type.
    pub fn handles<A: Asset>(&self) -> impl Iterator<Item = AssetHandle<A>> + '_ {
        self.storage.handles::<A>()
    }

    /// Returns an iterator over all loaded assets of a specific type.
    pub fn iter<A: Asset>(&self) -> impl Iterator<Item = (AssetHandle<A>, &A)> {
        self.storage.iter::<A>()
    }

    /// Creates a hot reload watcher for this asset server.
    ///
    /// The watcher will detect file changes in the asset root directory
    /// and automatically reload modified assets.
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be initialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::AssetServer;
    ///
    /// let mut server = AssetServer::new();
    /// let mut watcher = server.create_hot_reload_watcher().unwrap();
    ///
    /// // In game loop
    /// loop {
    ///     watcher.process_events(&mut server);
    ///     // ... rest of game loop
    /// }
    /// ```
    #[cfg(feature = "native")]
    pub fn create_hot_reload_watcher(&self) -> notify::Result<HotReloadWatcher> {
        HotReloadWatcher::new(self)
    }

    /// Creates a hot reload watcher with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be initialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::{AssetServer, HotReloadConfig};
    /// use std::time::Duration;
    ///
    /// let mut server = AssetServer::new();
    /// let config = HotReloadConfig::new()
    ///     .with_debounce(Duration::from_millis(200))
    ///     .watch_extension("png")
    ///     .watch_extension("json");
    ///
    /// let mut watcher = server.create_hot_reload_watcher_with_config(config).unwrap();
    ///
    /// // In game loop
    /// loop {
    ///     watcher.process_events(&mut server);
    ///     // ... rest of game loop
    /// }
    /// ```
    #[cfg(feature = "native")]
    pub fn create_hot_reload_watcher_with_config(
        &self,
        config: HotReloadConfig,
    ) -> notify::Result<HotReloadWatcher> {
        HotReloadWatcher::with_config(self, config)
    }
}
