//! Mesh asset loader.
//!
//! This module provides asset types and loaders for 3D mesh files.
//! Supports GLTF/GLB, OBJ, and FBX formats (behind the `native` feature gate).
//!
//! # Architecture
//!
//! Format-specific parsing is abstracted behind [`ModelProvider`].  Each
//! provider converts raw file bytes into format-agnostic [`ModelData`].
//! The [`ModelProviderRegistry`] dispatches by extension so callers never
//! touch format-specific types.
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
mod provider;
#[cfg(feature = "native")]
mod providers;

#[cfg(test)]
mod tests;

pub use asset::{MeshAsset, MeshBounds, MeshMaterial, MeshVertex, SubMesh};
pub use loader::{MeshFormat, MeshLoader};
pub use provider::{BoneData, ModelData, ModelProvider, ModelProviderRegistry, SkeletonData};
#[cfg(feature = "native")]
pub use providers::{default_registry, FbxProvider, GltfProvider, ObjProvider};
