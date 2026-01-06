//! Asset server for managing asset loading and caching.
//!
//! The `AssetServer` is the central coordinator for all asset operations:
//! - Loading assets from disk (sync and async)
//! - Caching loaded assets
//! - Tracking asset loading states
//! - Managing asset loaders
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                      AssetServer                            │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
//! │  │   Loaders    │  │   Storage    │  │  IO Thread   │    │
//! │  │  Registry    │  │   (Cache)    │  │    Pool      │    │
//! │  └──────────────┘  └──────────────┘  └──────────────┘    │
//! └────────────────────────────────────────────────────────────┘
//!         │                    │                    │
//!         ▼                    ▼                    ▼
//!    Load Asset          Get Cached           Async Load
//! ```
//!
//! # Example
//!
//! ```
//! use goud_engine::assets::{Asset, AssetServer, AssetPath};
//!
//! struct Texture { width: u32, height: u32 }
//! impl Asset for Texture {}
//!
//! // Create asset server
//! let mut server = AssetServer::new();
//!
//! // Load an asset (returns handle immediately, loads in background)
//! let handle = server.load::<Texture>("textures/player.png");
//!
//! // Check if loaded
//! if server.is_loaded(&handle) {
//!     if let Some(texture) = server.get(&handle) {
//!         println!("Texture: {}x{}", texture.width, texture.height);
//!     }
//! }
//! ```

use crate::assets::{
    Asset, AssetHandle, AssetId, AssetLoadError, AssetLoader, AssetPath, AssetState,
    AssetStorage, ErasedAssetLoader, HotReloadConfig, HotReloadWatcher, LoadContext,
    TypedAssetLoader,
};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

// =============================================================================
// AssetServer
// =============================================================================

/// Central coordinator for asset loading and caching.
///
/// The `AssetServer` manages:
/// - Asset loaders (registered by file extension)
/// - Asset storage (cached loaded assets)
/// - Loading queue (assets being loaded)
/// - Hot reloading (watching for file changes)
///
/// # Thread Safety
///
/// `AssetServer` is `Send` but NOT `Sync` - it should be accessed from a single
/// thread (typically the main thread). For multi-threaded asset loading, use
/// async handles and check loading state from the main thread.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetServer};
///
/// struct MyAsset { data: String }
/// impl Asset for MyAsset {}
///
/// let mut server = AssetServer::new();
///
/// // Load returns a handle immediately
/// let handle = server.load::<MyAsset>("data/config.json");
///
/// // Asset loads in background, check state
/// match server.get_load_state(&handle) {
///     Some(state) => println!("Loading: {}", state),
///     None => println!("Not found"),
/// }
/// ```
pub struct AssetServer {
    /// Base directory for asset files (e.g., "assets/").
    asset_root: PathBuf,

    /// Asset storage (cache).
    storage: AssetStorage,

    /// Registered asset loaders by extension.
    loaders: HashMap<String, Box<dyn ErasedAssetLoader>>,

    /// Loader registry by AssetId (for lookup without extension).
    loader_by_type: HashMap<AssetId, Box<dyn ErasedAssetLoader>>,
}

impl AssetServer {
    /// Creates a new asset server with the default asset root ("assets/").
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetServer;
    ///
    /// let server = AssetServer::new();
    /// ```
    pub fn new() -> Self {
        Self::with_root("assets")
    }

    /// Creates a new asset server with a custom asset root directory.
    ///
    /// # Arguments
    ///
    /// * `root` - Base directory for asset files (relative or absolute)
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetServer;
    ///
    /// let server = AssetServer::with_root("game_assets");
    /// ```
    pub fn with_root(root: impl AsRef<Path>) -> Self {
        Self {
            asset_root: root.as_ref().to_path_buf(),
            storage: AssetStorage::new(),
            loaders: HashMap::new(),
            loader_by_type: HashMap::new(),
        }
    }

    /// Returns the asset root directory.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetServer;
    ///
    /// let server = AssetServer::with_root("game_assets");
    /// assert_eq!(server.asset_root().to_str().unwrap(), "game_assets");
    /// ```
    #[inline]
    pub fn asset_root(&self) -> &Path {
        &self.asset_root
    }

    /// Sets the asset root directory.
    ///
    /// # Arguments
    ///
    /// * `root` - New base directory for asset files
    pub fn set_asset_root(&mut self, root: impl AsRef<Path>) {
        self.asset_root = root.as_ref().to_path_buf();
    }

