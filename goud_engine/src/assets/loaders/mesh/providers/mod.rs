//! Built-in [`ModelProvider`](super::provider::ModelProvider) implementations.
//!
//! Each sub-module wraps a format-specific parser behind the provider trait.
//! [`default_registry`] returns a registry pre-loaded with all built-in
//! providers.

mod fbx_provider;
mod gltf_helpers;
mod gltf_provider;
mod obj_provider;

pub use fbx_provider::FbxProvider;
pub use gltf_provider::GltfProvider;
pub use obj_provider::ObjProvider;

use super::provider::ModelProviderRegistry;

/// Returns a [`ModelProviderRegistry`] populated with all built-in providers.
pub fn default_registry() -> ModelProviderRegistry {
    let mut registry = ModelProviderRegistry::new();
    registry.register(Box::new(GltfProvider));
    registry.register(Box::new(ObjProvider));
    registry.register(Box::new(FbxProvider));
    registry
}
