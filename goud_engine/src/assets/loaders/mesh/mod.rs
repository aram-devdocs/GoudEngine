//! Mesh asset loader.
//!
//! This module provides asset types and loaders for 3D mesh files.
//! Supports GLTF/GLB and OBJ formats (behind the `native` feature gate).
//!
//! # Example
//!
//! ```no_run
//! use goud_engine::assets::{AssetServer, loaders::mesh::MeshLoader, loaders::mesh::MeshAsset};
//!
//! let mut server = AssetServer::new();
//! server.register_loader(MeshLoader::default());
//!
//! let handle = server.load::<MeshAsset>("models/scene.glb");
//! ```
mod asset;
#[cfg(feature = "native")]
mod gltf_parser;
mod loader;
#[cfg(feature = "native")]
mod obj_parser;

#[cfg(test)]
mod tests;

pub use asset::{MeshAsset, MeshBounds, MeshMaterial, MeshVertex, SubMesh};
pub use loader::{MeshFormat, MeshLoader};
