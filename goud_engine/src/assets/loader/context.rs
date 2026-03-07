//! Load context provided to asset loaders during loading.

use crate::assets::AssetPath;
use std::fmt;

/// Context provided to asset loaders during loading.
///
/// The context provides:
/// - The path of the asset being loaded
/// - Methods to declare dependencies on other assets
/// - Metadata about the loading process
///
/// Loaders call [`add_dependency`](LoadContext::add_dependency) to declare
/// that the asset being loaded depends on another asset. After loading
/// completes, the `AssetServer` records these edges in the dependency
/// graph so that cascade reloads work correctly.
pub struct LoadContext<'a> {
    /// The path of the asset being loaded (owned).
    asset_path: AssetPath<'static>,
    /// Paths of assets that this asset depends on.
    dependencies: Vec<String>,
    /// Marker for lifetime (in the future, this will hold references to AssetServer).
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> LoadContext<'a> {
    /// Creates a new load context for the given asset path.
    pub fn new(asset_path: AssetPath<'static>) -> Self {
        Self {
            asset_path,
            dependencies: Vec::new(),
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

    /// Declares that the asset being loaded depends on another asset.
    ///
    /// After the load completes, the `AssetServer` will record this
    /// dependency so that changes to `dependency_path` trigger a
    /// cascade reload of the current asset.
    ///
    /// # Arguments
    ///
    /// * `dependency_path` - Path of the asset this asset depends on
    pub fn add_dependency(&mut self, dependency_path: impl Into<String>) {
        self.dependencies.push(dependency_path.into());
    }

    /// Returns the list of dependencies declared during loading.
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    /// Consumes the context and returns the collected dependencies.
    pub(crate) fn into_dependencies(self) -> Vec<String> {
        self.dependencies
    }
}

impl<'a> fmt::Debug for LoadContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadContext")
            .field("asset_path", &self.asset_path)
            .field("dependencies", &self.dependencies.len())
            .finish()
    }
}
