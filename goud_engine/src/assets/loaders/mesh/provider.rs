//! [`ModelProvider`] trait and [`ModelProviderRegistry`] for format-agnostic 3D model loading.
//!
//! The provider abstraction decouples the renderer and asset pipeline from
//! format-specific parsing code.  Each format (glTF, OBJ, FBX, ...) is
//! implemented behind the [`ModelProvider`] trait and registered in a
//! [`ModelProviderRegistry`].  The registry performs extension-based dispatch
//! so callers never touch format-specific types.
//!
//! The struct definitions for `SkeletonData`, `BoneData`, and `ModelData` now
//! live in `core::types::mesh_data`.  This module re-exports them for backward
//! compatibility.

use crate::assets::{AssetLoadError, LoadContext};

// Re-export skeleton and model data types from the canonical core location.
pub use crate::core::types::mesh_data::{BoneData, ModelData, SkeletonData};

/// A pluggable parser for a specific 3D model format.
///
/// Implementations convert raw file bytes into format-agnostic [`ModelData`].
/// Providers are registered in a [`ModelProviderRegistry`] and dispatched by
/// file extension.
pub trait ModelProvider: Send + Sync {
    /// Human-readable name of this provider (e.g. `"glTF"`, `"OBJ"`).
    fn name(&self) -> &str;

    /// File extensions this provider handles (lowercase, no leading dot).
    fn extensions(&self) -> &[&str];

    /// Parses raw file bytes into [`ModelData`].
    fn load(&self, bytes: &[u8], context: &mut LoadContext) -> Result<ModelData, AssetLoadError>;
}

/// Registry of [`ModelProvider`] implementations.
///
/// Extensions are matched case-insensitively.  The first provider whose
/// extension list contains the requested extension wins.
pub struct ModelProviderRegistry {
    providers: Vec<Box<dyn ModelProvider>>,
}

impl ModelProviderRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Registers a new provider.
    pub fn register(&mut self, provider: Box<dyn ModelProvider>) {
        self.providers.push(provider);
    }

    /// Loads a model by dispatching to the provider that handles `ext`.
    ///
    /// The extension is matched case-insensitively.  Returns
    /// [`AssetLoadError::UnsupportedFormat`] when no provider matches.
    pub fn load(
        &self,
        ext: &str,
        bytes: &[u8],
        context: &mut LoadContext,
    ) -> Result<ModelData, AssetLoadError> {
        let ext_lower = ext.to_ascii_lowercase();
        for provider in &self.providers {
            if provider.extensions().contains(&ext_lower.as_str()) {
                return provider.load(bytes, context);
            }
        }
        Err(AssetLoadError::unsupported_format(ext))
    }

    /// Returns a flat list of all extensions handled by registered providers.
    pub fn supported_extensions(&self) -> Vec<&str> {
        self.providers
            .iter()
            .flat_map(|p| p.extensions().iter().copied())
            .collect()
    }
}

impl Default for ModelProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ModelProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelProviderRegistry")
            .field("provider_count", &self.providers.len())
            .field("extensions", &self.supported_extensions())
            .finish()
    }
}