    /// Registers an asset loader for specific file extensions.
    ///
    /// Loaders are matched by file extension. If multiple loaders support
    /// the same extension, the most recently registered one is used.
    ///
    /// # Arguments
    ///
    /// * `loader` - The asset loader to register
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetServer, AssetLoader, LoadContext, AssetLoadError};
    ///
    /// struct TextAsset { content: String }
    /// impl Asset for TextAsset {}
    ///
    /// struct TextLoader;
    /// impl AssetLoader for TextLoader {
    ///     type Asset = TextAsset;
    ///     type Settings = ();
    ///
    ///     fn extensions(&self) -> &[&str] {
    ///         &["txt"]
    ///     }
    ///
    ///     fn load<'a>(
    ///         &'a self,
    ///         bytes: &'a [u8],
    ///         _settings: &'a Self::Settings,
    ///         _context: &'a mut LoadContext,
    ///     ) -> Result<Self::Asset, AssetLoadError> {
    ///         let content = String::from_utf8(bytes.to_vec())
    ///             .map_err(|e| AssetLoadError::decode_failed(e.to_string()))?;
    ///         Ok(TextAsset { content })
    ///     }
    /// }
    ///
    /// let mut server = AssetServer::new();
    /// server.register_loader(TextLoader);
    /// ```
    pub fn register_loader<L: AssetLoader>(&mut self, loader: L) {
        let typed = TypedAssetLoader::new(loader);
        let asset_id = AssetId::of::<L::Asset>();

        // Register by extensions
        for extension in typed.extensions() {
            let ext = extension.to_lowercase();
            self.loaders.insert(ext, Box::new(typed.clone()));
        }

        // Register by asset type
        self.loader_by_type.insert(asset_id, Box::new(typed));
    }

    /// Registers an asset loader with custom settings.
    ///
    /// # Arguments
    ///
    /// * `loader` - The asset loader to register
    /// * `settings` - Custom settings for this loader
    pub fn register_loader_with_settings<L: AssetLoader>(
        &mut self,
        loader: L,
        settings: L::Settings,
    ) {
        let typed = TypedAssetLoader::with_settings(loader, settings);
        let asset_id = AssetId::of::<L::Asset>();

        for extension in typed.extensions() {
            let ext = extension.to_lowercase();
            self.loaders.insert(ext, Box::new(typed.clone()));
        }

        self.loader_by_type.insert(asset_id, Box::new(typed));
    }

    /// Returns true if a loader is registered for the given extension.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetServer;
    ///
    /// let server = AssetServer::new();
    /// // Assuming TextLoader is registered for "txt"
    /// // assert!(server.has_loader_for_extension("txt"));
    /// assert!(!server.has_loader_for_extension("unknown"));
    /// ```
    pub fn has_loader_for_extension(&self, extension: &str) -> bool {
        self.loaders.contains_key(&extension.to_lowercase())
    }

    /// Returns true if a loader is registered for the given asset type.
    pub fn has_loader_for_type<A: Asset>(&self) -> bool {
        self.loader_by_type.contains_key(&AssetId::of::<A>())
    }

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

    /// Internal method to load an asset synchronously.
    fn load_asset_sync<A: Asset>(&mut self, path: &AssetPath) -> Result<A, AssetLoadError> {
        // Get file extension
        let extension = path
            .extension()
            .ok_or_else(|| AssetLoadError::unsupported_format(""))?;

        // Find loader
        let loader = self
            .loaders
            .get(extension)
            .ok_or_else(|| AssetLoadError::unsupported_format(extension))?;

        // Build full path
        let full_path = self.asset_root.join(path.as_str());

        // Read file
        let bytes = std::fs::read(&full_path)
            .map_err(|e| AssetLoadError::io_error(&full_path, e))?;

        // Create load context
        let mut context = LoadContext::new(path.clone().into_owned());

        // Load asset (type-erased)
        let boxed = loader.load_erased(&bytes, &mut context)?;

        // Downcast to concrete type
        boxed
            .downcast::<A>()
            .map(|boxed| *boxed)
            .map_err(|_| AssetLoadError::custom("Type mismatch after loading"))
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

    /// Returns the number of registered loaders.
    #[inline]
    pub fn loader_count(&self) -> usize {
        self.loader_by_type.len()
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
    pub fn create_hot_reload_watcher_with_config(
        &self,
        config: HotReloadConfig,
    ) -> notify::Result<HotReloadWatcher> {
        HotReloadWatcher::with_config(self, config)
    }
}

impl Default for AssetServer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for AssetServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetServer")
            .field("asset_root", &self.asset_root)
            .field("total_assets", &self.total_loaded_count())
            .field("registered_types", &self.registered_type_count())
            .field("loaders", &self.loader_count())
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
    #[derive(Debug, Clone, PartialEq)]
    struct TestAsset {
        content: String,
    }

    impl Asset for TestAsset {
        fn asset_type_name() -> &'static str {
            "TestAsset"
        }
    }

    #[derive(Debug, Clone, PartialEq)]
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

    // Test loaders
    #[derive(Clone)]
    struct TestAssetLoader;

    impl AssetLoader for TestAssetLoader {
        type Asset = TestAsset;
        type Settings = ();

        fn extensions(&self) -> &[&str] {
            &["test"]
        }

        fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            _settings: &'a Self::Settings,
            _context: &'a mut LoadContext,
        ) -> Result<Self::Asset, AssetLoadError> {
            let content = String::from_utf8(bytes.to_vec())
                .map_err(|e| AssetLoadError::decode_failed(e.to_string()))?;
            Ok(TestAsset { content })
        }
    }

    #[derive(Clone)]
    struct TestTextureLoader;

    impl AssetLoader for TestTextureLoader {
        type Asset = TestTexture;
        type Settings = ();

        fn extensions(&self) -> &[&str] {
            &["png", "jpg"]
        }

        fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            _settings: &'a Self::Settings,
            _context: &'a mut LoadContext,
        ) -> Result<Self::Asset, AssetLoadError> {
            // Simple fake loader that just reads size from first 8 bytes
            if bytes.len() < 8 {
                return Err(AssetLoadError::decode_failed("Not enough data"));
            }

            let width = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            let height = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

            Ok(TestTexture { width, height })
        }
    }

    // =============================================================================
    // AssetServer Tests
    // =============================================================================

    mod asset_server {
        use super::*;

        #[test]
        fn test_new() {
            let server = AssetServer::new();
            assert_eq!(server.asset_root(), Path::new("assets"));
            assert_eq!(server.total_loaded_count(), 0);
            assert_eq!(server.loader_count(), 0);
        }

        #[test]
        fn test_with_root() {
            let server = AssetServer::with_root("custom_assets");
            assert_eq!(server.asset_root(), Path::new("custom_assets"));
        }

        #[test]
        fn test_set_asset_root() {
            let mut server = AssetServer::new();
            server.set_asset_root("game_assets");
            assert_eq!(server.asset_root(), Path::new("game_assets"));
        }

        #[test]
        fn test_default() {
            let server = AssetServer::default();
            assert_eq!(server.asset_root(), Path::new("assets"));
        }

        #[test]
        fn test_debug() {
            let server = AssetServer::new();
            let debug_str = format!("{:?}", server);
            assert!(debug_str.contains("AssetServer"));
            assert!(debug_str.contains("assets"));
        }
    }

    mod loader_registration {
        use super::*;

        #[test]
        fn test_register_loader() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);

            assert_eq!(server.loader_count(), 1);
            assert!(server.has_loader_for_extension("test"));
            assert!(server.has_loader_for_type::<TestAsset>());
        }

        #[test]
        fn test_register_multiple_loaders() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);
            server.register_loader(TestTextureLoader);

            assert_eq!(server.loader_count(), 2);
            assert!(server.has_loader_for_extension("test"));
            assert!(server.has_loader_for_extension("png"));
            assert!(server.has_loader_for_extension("jpg"));
        }

        #[test]
        fn test_has_loader_for_extension() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);

            assert!(server.has_loader_for_extension("test"));
            assert!(server.has_loader_for_extension("TEST")); // Case-insensitive
            assert!(!server.has_loader_for_extension("unknown"));
        }

        #[test]
        fn test_has_loader_for_type() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);

            assert!(server.has_loader_for_type::<TestAsset>());
            assert!(!server.has_loader_for_type::<TestTexture>());
        }

        #[test]
        fn test_register_loader_with_settings() {
            let mut server = AssetServer::new();
            server.register_loader_with_settings(TestAssetLoader, ());

            assert!(server.has_loader_for_type::<TestAsset>());
        }
    }

    mod asset_operations {
        use super::*;
        use std::fs;
        use std::io::Write;
        use tempfile::TempDir;

        #[test]
        fn test_loaded_count() {
            let server = AssetServer::new();
            assert_eq!(server.loaded_count::<TestAsset>(), 0);
            assert_eq!(server.total_loaded_count(), 0);
        }

        #[test]
        fn test_registered_type_count() {
            let server = AssetServer::new();
            assert_eq!(server.registered_type_count(), 0);
        }

        #[test]
        fn test_handles_iterator() {
            let server = AssetServer::new();
            let handles: Vec<_> = server.handles::<TestAsset>().collect();
            assert_eq!(handles.len(), 0);
        }

        #[test]
        fn test_iter() {
            let server = AssetServer::new();
            let assets: Vec<_> = server.iter::<TestAsset>().collect();
            assert_eq!(assets.len(), 0);
        }

        #[test]
        fn test_clear_type() {
            let mut server = AssetServer::new();
            server.clear_type::<TestAsset>();
            // Should not panic
        }

        #[test]
        fn test_clear() {
            let mut server = AssetServer::new();
            server.clear();
            assert_eq!(server.total_loaded_count(), 0);
        }

        #[test]
        fn test_load_and_get() {
            // Create temp directory with test file
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();

            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create test file
            let test_path = asset_root.join("test.test");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"Hello, World!").unwrap();

            // Load asset
            let handle = server.load::<TestAsset>("test.test");

            // Should be loaded
            assert!(server.is_loaded(&handle));
            assert_eq!(server.loaded_count::<TestAsset>(), 1);

            // Get asset
            let asset = server.get(&handle);
            assert!(asset.is_some());
            assert_eq!(asset.unwrap().content, "Hello, World!");
        }

        #[test]
        fn test_load_nonexistent_file() {
            let temp_dir = TempDir::new().unwrap();
            let mut server = AssetServer::with_root(temp_dir.path());
            server.register_loader(TestAssetLoader);

            let handle = server.load::<TestAsset>("nonexistent.test");

            // Handle is valid but asset is not loaded
            assert!(handle.is_valid());
            assert!(!server.is_loaded(&handle));

            // Check state is Failed
            let state = server.get_load_state(&handle);
            assert!(state.is_some());
            assert!(state.unwrap().is_failed());
        }

        #[test]
        fn test_load_unsupported_extension() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create file with unsupported extension
            let test_path = asset_root.join("test.unknown");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"data").unwrap();

            let handle = server.load::<TestAsset>("test.unknown");

            assert!(!server.is_loaded(&handle));
            let state = server.get_load_state(&handle);
            assert!(state.is_some());
            assert!(state.unwrap().is_failed());
        }

        #[test]
        fn test_load_deduplication() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create test file
            let test_path = asset_root.join("test.test");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"content").unwrap();

            // Load same asset twice
            let handle1 = server.load::<TestAsset>("test.test");
            let handle2 = server.load::<TestAsset>("test.test");

            // Should return same handle
            assert_eq!(handle1, handle2);
            assert_eq!(server.loaded_count::<TestAsset>(), 1);
        }

        #[test]
        fn test_unload() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create and load test file
            let test_path = asset_root.join("test.test");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"data").unwrap();

            let handle = server.load::<TestAsset>("test.test");
            assert!(server.is_loaded(&handle));

            // Unload
            let asset = server.unload(&handle);
            assert!(asset.is_some());
            assert!(!server.is_loaded(&handle));
            assert_eq!(server.loaded_count::<TestAsset>(), 0);
        }

        #[test]
        fn test_get_mut() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create and load test file
            let test_path = asset_root.join("test.test");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"original").unwrap();

            let handle = server.load::<TestAsset>("test.test");

            // Modify through get_mut
            if let Some(asset) = server.get_mut(&handle) {
                asset.content = "modified".to_string();
            }

            // Check modification
            assert_eq!(server.get(&handle).unwrap().content, "modified");
        }

        #[test]
        fn test_multiple_asset_types() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);
            server.register_loader(TestTextureLoader);

            // Create test files
            let test1 = asset_root.join("test.test");
            fs::File::create(&test1)
                .unwrap()
                .write_all(b"text")
                .unwrap();

            let test2 = asset_root.join("image.png");
            fs::File::create(&test2)
                .unwrap()
                .write_all(&[100, 0, 0, 0, 50, 0, 0, 0])
                .unwrap();

            // Load both types
            let handle1 = server.load::<TestAsset>("test.test");
            let handle2 = server.load::<TestTexture>("image.png");

            assert!(server.is_loaded(&handle1));
            assert!(server.is_loaded(&handle2));
            assert_eq!(server.loaded_count::<TestAsset>(), 1);
            assert_eq!(server.loaded_count::<TestTexture>(), 1);
            assert_eq!(server.total_loaded_count(), 2);
        }
    }

    mod thread_safety {
        use super::*;

        #[test]
        fn test_asset_server_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetServer>();
        }

        // Note: AssetServer is intentionally NOT Sync
        // It should be accessed from a single thread
    }
}
