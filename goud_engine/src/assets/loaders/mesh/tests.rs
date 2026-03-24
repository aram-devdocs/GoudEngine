//! Tests for the mesh asset loader.

use crate::assets::loaders::mesh::asset::{MeshAsset, MeshBounds, MeshVertex, SubMesh};
use crate::assets::loaders::mesh::loader::{MeshFormat, MeshLoader};
use crate::assets::{Asset, AssetLoader, AssetPath, AssetType, LoadContext};

// ============================================================================
// MeshAsset unit tests
// ============================================================================

#[test]
fn test_mesh_asset_type() {
    assert_eq!(MeshAsset::asset_type(), AssetType::Mesh);
    assert_eq!(MeshAsset::asset_type_name(), "Mesh");
}

#[test]
fn test_mesh_asset_extensions() {
    let exts = MeshAsset::extensions();
    assert!(exts.contains(&"gltf"));
    assert!(exts.contains(&"glb"));
    assert!(exts.contains(&"obj"));
    assert!(exts.contains(&"fbx"));
}

#[test]
fn test_mesh_asset_counts() {
    let asset = MeshAsset {
        vertices: vec![
            MeshVertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 0.0],
            },
            MeshVertex {
                position: [1.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                uv: [1.0, 0.0],
            },
            MeshVertex {
                position: [0.0, 1.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 1.0],
            },
        ],
        indices: vec![0, 1, 2],
        sub_meshes: vec![],
        bounds: MeshBounds::from_positions(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
    };

    assert_eq!(asset.vertex_count(), 3);
    assert_eq!(asset.index_count(), 3);
    assert_eq!(asset.triangle_count(), 1);
    assert!(!asset.is_empty());
}

#[test]
fn test_mesh_asset_empty() {
    let asset = MeshAsset {
        vertices: vec![],
        indices: vec![],
        sub_meshes: vec![],
        bounds: MeshBounds::default(),
    };
    assert!(asset.is_empty());
    assert_eq!(asset.triangle_count(), 0);
}

#[test]
fn test_mesh_asset_to_interleaved_floats() {
    let asset = MeshAsset {
        vertices: vec![MeshVertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 0.5],
        }],
        indices: vec![0],
        sub_meshes: vec![],
        bounds: MeshBounds::from_positions(&[[1.0, 2.0, 3.0]]),
    };

    let floats = asset.to_interleaved_floats();
    assert_eq!(floats.len(), 8);
    assert_eq!(floats, vec![1.0, 2.0, 3.0, 0.0, 1.0, 0.0, 0.5, 0.5]);
}

#[test]
fn test_mesh_vertex_serde_roundtrip() {
    let vertex = MeshVertex {
        position: [1.0, 2.0, 3.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.5, 0.75],
    };
    let json = serde_json::to_string(&vertex).unwrap();
    let deserialized: MeshVertex = serde_json::from_str(&json).unwrap();
    assert_eq!(vertex, deserialized);
}

#[test]
fn test_sub_mesh_serde_roundtrip() {
    let sub = SubMesh {
        name: "body".into(),
        start_index: 10,
        index_count: 36,
        material_index: Some(2),
        material: None,
        bounds: MeshBounds::from_positions(&[[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]),
    };
    let json = serde_json::to_string(&sub).unwrap();
    let deserialized: SubMesh = serde_json::from_str(&json).unwrap();
    assert_eq!(sub, deserialized);
}

// ============================================================================
// MeshFormat tests
// ============================================================================

#[test]
fn test_mesh_format_from_extension() {
    assert_eq!(MeshFormat::from_extension("gltf"), Some(MeshFormat::Gltf));
    assert_eq!(MeshFormat::from_extension("GLTF"), Some(MeshFormat::Gltf));
    assert_eq!(MeshFormat::from_extension("glb"), Some(MeshFormat::Glb));
    assert_eq!(MeshFormat::from_extension("obj"), Some(MeshFormat::Obj));
    assert_eq!(MeshFormat::from_extension("fbx"), Some(MeshFormat::Fbx));
    assert_eq!(MeshFormat::from_extension("stl"), None);
}

// ============================================================================
// MeshLoader tests
// ============================================================================

#[test]
fn test_mesh_loader_extensions() {
    let loader = MeshLoader::new();
    let exts = AssetLoader::extensions(&loader);
    assert!(exts.contains(&"glb"));
    assert!(exts.contains(&"gltf"));
    assert!(exts.contains(&"obj"));
}

#[test]
fn test_mesh_loader_unsupported_extension() {
    let loader = MeshLoader::new();
    let path = AssetPath::from_string("model.stl".to_string());
    let mut ctx = LoadContext::new(path);
    let result = loader.load(b"dummy data", &(), &mut ctx);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_unsupported_format());
}

