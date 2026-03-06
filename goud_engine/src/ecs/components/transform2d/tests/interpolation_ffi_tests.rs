//! Tests for interpolation, component trait conformance, FFI layout, and utility functions.

use crate::core::math::Vec2;
use crate::ecs::Component;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use crate::ecs::components::transform2d::mat3x3::Mat3x3;
use crate::ecs::components::transform2d::ops::{lerp_angle, normalize_angle};
use crate::ecs::components::transform2d::types::Transform2D;

// =========================================================================
// Interpolation Tests
// =========================================================================

#[test]
fn test_lerp_position() {
    let a = Transform2D::from_position(Vec2::zero());
    let b = Transform2D::from_position(Vec2::new(10.0, 20.0));

    let mid = a.lerp(b, 0.5);
    assert_eq!(mid.position, Vec2::new(5.0, 10.0));
}

#[test]
fn test_lerp_scale() {
    let a = Transform2D::from_scale(Vec2::one());
    let b = Transform2D::from_scale(Vec2::new(3.0, 3.0));

    let mid = a.lerp(b, 0.5);
    assert_eq!(mid.scale, Vec2::new(2.0, 2.0));
}

#[test]
fn test_lerp_rotation() {
    let a = Transform2D::from_rotation(0.0);
    let b = Transform2D::from_rotation(FRAC_PI_2);

    let mid = a.lerp(b, 0.5);
    assert!((mid.rotation - FRAC_PI_4).abs() < 0.0001);
}

#[test]
fn test_lerp_rotation_shortest_path() {
    // From -170 degrees to 170 degrees should go through 180, not through 0
    let a = Transform2D::from_rotation(-170.0_f32.to_radians());
    let b = Transform2D::from_rotation(170.0_f32.to_radians());

    let mid = a.lerp(b, 0.5);
    // Should be close to 180 degrees (PI or -PI)
    assert!(mid.rotation.abs() > 3.0); // Close to PI
}

#[test]
fn test_lerp_endpoints() {
    let a = Transform2D::new(Vec2::zero(), 0.0, Vec2::one());
    let b = Transform2D::new(Vec2::new(10.0, 10.0), PI, Vec2::new(2.0, 2.0));

    let start = a.lerp(b, 0.0);
    assert_eq!(start.position, a.position);
    assert_eq!(start.scale, a.scale);

    let end = a.lerp(b, 1.0);
    assert_eq!(end.position, b.position);
    assert_eq!(end.scale, b.scale);
}

// =========================================================================
// Component Trait Tests
// =========================================================================

#[test]
fn test_transform2d_is_component() {
    fn assert_component<T: Component>() {}
    assert_component::<Transform2D>();
}

#[test]
fn test_transform2d_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Transform2D>();
}

#[test]
fn test_transform2d_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Transform2D>();
}

#[test]
fn test_transform2d_clone() {
    let t = Transform2D::new(Vec2::new(1.0, 2.0), FRAC_PI_4, Vec2::new(2.0, 3.0));
    let cloned = t.clone();
    assert_eq!(t, cloned);
}

#[test]
fn test_transform2d_copy() {
    let t = Transform2D::default();
    let copied = t;
    assert_eq!(t, copied);
}

// =========================================================================
// FFI Layout Tests
// =========================================================================

#[test]
fn test_transform2d_size() {
    use std::mem::size_of;
    // Vec2 (8) + f32 (4) + Vec2 (8) = 20 bytes
    assert_eq!(size_of::<Transform2D>(), 20);
}

#[test]
fn test_transform2d_align() {
    use std::mem::align_of;
    assert_eq!(align_of::<Transform2D>(), 4); // f32 alignment
}

#[test]
fn test_mat3x3_size() {
    use std::mem::size_of;
    // 9 * f32 = 36 bytes
    assert_eq!(size_of::<Mat3x3>(), 36);
}

#[test]
fn test_mat3x3_align() {
    use std::mem::align_of;
    assert_eq!(align_of::<Mat3x3>(), 4);
}

#[test]
fn test_transform2d_field_layout() {
    let t = Transform2D::new(Vec2::new(1.0, 2.0), 3.0, Vec2::new(4.0, 5.0));
    let ptr = &t as *const Transform2D as *const f32;
    // SAFETY: Transform2D is #[repr(C)] with known field layout.
    // Reading adjacent f32 values from a valid, aligned pointer is safe.
    unsafe {
        assert_eq!(*ptr, 1.0); // position.x
        assert_eq!(*ptr.add(1), 2.0); // position.y
        assert_eq!(*ptr.add(2), 3.0); // rotation
        assert_eq!(*ptr.add(3), 4.0); // scale.x
        assert_eq!(*ptr.add(4), 5.0); // scale.y
    }
}

// =========================================================================
// Utility Function Tests
// =========================================================================

#[test]
fn test_normalize_angle() {
    // Within range
    assert!((normalize_angle(0.0) - 0.0).abs() < 0.0001);
    assert!((normalize_angle(1.0) - 1.0).abs() < 0.0001);

    // Above PI
    assert!((normalize_angle(PI + 0.5) - (-PI + 0.5)).abs() < 0.0001);

    // Below -PI
    assert!((normalize_angle(-PI - 0.5) - (PI - 0.5)).abs() < 0.0001);

    // Large positive
    let result = normalize_angle(3.0 * PI);
    assert!(result >= -PI && result < PI);

    // Large negative
    let result = normalize_angle(-3.0 * PI);
    assert!(result >= -PI && result < PI);
}

#[test]
fn test_lerp_angle_same_direction() {
    let result = lerp_angle(0.0, FRAC_PI_2, 0.5);
    assert!((result - FRAC_PI_4).abs() < 0.0001);
}

#[test]
fn test_lerp_angle_across_boundary() {
    // From -170 to 170 should go through 180
    let from = -170.0_f32.to_radians();
    let to = 170.0_f32.to_radians();
    let mid = lerp_angle(from, to, 0.5);

    // Should be close to 180 degrees (PI or -PI)
    assert!(mid.abs() > 3.0);
}

#[test]
fn test_lerp_angle_endpoints() {
    let from = 0.5;
    let to = 1.5;

    assert!((lerp_angle(from, to, 0.0) - from).abs() < 0.0001);
    assert!((lerp_angle(from, to, 1.0) - to).abs() < 0.0001);
}
