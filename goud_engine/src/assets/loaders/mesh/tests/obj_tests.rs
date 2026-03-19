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
    assert_eq!(asset.vertices[0].position, [0.0, 0.0, 0.0]);
    assert_eq!(asset.vertices[1].position, [1.0, 0.0, 0.0]);
    assert_eq!(asset.vertices[2].position, [0.0, 1.0, 0.0]);
    assert_eq!(asset.vertices[0].normal, [0.0, 0.0, 1.0]);
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
    assert_eq!(asset.vertices[0].normal, [0.0, 0.0, 1.0]);
}

#[test]
fn test_obj_parse_invalid_data() {
    let loader = MeshLoader::new();
    let path = AssetPath::from_string("bad.obj".to_string());
    let mut ctx = LoadContext::new(path);
    let result = loader.load(b"not valid obj data {\x00\x01}", &(), &mut ctx);
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
