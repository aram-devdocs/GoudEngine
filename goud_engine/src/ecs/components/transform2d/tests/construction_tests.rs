//! Tests for `Transform2D` constructors and `Default` impl.

use crate::core::math::Vec2;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::ecs::components::transform2d::types::Transform2D;

#[test]
fn test_default() {
    let t = Transform2D::default();
    assert_eq!(t.position, Vec2::zero());
    assert_eq!(t.rotation, 0.0);
    assert_eq!(t.scale, Vec2::one());
}

#[test]
fn test_new() {
    let pos = Vec2::new(10.0, 20.0);
    let rot = FRAC_PI_4;
    let scale = Vec2::new(2.0, 3.0);

    let t = Transform2D::new(pos, rot, scale);
    assert_eq!(t.position, pos);
    assert_eq!(t.rotation, rot);
    assert_eq!(t.scale, scale);
}

#[test]
fn test_from_position() {
    let pos = Vec2::new(100.0, 50.0);
    let t = Transform2D::from_position(pos);
    assert_eq!(t.position, pos);
    assert_eq!(t.rotation, 0.0);
    assert_eq!(t.scale, Vec2::one());
}

#[test]
fn test_from_rotation() {
    let t = Transform2D::from_rotation(FRAC_PI_2);
    assert_eq!(t.position, Vec2::zero());
    assert_eq!(t.rotation, FRAC_PI_2);
    assert_eq!(t.scale, Vec2::one());
}

#[test]
fn test_from_rotation_degrees() {
    let t = Transform2D::from_rotation_degrees(90.0);
    assert!((t.rotation - FRAC_PI_2).abs() < 0.0001);
}

#[test]
fn test_from_scale() {
    let scale = Vec2::new(2.0, 3.0);
    let t = Transform2D::from_scale(scale);
    assert_eq!(t.position, Vec2::zero());
    assert_eq!(t.rotation, 0.0);
    assert_eq!(t.scale, scale);
}

#[test]
fn test_from_scale_uniform() {
    let t = Transform2D::from_scale_uniform(2.0);
    assert_eq!(t.scale, Vec2::new(2.0, 2.0));
}

#[test]
fn test_from_position_rotation() {
    let pos = Vec2::new(10.0, 20.0);
    let t = Transform2D::from_position_rotation(pos, FRAC_PI_4);
    assert_eq!(t.position, pos);
    assert_eq!(t.rotation, FRAC_PI_4);
    assert_eq!(t.scale, Vec2::one());
}

#[test]
fn test_look_at() {
    let t = Transform2D::look_at(Vec2::zero(), Vec2::new(1.0, 0.0));
    assert!(t.rotation.abs() < 0.0001); // Should be 0 (looking right)

    let t2 = Transform2D::look_at(Vec2::zero(), Vec2::new(0.0, 1.0));
    assert!((t2.rotation - FRAC_PI_2).abs() < 0.0001); // Should be 90 degrees
}
