//! Tests for the `Mat3x3` type.

use crate::core::math::Vec2;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::ecs::components::transform2d::mat3x3::Mat3x3;

#[test]
fn test_identity() {
    let m = Mat3x3::IDENTITY;
    assert_eq!(m.m[0], 1.0);
    assert_eq!(m.m[4], 1.0);
    assert_eq!(m.m[8], 1.0);
}

#[test]
fn test_translation() {
    let m = Mat3x3::translation(10.0, 20.0);
    let p = m.transform_point(Vec2::zero());
    assert!((p.x - 10.0).abs() < 0.0001);
    assert!((p.y - 20.0).abs() < 0.0001);
}

#[test]
fn test_rotation() {
    let m = Mat3x3::rotation(FRAC_PI_2);
    let p = m.transform_point(Vec2::unit_x());
    // 90 degree rotation: (1, 0) -> (0, 1)
    assert!(p.x.abs() < 0.0001);
    assert!((p.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_scale() {
    let m = Mat3x3::scale(2.0, 3.0);
    let p = m.transform_point(Vec2::new(1.0, 1.0));
    assert!((p.x - 2.0).abs() < 0.0001);
    assert!((p.y - 3.0).abs() < 0.0001);
}

#[test]
fn test_multiply() {
    let t = Mat3x3::translation(10.0, 0.0);
    let r = Mat3x3::rotation(FRAC_PI_2);
    let combined = t * r;

    let p = combined.transform_point(Vec2::unit_x());
    // First rotate: (1, 0) -> (0, 1)
    // Then translate: (0, 1) -> (10, 1)
    assert!((p.x - 10.0).abs() < 0.0001);
    assert!((p.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_inverse() {
    let m = Mat3x3::translation(10.0, 20.0);
    let inv = m.inverse().unwrap();
    let result = m * inv;

    // Should be close to identity
    assert!((result.m[0] - 1.0).abs() < 0.0001);
    assert!((result.m[4] - 1.0).abs() < 0.0001);
    assert!((result.m[8] - 1.0).abs() < 0.0001);
}

#[test]
fn test_inverse_rotation() {
    let m = Mat3x3::rotation(FRAC_PI_4);
    let inv = m.inverse().unwrap();
    let result = m * inv;

    assert!((result.m[0] - 1.0).abs() < 0.0001);
    assert!((result.m[4] - 1.0).abs() < 0.0001);
}

#[test]
fn test_determinant() {
    let m = Mat3x3::IDENTITY;
    assert!((m.determinant() - 1.0).abs() < 0.0001);

    let s = Mat3x3::scale(2.0, 3.0);
    assert!((s.determinant() - 6.0).abs() < 0.0001);
}

#[test]
fn test_transform_direction() {
    let m = Mat3x3::translation(100.0, 100.0);
    let d = m.transform_direction(Vec2::unit_x());
    // Direction should not be affected by translation
    assert!((d.x - 1.0).abs() < 0.0001);
    assert!(d.y.abs() < 0.0001);
}

#[test]
fn test_to_mat4() {
    let m = Mat3x3::translation(10.0, 20.0);
    let m4 = m.to_mat4();

    // Check diagonal
    assert_eq!(m4[0], 1.0);
    assert_eq!(m4[5], 1.0);
    assert_eq!(m4[10], 1.0);
    assert_eq!(m4[15], 1.0);

    // Check translation
    assert_eq!(m4[12], 10.0);
    assert_eq!(m4[13], 20.0);
    assert_eq!(m4[14], 0.0);
}

#[test]
fn test_default() {
    assert_eq!(Mat3x3::default(), Mat3x3::IDENTITY);
}
