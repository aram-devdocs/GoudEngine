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
    assert_eq!(MeshFormat::from_extension("fbx"), None);
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
    let path = AssetPath::from_string("model.fbx".to_string());
    let mut ctx = LoadContext::new(path);
    let result = loader.load(b"dummy data", &(), &mut ctx);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_unsupported_format());
}

// ============================================================================
// OBJ parser tests (feature = "native")
// ============================================================================

#[cfg(feature = "native")]
mod gltf_tests;
#[cfg(feature = "native")]
mod obj_tests;
