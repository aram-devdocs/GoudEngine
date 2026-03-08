//! Core `AssetServer` type definition and construction helpers.

use crate::assets::dependency::DependencyGraph;
#[cfg(feature = "native")]
use crate::assets::AssetLoadError;
use crate::assets::vfs::{OsFs, VirtualFs};
use crate::assets::{AssetId, AssetStorage, ErasedAssetLoader};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

// =============================================================================
// LoadResult (native-only)
// =============================================================================

/// Result of an async asset load, sent from a background thread to the main thread.
#[cfg(feature = "native")]
pub(super) struct LoadResult {
    /// Index component of the asset handle.
    pub(super) handle_index: u32,
    /// Generation component of the asset handle.
    pub(super) handle_generation: u32,
    /// Type identifier for the asset.
    pub(super) asset_id: AssetId,
    /// The loaded asset data or an error.
    pub(super) result: Result<Box<dyn std::any::Any + Send>, AssetLoadError>,
}

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
    pub(super) asset_root: PathBuf,

    /// Virtual filesystem for reading asset bytes.
    pub(super) vfs: Box<dyn VirtualFs>,

    /// Asset storage (cache).
    pub(super) storage: AssetStorage,

    /// Registered asset loaders by extension.
    pub(super) loaders: HashMap<String, Box<dyn ErasedAssetLoader>>,

    /// Loader registry by AssetId (for lookup without extension).
    pub(super) loader_by_type: HashMap<AssetId, Box<dyn ErasedAssetLoader>>,

    /// Dependency graph for cascade reloading.
    pub(super) dependency_graph: DependencyGraph,

    /// Sender for background load results (used by native load_async).
    #[cfg(all(feature = "native", not(feature = "web")))]
    pub(super) load_sender: std::sync::mpsc::Sender<LoadResult>,

    /// Receiver for background load results (native-only).
    #[cfg(feature = "native")]
    pub(super) load_receiver: std::sync::mpsc::Receiver<LoadResult>,

    /// Queue of assets whose ref count reached zero, awaiting deferred removal.
    /// Each entry is `(AssetId, index, generation)`.
    pub(super) pending_unloads: Vec<(AssetId, u32, u32)>,
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
        #[cfg(all(feature = "native", not(feature = "web")))]
        let (load_sender, load_receiver) = std::sync::mpsc::channel();
        #[cfg(all(feature = "native", feature = "web"))]
        let (_load_sender, load_receiver) = std::sync::mpsc::channel::<LoadResult>();

        let root_path = root.as_ref().to_path_buf();
        Self {
            vfs: Box::new(OsFs::new(root_path.clone())),
            asset_root: root_path,
            storage: AssetStorage::new(),
            loaders: HashMap::new(),
            loader_by_type: HashMap::new(),
            dependency_graph: DependencyGraph::new(),
            #[cfg(all(feature = "native", not(feature = "web")))]
            load_sender,
            #[cfg(feature = "native")]
            load_receiver,
            pending_unloads: Vec::new(),
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
    /// Also updates the default VFS to use the new root. If a custom VFS
    /// was set via [`set_vfs`](Self::set_vfs), it will be replaced with
    /// an [`OsFs`] rooted at the new path.
    ///
    /// # Arguments
    ///
    /// * `root` - New base directory for asset files
    pub fn set_asset_root(&mut self, root: impl AsRef<Path>) {
        let root_path = root.as_ref().to_path_buf();
        self.vfs = Box::new(OsFs::new(root_path.clone()));
        self.asset_root = root_path;
    }

    /// Replaces the virtual filesystem used for asset I/O.
    ///
    /// This allows the asset server to read from archives, embedded
    /// resources, or custom I/O layers instead of the OS filesystem.
    ///
    /// # Arguments
    ///
    /// * `vfs` - The new virtual filesystem implementation
    pub fn set_vfs(&mut self, vfs: Box<dyn VirtualFs>) {
        self.vfs = vfs;
    }

    /// Returns a reference to the current virtual filesystem.
    pub fn vfs(&self) -> &dyn VirtualFs {
        self.vfs.as_ref()
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
            .field("total_assets", &self.storage.total_len())
            .field("registered_types", &self.storage.type_count())
            .field("loaders", &self.loader_by_type.len())
            .finish()
    }
}
