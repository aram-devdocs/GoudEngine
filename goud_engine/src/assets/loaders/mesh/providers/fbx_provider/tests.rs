//! Tests for FBX provider utilities.

use super::animation::{euler_xyz_to_quat, lerp_curve_at};
use super::helpers::{extract_vec2, extract_vec3, is_connected, strip_fbx_name, FbxConnections};
use super::{FbxProvider, DEFAULT_FBX_ROUGHNESS};

use crate::assets::loaders::mesh::provider::ModelProvider;
use crate::assets::LoadContext;

#[test]
fn test_euler_xyz_to_quat_identity() {
    let [x, y, z, w] = euler_xyz_to_quat(0.0, 0.0, 0.0);
    assert!(
        (w - 1.0).abs() < 1e-6,
        "w should be 1 for identity rotation"
    );
    assert!(x.abs() < 1e-6);
    assert!(y.abs() < 1e-6);
    assert!(z.abs() < 1e-6);
}

#[test]
fn test_euler_xyz_to_quat_90_x() {
    let [x, y, z, w] = euler_xyz_to_quat(90.0, 0.0, 0.0);
    let expected_sin = (45.0f32.to_radians()).sin();
    let expected_cos = (45.0f32.to_radians()).cos();
    assert!((x - expected_sin).abs() < 1e-5);
    assert!(y.abs() < 1e-5);
    assert!(z.abs() < 1e-5);
    assert!((w - expected_cos).abs() < 1e-5);
}

#[test]
fn test_euler_xyz_to_quat_unit_length() {
    let [x, y, z, w] = euler_xyz_to_quat(30.0, 45.0, 60.0);
    let len = (x * x + y * y + z * z + w * w).sqrt();
    assert!((len - 1.0).abs() < 1e-5, "quaternion should be unit length");
}

#[test]
fn test_strip_fbx_name_simple() {
    assert_eq!(strip_fbx_name("BoneName"), "BoneName");
}

#[test]
fn test_strip_fbx_name_with_prefix() {
    assert_eq!(strip_fbx_name("Model::Armature"), "Armature");
}

#[test]
fn test_strip_fbx_name_with_null() {
    assert_eq!(strip_fbx_name("Bone\x00\x01Blender"), "Bone");
}

#[test]
fn test_strip_fbx_name_double_prefix() {
    assert_eq!(strip_fbx_name("Model::Armature::Bone1"), "Bone1");
}

#[test]
fn test_lerp_curve_at_empty() {
    assert!((lerp_curve_at(1.0, &[]) - 0.0).abs() < 1e-6);
}

#[test]
fn test_lerp_curve_at_single() {
    let curve = [(0.5, 3.0)];
    assert!((lerp_curve_at(0.0, &curve) - 3.0).abs() < 1e-6);
    assert!((lerp_curve_at(1.0, &curve) - 3.0).abs() < 1e-6);
}

#[test]
fn test_lerp_curve_at_interpolation() {
    let curve = [(0.0, 0.0), (1.0, 10.0)];
    assert!((lerp_curve_at(0.5, &curve) - 5.0).abs() < 1e-5);
    assert!((lerp_curve_at(0.25, &curve) - 2.5).abs() < 1e-5);
}

#[test]
fn test_lerp_curve_at_clamp() {
    let curve = [(1.0, 5.0), (2.0, 15.0)];
    assert!((lerp_curve_at(0.0, &curve) - 5.0).abs() < 1e-6);
    assert!((lerp_curve_at(3.0, &curve) - 15.0).abs() < 1e-6);
}

#[test]
fn test_extract_vec3_valid() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    assert_eq!(extract_vec3(Some(&data), 0), Some([1.0, 2.0, 3.0]));
    assert_eq!(extract_vec3(Some(&data), 1), Some([4.0, 5.0, 6.0]));
}

#[test]
fn test_extract_vec3_out_of_bounds() {
    let data = [1.0, 2.0];
    assert_eq!(extract_vec3(Some(&data), 0), None);
}

#[test]
fn test_extract_vec3_none() {
    assert_eq!(extract_vec3(None, 0), None);
}

#[test]
fn test_extract_vec2_valid() {
    let data = [1.0, 2.0, 3.0, 4.0];
    assert_eq!(extract_vec2(Some(&data), 0), Some([1.0, 2.0]));
    assert_eq!(extract_vec2(Some(&data), 1), Some([3.0, 4.0]));
}

#[test]
fn test_is_connected_bidirectional() {
    let mut conns = FbxConnections::default();
    conns.children_of.entry(100).or_default().push(200);
    conns.parents_of.entry(200).or_default().push(100);

    assert!(is_connected(200, 100, &conns));
    assert!(is_connected(100, 200, &conns));
    assert!(!is_connected(300, 100, &conns));
}

#[test]
fn test_fbx_provider_extensions() {
    let provider = FbxProvider;
    let exts = provider.extensions();
    assert!(
        exts.contains(&"fbx"),
        "FbxProvider should support the 'fbx' extension"
    );
    assert!(
        !exts.contains(&"obj"),
        "FbxProvider should not support 'obj'"
    );
}

#[test]
fn test_fbx_provider_name() {
    let provider = FbxProvider;
    assert_eq!(provider.name(), "FBX");
}

#[test]
fn test_fbx_provider_empty_input() {
    let provider = FbxProvider;
    let mut ctx = LoadContext::new(crate::assets::AssetPath::new("test.fbx").into_owned());
    let result = provider.load(&[], &mut ctx);
    assert!(
        result.is_err(),
        "Loading empty bytes should return an error"
    );
}

#[test]
fn test_fbx_provider_invalid_magic() {
    let provider = FbxProvider;
    let mut ctx = LoadContext::new(crate::assets::AssetPath::new("test.fbx").into_owned());
    let result = provider.load(b"not a valid fbx file", &mut ctx);
    assert!(
        result.is_err(),
        "Loading invalid FBX magic bytes should return an error"
    );
}

#[test]
fn test_default_fbx_roughness_constant() {
    assert!(
        (DEFAULT_FBX_ROUGHNESS - 0.5).abs() < f32::EPSILON,
        "DEFAULT_FBX_ROUGHNESS should be 0.5"
    );
}

#[test]
fn test_extract_vec2_out_of_bounds() {
    let data = [1.0];
    assert_eq!(extract_vec2(Some(&data), 0), None);
}

#[test]
fn test_extract_vec2_none() {
    assert_eq!(extract_vec2(None, 0), None);
}
