//! Tests for direction vectors, matrix generation, and point transformation.

use crate::core::math::Vec2;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::ecs::components::transform2d::types::Transform2D;

// =========================================================================
// Direction Tests
// =========================================================================

#[test]
fn test_directions_identity() {
    let t = Transform2D::default();

    let fwd = t.forward();
    assert!((fwd.x - 1.0).abs() < 0.0001);
    assert!(fwd.y.abs() < 0.0001);

    let right = t.right();
    assert!(right.x.abs() < 0.0001);
    assert!((right.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_directions_rotated() {
    let t = Transform2D::from_rotation(FRAC_PI_2);

    // After 90 degree rotation:
    // forward (1, 0) -> (0, 1)
    let fwd = t.forward();
    assert!(fwd.x.abs() < 0.0001);
    assert!((fwd.y - 1.0).abs() < 0.0001);

    // right (0, 1) -> (-1, 0)
    let right = t.right();
    assert!((right.x - (-1.0)).abs() < 0.0001);
    assert!(right.y.abs() < 0.0001);
}

#[test]
fn test_backward_and_left() {
    let t = Transform2D::default();

    let back = t.backward();
    assert!((back.x - (-1.0)).abs() < 0.0001);

    let left = t.left();
    assert!((left.y - (-1.0)).abs() < 0.0001);
}

// =========================================================================
// Matrix Tests
// =========================================================================

#[test]
fn test_matrix_identity() {
    let t = Transform2D::default();
    let m = t.matrix();

    // Should be close to identity
    assert!((m.m[0] - 1.0).abs() < 0.0001);
    assert!((m.m[4] - 1.0).abs() < 0.0001);
    assert!((m.m[8] - 1.0).abs() < 0.0001);
}

#[test]
fn test_matrix_translation() {
    let t = Transform2D::from_position(Vec2::new(10.0, 20.0));
    let m = t.matrix();

    assert!((m.m[6] - 10.0).abs() < 0.0001);
    assert!((m.m[7] - 20.0).abs() < 0.0001);
}

#[test]
fn test_matrix_scale() {
    let t = Transform2D::from_scale(Vec2::new(2.0, 3.0));
    let m = t.matrix();

    assert!((m.m[0] - 2.0).abs() < 0.0001);
    assert!((m.m[4] - 3.0).abs() < 0.0001);
}

#[test]
fn test_matrix_rotation() {
    let t = Transform2D::from_rotation(FRAC_PI_2);
    let m = t.matrix();

    let p = m.transform_point(Vec2::new(1.0, 0.0));
    // 90 degree rotation: (1, 0) -> (0, 1)
    assert!(p.x.abs() < 0.0001);
    assert!((p.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_matrix_inverse() {
    let t = Transform2D::new(Vec2::new(10.0, 20.0), FRAC_PI_4, Vec2::new(2.0, 3.0));

    let m = t.matrix();
    let m_inv = t.matrix_inverse();

    let result = m * m_inv;

    // Should be close to identity
    assert!((result.m[0] - 1.0).abs() < 0.001);
    assert!((result.m[4] - 1.0).abs() < 0.001);
    assert!((result.m[8] - 1.0).abs() < 0.001);
    assert!(result.m[6].abs() < 0.001);
    assert!(result.m[7].abs() < 0.001);
}

#[test]
fn test_to_mat4() {
    let t = Transform2D::from_position(Vec2::new(5.0, 10.0));
    let m4 = t.to_mat4();

    // Check translation
    assert_eq!(m4[12], 5.0);
    assert_eq!(m4[13], 10.0);
    assert_eq!(m4[14], 0.0);

    // Check diagonal
    assert_eq!(m4[0], 1.0);
    assert_eq!(m4[5], 1.0);
    assert_eq!(m4[10], 1.0);
    assert_eq!(m4[15], 1.0);
}

// =========================================================================
// Point Transformation Tests
// =========================================================================

#[test]
fn test_transform_point_translation() {
    let t = Transform2D::from_position(Vec2::new(10.0, 20.0));
    let p = t.transform_point(Vec2::zero());
    assert_eq!(p, Vec2::new(10.0, 20.0));
}

#[test]
fn test_transform_point_scale() {
    let t = Transform2D::from_scale(Vec2::new(2.0, 3.0));
    let p = t.transform_point(Vec2::new(5.0, 5.0));
    assert_eq!(p, Vec2::new(10.0, 15.0));
}

#[test]
fn test_transform_point_rotation() {
    let t = Transform2D::from_rotation(FRAC_PI_2);
    let p = t.transform_point(Vec2::new(1.0, 0.0));
    // 90 degree rotation: (1, 0) -> (0, 1)
    assert!(p.x.abs() < 0.0001);
    assert!((p.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_transform_direction() {
    let t = Transform2D::new(
        Vec2::new(100.0, 100.0), // Translation should not affect direction
        FRAC_PI_2,
        Vec2::one(),
    );

    let dir = Vec2::new(1.0, 0.0);
    let transformed = t.transform_direction(dir);

    // 90 degree rotation: (1, 0) -> (0, 1)
    assert!(transformed.x.abs() < 0.0001);
    assert!((transformed.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_inverse_transform_point() {
    let t = Transform2D::new(Vec2::new(10.0, 20.0), FRAC_PI_4, Vec2::new(2.0, 2.0));

    let world_point = Vec2::new(5.0, 5.0);
    let local = t.inverse_transform_point(world_point);
    let back_to_world = t.transform_point(local);

    assert!((back_to_world.x - world_point.x).abs() < 0.001);
    assert!((back_to_world.y - world_point.y).abs() < 0.001);
}

#[test]
fn test_inverse_transform_direction() {
    let t = Transform2D::from_rotation(FRAC_PI_2);

    let world_dir = Vec2::new(1.0, 0.0);
    let local = t.inverse_transform_direction(world_dir);
    let back = t.transform_direction(local);

    assert!((back.x - world_dir.x).abs() < 0.0001);
    assert!((back.y - world_dir.y).abs() < 0.0001);
}
