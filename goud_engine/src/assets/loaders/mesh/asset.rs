//! [`MeshAsset`] -- CPU-side mesh data (vertices, indices, sub-meshes).
//!
//! The struct definitions now live in `core::types::mesh_data` so that both
//! the asset pipeline (Layer 3) and the renderer (Layer 2) can use them
//! without layer violations.  This module re-exports them for backward
//! compatibility and provides the [`Asset`] trait implementation.

use crate::assets::{Asset, AssetType};

// Re-export all mesh data types from the canonical core location.
pub use crate::core::types::mesh_data::{MeshAsset, MeshBounds, MeshMaterial, MeshVertex, SubMesh};

impl Asset for MeshAsset {
    fn asset_type_name() -> &'static str {
        "Mesh"
    }

    fn asset_type() -> AssetType {
        AssetType::Mesh
    }

    fn extensions() -> &'static [&'static str] {
        &["gltf", "glb", "obj", "fbx"]
    }
}
