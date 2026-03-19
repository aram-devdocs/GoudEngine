//! Load context provided to asset loaders during loading.

use crate::assets::{AssetLoadError, AssetPath};
use std::fmt;

type AssetReader<'a> = dyn Fn(&str) -> Result<Vec<u8>, AssetLoadError> + 'a;

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
    /// Byte-backed child assets emitted while loading this asset.
    embedded_assets: Vec<EmbeddedAsset>,
    /// Reader for sibling assets referenced during the current load.
    reader: Option<&'a AssetReader<'a>>,
    /// Marker for lifetime (in the future, this will hold references to AssetServer).
    _marker: std::marker::PhantomData<&'a ()>,
}

/// Byte-backed child asset declared by a loader.
#[derive(Clone, Debug)]
pub struct EmbeddedAsset {
    /// Logical asset path used for deduplication and dependency tracking.
    pub path: String,
    /// Raw asset bytes to feed back through the asset server.
    pub bytes: Vec<u8>,
}

impl<'a> LoadContext<'a> {
    /// Creates a new load context for the given asset path.
    pub fn new(asset_path: AssetPath<'static>) -> Self {
        Self {
            asset_path,
            dependencies: Vec::new(),
            embedded_assets: Vec::new(),
            reader: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a new load context with a read callback for sibling assets.
    pub(crate) fn with_reader(asset_path: AssetPath<'static>, reader: &'a AssetReader<'a>) -> Self {
        Self {
            asset_path,
            dependencies: Vec::new(),
            embedded_assets: Vec::new(),
            reader: Some(reader),
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

    /// Declares a byte-backed child asset emitted during this load.
    ///
    /// The asset server will materialize the child asset after the parent
    /// asset loads, then record the generated path as a dependency edge.
    pub fn add_embedded_asset(&mut self, path: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        let path = path.into();
        self.dependencies.push(path.clone());
        self.embedded_assets.push(EmbeddedAsset {
            path,
            bytes: bytes.into(),
        });
    }

    /// Returns the byte-backed child assets declared during loading.
    pub fn embedded_assets(&self) -> &[EmbeddedAsset] {
        &self.embedded_assets
    }

    /// Reads a sibling asset through the current asset server source.
    pub fn read_asset_bytes(&self, path: &str) -> Result<Vec<u8>, AssetLoadError> {
        let reader = self.reader.ok_or_else(|| {
            AssetLoadError::custom(format!(
                "Asset loader cannot read dependency '{}' without an attached reader",
                path
            ))
        })?;
        reader(path)
    }

    /// Consumes the context and returns the collected dependencies.
    pub(crate) fn into_parts(self) -> (Vec<String>, Vec<EmbeddedAsset>) {
        (self.dependencies, self.embedded_assets)
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
