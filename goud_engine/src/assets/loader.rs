//! Asset loading infrastructure.
//!
//! This module provides traits and types for implementing custom asset loaders.
//! Asset loaders are responsible for converting raw data (e.g., file bytes) into
//! typed asset instances.
//!
//! # Architecture
//!
//! The loading system is designed for asynchronous operation:
//!
//! ```text
//! ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
//! │  Raw Bytes   │────▶│    Loader    │────▶│    Asset     │
//! │  (from I/O)  │     │  (Parser)    │     │  (Typed)     │
//! └──────────────┘     └──────────────┘     └──────────────┘
//! ```
//!
///
/// ```
/// use goud_engine::assets::{Asset, AssetLoader, LoadContext, AssetLoadError};
///
/// #[derive(Clone)]
/// struct MyAsset {
///     data: String,
/// }
///
/// impl Asset for MyAsset {}
///
/// #[derive(Clone)]
/// struct MyAssetLoader;
///
/// impl AssetLoader for MyAssetLoader {
///     type Asset = MyAsset;
///     type Settings = ();
///
///     fn extensions(&self) -> &[&str] {
///         &["myasset"]
///     }
///
///     fn load<'a>(
///         &'a self,
///         bytes: &'a [u8],
///         _settings: &'a Self::Settings,
///         _context: &'a mut LoadContext,
///     ) -> Result<Self::Asset, AssetLoadError> {
///         let data = String::from_utf8(bytes.to_vec())
///             .map_err(|e| AssetLoadError::DecodeFailed(e.to_string()))?;
///         Ok(MyAsset { data })
///     }
/// }
/// ```
use crate::assets::{Asset, AssetPath};
use std::error::Error;
use std::fmt;
use std::path::Path;

/// Errors that can occur during asset loading.
#[derive(Debug, Clone)]
pub enum AssetLoadError {
    /// The asset file was not found.
    NotFound {
        /// The path to the asset that was not found.
        path: String,
    },

    /// Failed to read the asset file.
    IoError {
        /// The path to the asset that failed to load.
        path: String,
        /// The I/O error message.
        message: String,
    },

    /// Failed to decode/parse the asset data.
    DecodeFailed(
        /// The decoding error message.
        String,
    ),

    /// The asset format is not supported.
    UnsupportedFormat {
        /// The unsupported file extension.
        extension: String,
    },

    /// A dependency asset failed to load.
    DependencyFailed {
        /// The path of the asset with the failed dependency.
        asset_path: String,
        /// The path of the dependency that failed to load.
        dependency_path: String,
        /// The error message from the dependency failure.
        message: String,
    },

    /// A custom loader-specific error occurred.
    Custom(
        /// The custom error message.
        String,
    ),
}

impl AssetLoadError {
    /// Creates a NotFound error from a path.
    pub fn not_found(path: impl AsRef<Path>) -> Self {
        Self::NotFound {
            path: path.as_ref().display().to_string(),
        }
    }

    /// Creates an IoError from a path and error.
    pub fn io_error(path: impl AsRef<Path>, error: impl Error) -> Self {
        Self::IoError {
            path: path.as_ref().display().to_string(),
            message: error.to_string(),
        }
    }

    /// Creates a DecodeFailed error from a message.
    pub fn decode_failed(message: impl Into<String>) -> Self {
        Self::DecodeFailed(message.into())
    }

    /// Creates an UnsupportedFormat error from an extension.
    pub fn unsupported_format(extension: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            extension: extension.into(),
        }
    }

    /// Creates a DependencyFailed error.
    pub fn dependency_failed(
        asset_path: impl Into<String>,
        dependency_path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::DependencyFailed {
            asset_path: asset_path.into(),
            dependency_path: dependency_path.into(),
            message: message.into(),
        }
    }

    /// Creates a Custom error from a message.
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    /// Returns true if this is a NotFound error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Returns true if this is an IoError.
    pub fn is_io_error(&self) -> bool {
        matches!(self, Self::IoError { .. })
    }

    /// Returns true if this is a DecodeFailed error.
    pub fn is_decode_failed(&self) -> bool {
        matches!(self, Self::DecodeFailed(_))
    }

    /// Returns true if this is an UnsupportedFormat error.
    pub fn is_unsupported_format(&self) -> bool {
        matches!(self, Self::UnsupportedFormat { .. })
    }

    /// Returns true if this is a DependencyFailed error.
    pub fn is_dependency_failed(&self) -> bool {
        matches!(self, Self::DependencyFailed { .. })
    }

    /// Returns true if this is a Custom error.
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }
}

