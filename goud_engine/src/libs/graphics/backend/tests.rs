//! Unit tests for the graphics backend module.

use super::blend::{BlendFactor, CullFace};
use super::capabilities::{BackendCapabilities, BackendInfo};

#[test]
fn test_backend_capabilities_default() {
    let caps = BackendCapabilities::default();
    assert_eq!(caps.max_texture_units, 16);
    assert_eq!(caps.max_texture_size, 8192);
    assert_eq!(caps.max_vertex_attributes, 16);
    assert!(caps.supports_instancing);
    assert!(!caps.supports_compute_shaders);
}

#[test]
fn test_backend_capabilities_clone() {
    let caps1 = BackendCapabilities::default();
    let caps2 = caps1.clone();
    assert_eq!(caps1, caps2);
}

#[test]
fn test_backend_info_clone() {
    let info = BackendInfo {
        name: "TestBackend",
        version: "1.0".to_string(),
        vendor: "Test".to_string(),
        renderer: "Test Renderer".to_string(),
        capabilities: BackendCapabilities::default(),
    };
    let info2 = info.clone();
    assert_eq!(info, info2);
}

#[test]
fn test_blend_factor_variants() {
    assert_eq!(BlendFactor::Zero as u8, 0);
    assert_eq!(BlendFactor::One as u8, 1);
    assert_eq!(BlendFactor::SrcAlpha as u8, 6);
    assert_eq!(BlendFactor::OneMinusSrcAlpha as u8, 7);
}

#[test]
fn test_cull_face_variants() {
    assert_eq!(CullFace::Front as u8, 0);
    assert_eq!(CullFace::Back as u8, 1);
    assert_eq!(CullFace::FrontAndBack as u8, 2);
}

#[test]
fn test_cull_face_default() {
    assert_eq!(CullFace::default(), CullFace::Back);
}

#[test]
fn test_blend_factor_copy() {
    let f1 = BlendFactor::SrcAlpha;
    let f2 = f1;
    assert_eq!(f1, f2);
}

#[test]
fn test_cull_face_copy() {
    let c1 = CullFace::Back;
    let c2 = c1;
    assert_eq!(c1, c2);
}