#[test]
fn test_mesh_loader_includes_fbx() {
    let loader = MeshLoader::new();
    let exts = AssetLoader::extensions(&loader);
    assert!(exts.contains(&"fbx"));
}

// ============================================================================
// ModelProviderRegistry tests
// ============================================================================

use crate::assets::loaders::mesh::provider::{ModelProvider, ModelProviderRegistry};

/// A trivial provider used only in tests.
struct StubProvider;

impl ModelProvider for StubProvider {
    fn name(&self) -> &str {
        "Stub"
    }

    fn extensions(&self) -> &[&str] {
        &["stub"]
    }

    fn load(
        &self,
        _bytes: &[u8],
        _context: &mut LoadContext,
    ) -> Result<crate::assets::loaders::mesh::provider::ModelData, crate::assets::AssetLoadError>
    {
        Ok(crate::assets::loaders::mesh::provider::ModelData {
            mesh: MeshAsset {
                vertices: vec![MeshVertex {
                    position: [0.0, 0.0, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    uv: [0.0, 0.0],
                }],
                indices: vec![0],
                sub_meshes: vec![],
                bounds: MeshBounds::default(),
            },
            skeleton: None,
            animations: vec![],
        })
    }
}

#[test]
fn test_registry_load_known_extension() {
    let mut registry = ModelProviderRegistry::new();
    registry.register(Box::new(StubProvider));

    let path = AssetPath::from_string("model.stub".to_string());
    let mut ctx = LoadContext::new(path);
    let result = registry.load("stub", b"dummy", &mut ctx);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.mesh.vertex_count(), 1);
    assert!(data.skeleton.is_none());
    assert!(data.animations.is_empty());
}

#[test]
fn test_registry_load_unknown_extension() {
    let registry = ModelProviderRegistry::new();
    let path = AssetPath::from_string("model.xyz".to_string());
    let mut ctx = LoadContext::new(path);
    let result = registry.load("xyz", b"dummy", &mut ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().is_unsupported_format());
}

#[test]
fn test_registry_case_insensitive_dispatch() {
    let mut registry = ModelProviderRegistry::new();
    registry.register(Box::new(StubProvider));

    let path = AssetPath::from_string("model.STUB".to_string());
    let mut ctx = LoadContext::new(path);
    let result = registry.load("STUB", b"dummy", &mut ctx);
    assert!(result.is_ok());
}

#[test]
fn test_registry_supported_extensions() {
    let mut registry = ModelProviderRegistry::new();
    registry.register(Box::new(StubProvider));
    let exts = registry.supported_extensions();
    assert_eq!(exts, vec!["stub"]);
}

#[cfg(feature = "native")]
#[test]
fn test_default_registry_extensions() {
    let registry = super::providers::default_registry();
    let exts = registry.supported_extensions();
    assert!(exts.contains(&"gltf"));
    assert!(exts.contains(&"glb"));
    assert!(exts.contains(&"obj"));
    assert!(exts.contains(&"fbx"));
}

#[cfg(feature = "native")]
#[test]
fn test_gltf_provider_metadata() {
    let provider = super::providers::GltfProvider;
    assert_eq!(provider.name(), "glTF");
    assert!(provider.extensions().contains(&"gltf"));
    assert!(provider.extensions().contains(&"glb"));
}

#[cfg(feature = "native")]
#[test]
fn test_obj_provider_metadata() {
    let provider = super::providers::ObjProvider;
    assert_eq!(provider.name(), "OBJ");
    assert!(provider.extensions().contains(&"obj"));
}

#[cfg(feature = "native")]
#[test]
fn test_fbx_provider_metadata() {
    let provider = super::providers::FbxProvider;
    assert_eq!(provider.name(), "FBX");
    assert!(provider.extensions().contains(&"fbx"));
}

// ============================================================================
// OBJ parser tests (feature = "native")
// ============================================================================

#[cfg(feature = "native")]
mod gltf_tests;
#[cfg(feature = "native")]
mod obj_tests;
