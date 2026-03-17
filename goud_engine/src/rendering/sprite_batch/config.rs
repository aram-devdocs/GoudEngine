//! Configuration for the sprite batch renderer.

use crate::assets::{
    loaders::{MaterialAsset, ShaderAsset},
    AssetHandle,
};

/// Configuration for sprite batch rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteBatchConfig {
    /// Initial capacity for vertex buffer (number of sprites).
    pub initial_capacity: usize,

    /// Maximum number of sprites per batch before automatic flush.
    pub max_batch_size: usize,

    /// Enable Z-layer sorting (disable for UI layers that don't need depth).
    pub enable_z_sorting: bool,

    /// Enable automatic batching by texture (disable for debugging).
    pub enable_batching: bool,

    /// Optional shader asset used by the batch renderer.
    pub shader_asset: AssetHandle<ShaderAsset>,

    /// Optional material asset used by the batch renderer.
    pub material_asset: AssetHandle<MaterialAsset>,
}

impl Default for SpriteBatchConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 1024, // Start with space for 1024 sprites
            max_batch_size: 10000,  // Flush after 10K sprites
            enable_z_sorting: true, // Sort by Z-layer by default
            enable_batching: true,  // Batch by texture by default
            shader_asset: AssetHandle::INVALID,
            material_asset: AssetHandle::INVALID,
        }
    }
}
