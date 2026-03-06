//! Tests for `Transform2D` mutation methods (translate, rotate, scale).

use crate::core::math::Vec2;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::ecs::components::transform2d::types::Transform2D;

#[test]
fn test_translate() {
    let mut t = Transform2D::default();
    t.translate(Vec2::new(5.0, 10.0));
    assert_eq!(t.position, Vec2::new(5.0, 10.0));

    t.translate(Vec2::new(3.0, 2.0));
    assert_eq!(t.position, Vec2::new(8.0, 12.0));
}

#[test]
fn test_translate_local() {
    let mut t = Transform2D::from_rotation(FRAC_PI_2);
    t.translate_local(Vec2::new(1.0, 0.0));

    // 90 degree rotation: local X (1, 0) -> world (0, 1)
    assert!(t.position.x.abs() < 0.0001);
    assert!((t.position.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_set_position() {
    let mut t = Transform2D::from_position(Vec2::new(10.0, 20.0));
    t.set_position(Vec2::new(100.0, 200.0));
    assert_eq!(t.position, Vec2::new(100.0, 200.0));
}

#[test]
fn test_rotate() {
    let mut t = Transform2D::default();
    t.rotate(FRAC_PI_4);
    assert!((t.rotation - FRAC_PI_4).abs() < 0.0001);

    t.rotate(FRAC_PI_4);
    assert!((t.rotation - FRAC_PI_2).abs() < 0.0001);
}

#[test]
fn test_rotate_degrees() {
    let mut t = Transform2D::default();
    t.rotate_degrees(45.0);
    assert!((t.rotation - FRAC_PI_4).abs() < 0.0001);
}

#[test]
fn test_set_rotation() {
    let mut t = Transform2D::default();
    t.set_rotation(FRAC_PI_2);
    assert!((t.rotation - FRAC_PI_2).abs() < 0.0001);
}

#[test]
fn test_set_rotation_degrees() {
    let mut t = Transform2D::default();
    t.set_rotation_degrees(90.0);
    assert!((t.rotation - FRAC_PI_2).abs() < 0.0001);
}

#[test]
fn test_rotation_degrees() {
    let t = Transform2D::from_rotation(FRAC_PI_2);
    assert!((t.rotation_degrees() - 90.0).abs() < 0.01);
}

#[test]
fn test_look_at_target() {
    let mut t = Transform2D::from_position(Vec2::new(10.0, 10.0));
    t.look_at_target(Vec2::new(20.0, 10.0));
    assert!(t.rotation.abs() < 0.0001); // Looking right = 0 degrees
}

#[test]
fn test_set_scale() {
    let mut t = Transform2D::default();
    t.set_scale(Vec2::new(2.0, 3.0));
    assert_eq!(t.scale, Vec2::new(2.0, 3.0));
}

#[test]
fn test_set_scale_uniform() {
    let mut t = Transform2D::default();
    t.set_scale_uniform(5.0);
    assert_eq!(t.scale, Vec2::new(5.0, 5.0));
}

#[test]
fn test_scale_by() {
    let mut t = Transform2D::from_scale(Vec2::new(2.0, 3.0));
    t.scale_by(Vec2::new(2.0, 2.0));
    assert_eq!(t.scale, Vec2::new(4.0, 6.0));
}
