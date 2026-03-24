//! [`ModelProvider`] trait and [`ModelProviderRegistry`] for format-agnostic 3D model loading.
//!
//! The provider abstraction decouples the renderer and asset pipeline from
//! format-specific parsing code.  Each format (glTF, OBJ, FBX, ...) is
//! implemented behind the [`ModelProvider`] trait and registered in a
//! [`ModelProviderRegistry`].  The registry performs extension-based dispatch
//! so callers never touch format-specific types.

use crate::assets::{AssetLoadError, LoadContext};

use super::asset::MeshAsset;
use crate::assets::loaders::animation::KeyframeAnimation;

/// Format-agnostic skeleton data extracted from a model file.
///
/// Bone indices and weights are stored per-vertex, parallel to the mesh
/// vertex array.  Phase A0 providers may return `None` for skeleton data;
/// full extraction lands in Phase B.
#[derive(Debug, Clone)]
pub struct SkeletonData {
    /// Bones in the skeleton hierarchy.
    pub bones: Vec<BoneData>,
    /// Per-vertex bone indices (4 influences per vertex).
    pub bone_indices: Vec<[u32; 4]>,
    /// Per-vertex bone weights (4 influences per vertex).
    pub bone_weights: Vec<[f32; 4]>,
}

/// A single bone in a skeleton hierarchy.
#[derive(Debug, Clone)]
pub struct BoneData {
    /// Human-readable bone name.
    pub name: String,
    /// Index of the parent bone, or -1 for root bones.
    pub parent_index: i32,
    /// Column-major 4x4 inverse bind matrix.
    pub inverse_bind_matrix: [f32; 16],
}

/// Complete output of a model provider: mesh geometry plus optional
/// skeleton and animation data.
#[derive(Debug, Clone)]
pub struct ModelData {
    /// Parsed mesh geometry (vertices, indices, sub-meshes, bounds).
    pub mesh: MeshAsset,
    /// Optional skeleton for skinned meshes.
    pub skeleton: Option<SkeletonData>,
    /// Animations embedded in the model file.
    pub animations: Vec<KeyframeAnimation>,
}

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