impl fmt::Display for AssetLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { path } => write!(f, "Asset not found: {path}"),
            Self::IoError { path, message } => {
                write!(f, "I/O error loading asset '{path}': {message}")
            }
            Self::DecodeFailed(msg) => write!(f, "Failed to decode asset: {msg}"),
            Self::UnsupportedFormat { extension } => {
                write!(f, "Unsupported asset format: '.{extension}'")
            }
            Self::DependencyFailed {
                asset_path,
                dependency_path,
                message,
            } => write!(
                f,
                "Dependency '{dependency_path}' of asset '{asset_path}' failed to load: {message}"
            ),
            Self::Custom(msg) => write!(f, "Asset loading error: {msg}"),
        }
    }
}

impl Error for AssetLoadError {}

/// Context provided to asset loaders during loading.
///
/// The context provides:
/// - The path of the asset being loaded
/// - Methods to load dependencies
/// - Metadata about the loading process
pub struct LoadContext<'a> {
    /// The path of the asset being loaded (owned).
    asset_path: AssetPath<'static>,
    /// Marker for lifetime (in the future, this will hold references to AssetServer).
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> LoadContext<'a> {
    /// Creates a new load context for the given asset path.
    pub fn new(asset_path: AssetPath<'static>) -> Self {
        Self {
            asset_path,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the path of the asset being loaded.
    pub fn path(&self) -> &AssetPath<'static> {
        &self.asset_path
    }

    /// Returns the asset path as a string.
    pub fn path_str(&self) -> &str {
        self.asset_path.as_str()
    }

    /// Returns the file extension of the asset.
    pub fn extension(&self) -> Option<&str> {
        self.asset_path.extension()
    }

    /// Returns the file name of the asset.
    pub fn file_name(&self) -> Option<&str> {
        self.asset_path.file_name()
    }
}

impl<'a> fmt::Debug for LoadContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadContext")
            .field("asset_path", &self.asset_path)
            .finish()
    }
}

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

// Test asset types - used in doctests and unit tests
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TextAsset {
    pub content: String,
}

impl Asset for TextAsset {}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryAsset {
    pub data: Vec<u8>,
}

impl Asset for BinaryAsset {}

// Test loaders - used in doctests and unit tests
#[allow(dead_code)]
#[derive(Clone)]
pub struct TextAssetLoader;

