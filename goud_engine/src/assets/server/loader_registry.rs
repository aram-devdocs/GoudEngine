//! Loader registration methods for `AssetServer`.

use super::core::AssetServer;
use crate::assets::{Asset, AssetId, AssetLoader, ErasedAssetLoader, TypedAssetLoader};

impl AssetServer {
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
    /// ```no_run
    /// use goud_engine::assets::{Asset, AssetServer, AssetLoader, LoadContext, AssetLoadError};
    ///
    /// #[derive(Clone)]
    /// struct TextAsset { content: String }
    /// impl Asset for TextAsset {}
    ///
    /// #[derive(Clone)]
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

    /// Returns the number of registered loaders.
    #[inline]
    pub fn loader_count(&self) -> usize {
        self.loader_by_type.len()
    }
}
