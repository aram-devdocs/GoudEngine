//! Asset loading and access operations for `AssetServer`.

use super::core::AssetServer;
use crate::assets::{Asset, AssetHandle, AssetLoadError, AssetPath, AssetState, LoadContext};
#[cfg(feature = "native")]
use crate::assets::{HotReloadConfig, HotReloadWatcher};
use std::path::Path;

impl AssetServer {
    /// Loads an asset from a path, returning a handle immediately.
    ///
    /// The asset loads synchronously (blocking). Use `load_async` for
    /// non-blocking loads in a real implementation.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the asset file (relative to asset root)
    ///
    /// # Returns
    ///
    /// A handle to the asset. The handle is valid even if loading fails.
    /// Check the asset state with `get_load_state()`.
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
    /// let handle = server.load::<Texture>("textures/player.png");
    ///
    /// // Check if loaded
    /// if server.is_loaded(&handle) {
    ///     println!("Texture loaded!");
    /// }
    /// ```
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
            Ok(asset) => {
                self.storage.set_loaded(&handle, asset);
            }
            Err(error) => {
                // Mark as failed
                if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
                    entry.set_failed(error.to_string());
                }
            }
        }

        handle
    }

    /// Runs raw bytes through the registered loader for the given asset path's extension.
    pub(super) fn parse_bytes<A: Asset>(
        &self,
        path: &AssetPath,
        bytes: &[u8],
    ) -> Result<A, AssetLoadError> {
        let extension = path
            .extension()
            .ok_or_else(|| AssetLoadError::unsupported_format(""))?;

        let loader = self
            .loaders
            .get(extension)
            .ok_or_else(|| AssetLoadError::unsupported_format(extension))?;

        let mut context = LoadContext::new(path.clone().into_owned());
        let boxed = loader.load_erased(bytes, &mut context)?;

        boxed
            .downcast::<A>()
            .map(|boxed| *boxed)
            .map_err(|_| AssetLoadError::custom("Type mismatch after loading"))
    }

    /// Reads a file from disk and parses it into an asset.
    fn load_asset_sync<A: Asset>(&self, path: &AssetPath) -> Result<A, AssetLoadError> {
        let full_path = self.asset_root.join(path.as_str());
        let bytes =
            std::fs::read(&full_path).map_err(|e| AssetLoadError::io_error(&full_path, e))?;
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
            Ok(asset) => {
                self.storage.set_loaded(&handle, asset);
            }
            Err(error) => {
                if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
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

    /// Returns the loading state for a handle.
    ///
    /// Returns `None` if the handle is invalid or not tracked.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetServer, AssetState};
    ///
    /// struct MyAsset;
    /// impl Asset for MyAsset {}
    ///
    /// let server = AssetServer::new();
    /// // let handle = server.load::<MyAsset>("data.json");
    /// // match server.get_load_state(&handle) {
    /// //     Some(AssetState::Loaded) => println!("Ready!"),
    /// //     Some(AssetState::Loading { progress }) => println!("Loading: {}%", progress * 100.0),
    /// //     Some(AssetState::Failed { error }) => println!("Error: {}", error),
    /// //     _ => println!("Unknown state"),
    /// // }
    /// ```
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
        self.storage.remove(handle)
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
