//! Tests for the mesh asset loader.

use crate::assets::loaders::mesh::asset::{MeshAsset, MeshVertex, SubMesh};
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
mod obj_tests {
    use super::*;

    const TRIANGLE_OBJ: &[u8] = b"\
# Simple triangle
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
f 1/1/1 2/2/2 3/3/3
";

    #[test]
    fn test_obj_parse_triangle() {
        let loader = MeshLoader::new();
        let path = AssetPath::from_string("triangle.obj".to_string());
        let mut ctx = LoadContext::new(path);
        let asset = loader.load(TRIANGLE_OBJ, &(), &mut ctx).unwrap();

        assert_eq!(asset.vertex_count(), 3);
        assert_eq!(asset.index_count(), 3);
        assert_eq!(asset.triangle_count(), 1);
        assert_eq!(asset.sub_meshes.len(), 1);

        // Check positions
        assert_eq!(asset.vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(asset.vertices[1].position, [1.0, 0.0, 0.0]);
        assert_eq!(asset.vertices[2].position, [0.0, 1.0, 0.0]);

        // Check normals
        assert_eq!(asset.vertices[0].normal, [0.0, 0.0, 1.0]);

        // Check UVs
        assert_eq!(asset.vertices[0].uv, [0.0, 0.0]);
        assert_eq!(asset.vertices[1].uv, [1.0, 0.0]);
    }

    #[test]
    fn test_obj_parse_multi_object() {
        let obj_data = b"\
o Cube
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
o Plane
v 2.0 0.0 0.0
v 3.0 0.0 0.0
v 2.0 1.0 0.0
f 4 5 6
";
        let loader = MeshLoader::new();
        let path = AssetPath::from_string("multi.obj".to_string());
        let mut ctx = LoadContext::new(path);
        let asset = loader.load(obj_data, &(), &mut ctx).unwrap();

        assert_eq!(asset.sub_meshes.len(), 2);
        assert_eq!(asset.sub_meshes[0].name, "Cube");
        assert_eq!(asset.sub_meshes[1].name, "Plane");
        assert_eq!(asset.vertex_count(), 6);
        assert_eq!(asset.triangle_count(), 2);
    }

    #[test]
    fn test_obj_parse_no_normals_defaults() {
        let obj_data = b"\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
";
        let loader = MeshLoader::new();
        let path = AssetPath::from_string("no_normals.obj".to_string());
        let mut ctx = LoadContext::new(path);
        let asset = loader.load(obj_data, &(), &mut ctx).unwrap();

        // Default normal should be [0, 0, 1]
        assert_eq!(asset.vertices[0].normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_obj_parse_invalid_data() {
        let loader = MeshLoader::new();
        let path = AssetPath::from_string("bad.obj".to_string());
        let mut ctx = LoadContext::new(path);
        let result = loader.load(b"not valid obj data {\x00\x01}", &(), &mut ctx);
        // tobj may either error or produce empty -- either way we handle it
        // Empty mesh should produce a decode error from our code
        assert!(result.is_err() || result.unwrap().is_empty());
    }

    #[test]
    fn test_obj_submesh_indices() {
        let obj_data = b"\
o Triangle
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
";
        let loader = MeshLoader::new();
        let path = AssetPath::from_string("sub.obj".to_string());
        let mut ctx = LoadContext::new(path);
        let asset = loader.load(obj_data, &(), &mut ctx).unwrap();

        let sub = &asset.sub_meshes[0];
        assert_eq!(sub.start_index, 0);
        assert_eq!(sub.index_count, 3);
        assert_eq!(sub.material_index, None);
    }
}

// ============================================================================
// GLTF parser tests (feature = "native")
// ============================================================================

#[cfg(feature = "native")]
mod gltf_tests {
    use super::*;

    /// Builds a minimal valid GLB containing a single triangle.
    ///
    /// The GLB binary format is:
    ///   Header (12 bytes): magic + version + total_length
    ///   Chunk 0 (JSON): length + type + data
    ///   Chunk 1 (BIN):  length + type + data
    fn build_triangle_glb() -> Vec<u8> {
        // Triangle: 3 vertices (position only), 3 indices (u16)
        //
        // Positions: 3 * 3 * 4 = 36 bytes (3 vec3 floats)
        // Indices:   3 * 2     =  6 bytes (3 u16) + 2 padding = 8 bytes
        // Total BIN = 44 bytes

        // -- BIN chunk data --
        let mut bin = Vec::new();

        // Positions (accessor 1, buffer view 0, offset 0, count 3)
        // v0 = (0, 0, 0)
        bin.extend_from_slice(&0.0f32.to_le_bytes());
        bin.extend_from_slice(&0.0f32.to_le_bytes());
        bin.extend_from_slice(&0.0f32.to_le_bytes());
        // v1 = (1, 0, 0)
        bin.extend_from_slice(&1.0f32.to_le_bytes());
        bin.extend_from_slice(&0.0f32.to_le_bytes());
        bin.extend_from_slice(&0.0f32.to_le_bytes());
        // v2 = (0, 1, 0)
        bin.extend_from_slice(&0.0f32.to_le_bytes());
        bin.extend_from_slice(&1.0f32.to_le_bytes());
        bin.extend_from_slice(&0.0f32.to_le_bytes());

        // Indices (accessor 0, buffer view 1, offset 36, count 3)
        bin.extend_from_slice(&0u16.to_le_bytes());
        bin.extend_from_slice(&1u16.to_le_bytes());
        bin.extend_from_slice(&2u16.to_le_bytes());
        // Pad to 4-byte alignment
        bin.extend_from_slice(&[0u8; 2]);

        assert_eq!(bin.len(), 44);

        // -- JSON chunk --
        let json_str = serde_json::json!({
            "asset": { "version": "2.0" },
            "buffers": [{ "byteLength": 44 }],
            "bufferViews": [
                {
                    "buffer": 0,
                    "byteOffset": 0,
                    "byteLength": 36,
                    "target": 34962
                },
                {
                    "buffer": 0,
                    "byteOffset": 36,
                    "byteLength": 6,
                    "target": 34963
                }
            ],
            "accessors": [
                {
                    "bufferView": 1,
                    "componentType": 5123,
                    "count": 3,
                    "type": "SCALAR",
                    "max": [2],
                    "min": [0]
                },
                {
                    "bufferView": 0,
                    "componentType": 5126,
                    "count": 3,
                    "type": "VEC3",
                    "max": [1.0, 1.0, 0.0],
                    "min": [0.0, 0.0, 0.0]
                }
            ],
            "meshes": [{
                "name": "Triangle",
                "primitives": [{
                    "attributes": { "POSITION": 1 },
                    "indices": 0
                }]
            }],
            "nodes": [{ "mesh": 0 }],
            "scenes": [{ "nodes": [0] }],
            "scene": 0
        })
        .to_string();

        let mut json_bytes = json_str.into_bytes();
        // Pad JSON to 4-byte alignment with spaces
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }

        // -- Assemble GLB --
        let json_chunk_len = json_bytes.len() as u32;
        let bin_chunk_len = bin.len() as u32;
        let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

        let mut glb = Vec::with_capacity(total_len as usize);

        // Header
        glb.extend_from_slice(b"glTF"); // magic
        glb.extend_from_slice(&2u32.to_le_bytes()); // version
        glb.extend_from_slice(&total_len.to_le_bytes()); // total length

        // JSON chunk
        glb.extend_from_slice(&json_chunk_len.to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json_bytes);

        // BIN chunk
        glb.extend_from_slice(&bin_chunk_len.to_le_bytes());
        glb.extend_from_slice(&[0x42, 0x49, 0x4E, 0x00]); // BIN\0 chunk type
        glb.extend_from_slice(&bin);

        glb
    }

    #[test]
    fn test_gltf_parse_triangle_glb() {
        let glb = build_triangle_glb();
        let loader = MeshLoader::new();
        let path = AssetPath::from_string("triangle.glb".to_string());
        let mut ctx = LoadContext::new(path);
        let asset = loader.load(&glb, &(), &mut ctx).unwrap();

        assert_eq!(asset.vertex_count(), 3);
        assert_eq!(asset.index_count(), 3);
        assert_eq!(asset.triangle_count(), 1);

        // Check positions
        assert_eq!(asset.vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(asset.vertices[1].position, [1.0, 0.0, 0.0]);
        assert_eq!(asset.vertices[2].position, [0.0, 1.0, 0.0]);

        // No normals in the GLB, should default to [0, 0, 1]
        assert_eq!(asset.vertices[0].normal, [0.0, 0.0, 1.0]);

        // No UVs in the GLB, should default to [0, 0]
        assert_eq!(asset.vertices[0].uv, [0.0, 0.0]);
    }

    #[test]
    fn test_gltf_submesh_extraction() {
        let glb = build_triangle_glb();
        let loader = MeshLoader::new();
        let path = AssetPath::from_string("test.glb".to_string());
        let mut ctx = LoadContext::new(path);
        let asset = loader.load(&glb, &(), &mut ctx).unwrap();

        assert_eq!(asset.sub_meshes.len(), 1);
        assert_eq!(asset.sub_meshes[0].name, "Triangle");
        assert_eq!(asset.sub_meshes[0].start_index, 0);
        assert_eq!(asset.sub_meshes[0].index_count, 3);
        assert_eq!(asset.sub_meshes[0].material_index, None);
    }

    #[test]
    fn test_gltf_parse_invalid_data() {
        let loader = MeshLoader::new();
        let path = AssetPath::from_string("bad.glb".to_string());
        let mut ctx = LoadContext::new(path);
        let result = loader.load(b"not a valid glb file", &(), &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_gltf_parse_empty_glb() {
        // Valid GLTF JSON with no meshes
        let json_str = serde_json::json!({
            "asset": { "version": "2.0" }
        })
        .to_string();

        let mut json_bytes = json_str.into_bytes();
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }

        let json_chunk_len = json_bytes.len() as u32;
        let total_len = 12 + 8 + json_chunk_len;

        let mut glb = Vec::new();
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&total_len.to_le_bytes());
        glb.extend_from_slice(&json_chunk_len.to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json_bytes);

        let loader = MeshLoader::new();
        let path = AssetPath::from_string("empty.glb".to_string());
        let mut ctx = LoadContext::new(path);
        let result = loader.load(&glb, &(), &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }
}
