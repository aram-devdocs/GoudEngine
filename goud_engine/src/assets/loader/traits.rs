//! Asset loader traits and type-erased loader wrapper.

use crate::assets::Asset;

use super::{AssetLoadError, LoadContext};

/// Trait for types that can load assets from raw bytes.
///
/// Asset loaders are registered with the AssetServer and invoked when
/// an asset of the corresponding type is requested.
///
/// # Type Parameters
///
/// - `Asset`: The type of asset this loader produces
/// - `Settings`: Configuration type for the loader (use `()` if not needed)
///
/// # Example
///
/// ```ignore
/// use goud_engine::assets::{Asset, AssetLoader, LoadContext, AssetLoadError, TextAsset, TextAssetLoader};
///
/// let loader = TextAssetLoader;
/// let bytes = b"Hello, World!";
/// let path = goud_engine::assets::AssetPath::from_string("test.txt".to_string());
/// let mut context = LoadContext::new(path);
///
/// let result = loader.load(bytes, &(), &mut context);
/// assert!(result.is_ok());
/// let asset = result.unwrap();
/// assert_eq!(asset.content, "Hello, World!");
/// ```
pub trait AssetLoader: Send + Sync + Clone + 'static {
    /// The type of asset this loader produces.
    type Asset: Asset;

    /// The settings type for this loader.
    ///
    /// Use `()` if no settings are needed.
    type Settings: Send + Sync + Clone + Default + 'static;

    /// Returns the file extensions supported by this loader.
    ///
    /// Extensions should not include the leading dot (e.g., "png", not ".png").
    fn extensions(&self) -> &[&str];

    /// Loads an asset from raw bytes.
    ///
    /// # Arguments
    ///
    /// - `bytes`: The raw bytes of the asset file
    /// - `settings`: Loader-specific settings
    /// - `context`: Loading context with asset path and dependency loading
    ///
    /// # Returns
    ///
    /// The loaded asset, or an error if loading failed.
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError>;

    /// Returns whether this loader can load assets with the given extension.
    ///
    /// Default implementation checks against `extensions()`.
    fn supports_extension(&self, extension: &str) -> bool {
        self.extensions()
            .iter()
            .any(|&ext| ext.eq_ignore_ascii_case(extension))
    }
}

/// Type-erased asset loader for dynamic dispatch.
///
/// This trait allows storing different loader types in a collection.
pub trait ErasedAssetLoader: Send + Sync + 'static {
    /// Returns the file extensions supported by this loader.
    fn extensions(&self) -> &[&str];

    /// Loads an asset from raw bytes, returning a boxed Any.
    ///
    /// The caller is responsible for downcasting to the correct asset type.
    fn load_erased<'a>(
        &'a self,
        bytes: &'a [u8],
        context: &'a mut LoadContext,
    ) -> Result<Box<dyn std::any::Any + Send>, AssetLoadError>;

    /// Returns whether this loader can load assets with the given extension.
    fn supports_extension(&self, extension: &str) -> bool;
}

/// Wrapper that implements ErasedAssetLoader for any AssetLoader.
#[derive(Clone)]
pub struct TypedAssetLoader<L: AssetLoader> {
    loader: L,
    settings: L::Settings,
}

impl<L: AssetLoader> TypedAssetLoader<L> {
    /// Creates a new typed asset loader with default settings.
    pub fn new(loader: L) -> Self {
        Self {
            loader,
            settings: L::Settings::default(),
        }
    }

    /// Creates a new typed asset loader with custom settings.
    pub fn with_settings(loader: L, settings: L::Settings) -> Self {
        Self { loader, settings }
    }

    /// Returns a reference to the inner loader.
    pub fn loader(&self) -> &L {
        &self.loader
    }

    /// Returns a reference to the settings.
    pub fn settings(&self) -> &L::Settings {
        &self.settings
    }

    /// Returns a mutable reference to the settings.
    pub fn settings_mut(&mut self) -> &mut L::Settings {
        &mut self.settings
    }
}

impl<L: AssetLoader> ErasedAssetLoader for TypedAssetLoader<L> {
    fn extensions(&self) -> &[&str] {
        self.loader.extensions()
    }

    fn load_erased<'a>(
        &'a self,
        bytes: &'a [u8],
        context: &'a mut LoadContext,
    ) -> Result<Box<dyn std::any::Any + Send>, AssetLoadError> {
        let asset = self.loader.load(bytes, &self.settings, context)?;
        Ok(Box::new(asset))
    }

    fn supports_extension(&self, extension: &str) -> bool {
        self.loader.supports_extension(extension)
    }
}