impl AssetLoader for TextAssetLoader {
    type Asset = TextAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["txt", "text"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        _context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| AssetLoadError::decode_failed(e.to_string()))?;
        Ok(TextAsset { content })
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct BinaryAssetLoader;

impl AssetLoader for BinaryAssetLoader {
    type Asset = BinaryAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["bin", "dat"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        _context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        Ok(BinaryAsset {
            data: bytes.to_vec(),
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LoaderSettings {
    pub max_size: usize,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct SettingsLoader;

impl AssetLoader for SettingsLoader {
    type Asset = BinaryAsset;
    type Settings = LoaderSettings;

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        settings: &'a Self::Settings,
        _context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        if bytes.len() > settings.max_size {
            return Err(AssetLoadError::custom(format!(
                "Asset too large: {} > {}",
                bytes.len(),
                settings.max_size
            )));
        }
        Ok(BinaryAsset {
            data: bytes.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AssetLoadError tests
    mod asset_load_error {
        use super::*;

        #[test]
        fn test_not_found() {
            let error = AssetLoadError::not_found("test.png");
            assert!(error.is_not_found());
            assert!(!error.is_io_error());
            assert!(error.to_string().contains("test.png"));
        }

        #[test]
        fn test_io_error() {
            let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
            let error = AssetLoadError::io_error("test.png", io_err);
            assert!(error.is_io_error());
            assert!(!error.is_not_found());
            assert!(error.to_string().contains("test.png"));
            assert!(error.to_string().contains("access denied"));
        }

        #[test]
        fn test_decode_failed() {
            let error = AssetLoadError::decode_failed("Invalid UTF-8");
            assert!(error.is_decode_failed());
            assert!(error.to_string().contains("Invalid UTF-8"));
        }

        #[test]
        fn test_unsupported_format() {
            let error = AssetLoadError::unsupported_format("xyz");
            assert!(error.is_unsupported_format());
            assert!(error.to_string().contains("xyz"));
        }

        #[test]
        fn test_dependency_failed() {
            let error =
                AssetLoadError::dependency_failed("main.asset", "dependency.asset", "not found");
            assert!(error.is_dependency_failed());
            let msg = error.to_string();
            assert!(msg.contains("main.asset"));
            assert!(msg.contains("dependency.asset"));
            assert!(msg.contains("not found"));
        }

        #[test]
        fn test_custom() {
            let error = AssetLoadError::custom("Something went wrong");
            assert!(error.is_custom());
            assert!(error.to_string().contains("Something went wrong"));
        }

        #[test]
        fn test_clone() {
            let error = AssetLoadError::decode_failed("test");
            let cloned = error.clone();
            assert_eq!(error.to_string(), cloned.to_string());
        }
    }

    // LoadContext tests
    mod load_context {
        use super::*;

        #[test]
        fn test_new() {
            let path = AssetPath::from_string("assets/test.txt".to_string());
            let context = LoadContext::new(path);
            assert_eq!(context.path_str(), "assets/test.txt");
        }

        #[test]
        fn test_path_str() {
            let path = AssetPath::from_string("assets/test.txt".to_string());
            let context = LoadContext::new(path);
            assert_eq!(context.path_str(), "assets/test.txt");
        }

        #[test]
        fn test_extension() {
            let path = AssetPath::from_string("assets/test.txt".to_string());
            let context = LoadContext::new(path);
            assert_eq!(context.extension(), Some("txt"));
        }

        #[test]
        fn test_file_name() {
            let path = AssetPath::from_string("assets/test.txt".to_string());
            let context = LoadContext::new(path);
            assert_eq!(context.file_name(), Some("test.txt"));
        }

        #[test]
        fn test_debug() {
            let path = AssetPath::from_string("assets/test.txt".to_string());
            let context = LoadContext::new(path);
            let debug_str = format!("{context:?}");
            assert!(debug_str.contains("LoadContext"));
            assert!(debug_str.contains("assets/test.txt"));
        }
    }

    // AssetLoader tests
    mod asset_loader {
        use super::*;

        #[test]
        fn test_text_loader_extensions() {
            let loader = TextAssetLoader;
            assert_eq!(loader.extensions(), &["txt", "text"]);
        }

        #[test]
        fn test_text_loader_supports_extension() {
            let loader = TextAssetLoader;
            assert!(loader.supports_extension("txt"));
            assert!(loader.supports_extension("text"));
            assert!(loader.supports_extension("TXT")); // case-insensitive
            assert!(!loader.supports_extension("png"));
        }

        #[test]
        fn test_text_loader_load_success() {
            let loader = TextAssetLoader;
            let bytes = b"Hello, World!";
            let path = AssetPath::from_string("test.txt".to_string());
            let mut context = LoadContext::new(path);

            let result = loader.load(bytes, &(), &mut context);
            assert!(result.is_ok());
            let asset = result.unwrap();
            assert_eq!(asset.content, "Hello, World!");
        }

        #[test]
        fn test_text_loader_load_invalid_utf8() {
            let loader = TextAssetLoader;
            let bytes = &[0xFF, 0xFE, 0xFD]; // Invalid UTF-8
            let path = AssetPath::from_string("test.txt".to_string());
            let mut context = LoadContext::new(path);

            let result = loader.load(bytes, &(), &mut context);
            assert!(result.is_err());
            assert!(result.unwrap_err().is_decode_failed());
        }

        #[test]
        fn test_binary_loader() {
            let loader = BinaryAssetLoader;
            let bytes = vec![1, 2, 3, 4, 5];
            let path = AssetPath::from_string("test.bin".to_string());
            let mut context = LoadContext::new(path);

            let result = loader.load(&bytes, &(), &mut context);
            assert!(result.is_ok());
            let asset = result.unwrap();
            assert_eq!(asset.data, bytes);
        }

        #[test]
        fn test_loader_with_settings() {
            let loader = SettingsLoader;
            let settings = LoaderSettings { max_size: 10 };
            let bytes = vec![1, 2, 3, 4, 5]; // 5 bytes, under limit
            let path = AssetPath::from_string("test.custom".to_string());
            let mut context = LoadContext::new(path);

            let result = loader.load(&bytes, &settings, &mut context);
            assert!(result.is_ok());
        }

        #[test]
        fn test_loader_settings_limit_exceeded() {
            let loader = SettingsLoader;
            let settings = LoaderSettings { max_size: 3 };
            let bytes = vec![1, 2, 3, 4, 5]; // 5 bytes, over limit
            let path = AssetPath::from_string("test.custom".to_string());
            let mut context = LoadContext::new(path);

            let result = loader.load(&bytes, &settings, &mut context);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.is_custom());
            assert!(error.to_string().contains("too large"));
        }
    }

    // TypedAssetLoader tests
    mod typed_asset_loader {
        use super::*;

        #[test]
        fn test_new() {
            let loader = TypedAssetLoader::new(TextAssetLoader);
            assert_eq!(loader.extensions(), &["txt", "text"]);
        }

        #[test]
        fn test_with_settings() {
            let settings = LoaderSettings { max_size: 100 };
            let loader = TypedAssetLoader::with_settings(SettingsLoader, settings);
            assert_eq!(loader.settings().max_size, 100);
        }

        #[test]
        fn test_loader_accessor() {
            let typed = TypedAssetLoader::new(TextAssetLoader);
            let loader = typed.loader();
            assert_eq!(loader.extensions(), &["txt", "text"]);
        }

        #[test]
        fn test_settings_accessor() {
            let settings = LoaderSettings { max_size: 50 };
            let typed = TypedAssetLoader::with_settings(SettingsLoader, settings);
            assert_eq!(typed.settings().max_size, 50);
        }

        #[test]
        fn test_settings_mut_accessor() {
            let mut typed = TypedAssetLoader::new(SettingsLoader);
            typed.settings_mut().max_size = 200;
            assert_eq!(typed.settings().max_size, 200);
        }
    }

    // ErasedAssetLoader tests
    mod erased_asset_loader {
        use super::*;

        #[test]
        fn test_erased_extensions() {
            let typed = TypedAssetLoader::new(TextAssetLoader);
            let erased: &dyn ErasedAssetLoader = &typed;
            assert_eq!(erased.extensions(), &["txt", "text"]);
        }

        #[test]
        fn test_erased_supports_extension() {
            let typed = TypedAssetLoader::new(TextAssetLoader);
            let erased: &dyn ErasedAssetLoader = &typed;
            assert!(erased.supports_extension("txt"));
            assert!(!erased.supports_extension("png"));
        }

        #[test]
        fn test_erased_load_success() {
            let typed = TypedAssetLoader::new(TextAssetLoader);
            let erased: &dyn ErasedAssetLoader = &typed;
            let bytes = b"Hello, World!";
            let path = AssetPath::from_string("test.txt".to_string());
            let mut context = LoadContext::new(path);

            let result = erased.load_erased(bytes, &mut context);
            assert!(result.is_ok());

            let boxed = result.unwrap();
            let asset = boxed.downcast::<TextAsset>().unwrap();
            assert_eq!(asset.content, "Hello, World!");
        }

        #[test]
        fn test_erased_load_failure() {
            let typed = TypedAssetLoader::new(TextAssetLoader);
            let erased: &dyn ErasedAssetLoader = &typed;
            let bytes = &[0xFF, 0xFE]; // Invalid UTF-8
            let path = AssetPath::from_string("test.txt".to_string());
            let mut context = LoadContext::new(path);

            let result = erased.load_erased(bytes, &mut context);
            assert!(result.is_err());
        }

        #[test]
        fn test_multiple_loaders() {
            let text_loader: Box<dyn ErasedAssetLoader> =
                Box::new(TypedAssetLoader::new(TextAssetLoader));
            let binary_loader: Box<dyn ErasedAssetLoader> =
                Box::new(TypedAssetLoader::new(BinaryAssetLoader));

            assert!(text_loader.supports_extension("txt"));
            assert!(!text_loader.supports_extension("bin"));

            assert!(binary_loader.supports_extension("bin"));
            assert!(!binary_loader.supports_extension("txt"));
        }

        #[test]
        fn test_boxed_collection() {
            let loaders: Vec<Box<dyn ErasedAssetLoader>> = vec![
                Box::new(TypedAssetLoader::new(TextAssetLoader)),
                Box::new(TypedAssetLoader::new(BinaryAssetLoader)),
            ];

            assert_eq!(loaders.len(), 2);
            assert!(loaders[0].supports_extension("txt"));
            assert!(loaders[1].supports_extension("bin"));
        }
    }

    // Thread safety tests
    mod thread_safety {
        use super::*;

        #[test]
        fn test_asset_load_error_send() {
            fn assert_send<T: Send>() {}
            assert_send::<AssetLoadError>();
        }

        #[test]
        fn test_asset_load_error_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<AssetLoadError>();
        }

        #[test]
        fn test_text_loader_send() {
            fn assert_send<T: Send>() {}
            assert_send::<TextAssetLoader>();
        }

        #[test]
        fn test_text_loader_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<TextAssetLoader>();
        }

        #[test]
        fn test_typed_loader_send() {
            fn assert_send<T: Send>() {}
            assert_send::<TypedAssetLoader<TextAssetLoader>>();
        }

        #[test]
        fn test_typed_loader_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<TypedAssetLoader<TextAssetLoader>>();
        }
    }

    // Integration tests
    mod integration {
        use super::*;

        #[test]
        fn test_full_workflow() {
            // Create loader
            let loader = TypedAssetLoader::new(TextAssetLoader);

            // Load asset
            let bytes = b"Test content";
            let path = AssetPath::from_string("assets/test.txt".to_string());
            let mut context = LoadContext::new(path);

            let result = loader.load_erased(bytes, &mut context);
            assert!(result.is_ok());

            // Downcast to concrete type
            let boxed = result.unwrap();
            let asset = boxed.downcast::<TextAsset>().unwrap();
            assert_eq!(asset.content, "Test content");
        }

        #[test]
        fn test_loader_registry_pattern() {
            use std::collections::HashMap;

            let mut registry: HashMap<&str, Box<dyn ErasedAssetLoader>> = HashMap::new();
            registry.insert("txt", Box::new(TypedAssetLoader::new(TextAssetLoader)));
            registry.insert("bin", Box::new(TypedAssetLoader::new(BinaryAssetLoader)));

            // Look up loader by extension
            let loader = registry.get("txt").unwrap();
            assert!(loader.supports_extension("txt"));

            let loader = registry.get("bin").unwrap();
            assert!(loader.supports_extension("bin"));
        }

        #[test]
        fn test_error_propagation() {
            let loader = TypedAssetLoader::new(TextAssetLoader);
            let bytes = &[0xFF, 0xFE]; // Invalid UTF-8
            let path = AssetPath::from_string("test.txt".to_string());
            let mut context = LoadContext::new(path);

            let result = loader.load_erased(bytes, &mut context);
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.is_decode_failed());
        }
    }
}
