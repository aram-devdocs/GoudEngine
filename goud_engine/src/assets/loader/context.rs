//! Load context provided to asset loaders during loading.

use crate::assets::AssetPath;
use std::fmt;

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
