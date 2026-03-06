//! Tests for Quat, Transform construction, and mutation operations.

use crate::core::math::{Quaternion, Vec3};
use crate::ecs::components::transform::core::Transform;
use crate::ecs::components::transform::quat::Quat;
use std::f32::consts::FRAC_PI_2;
use std::f32::consts::FRAC_PI_4;
use std::f32::consts::PI;

// =========================================================================
// Quat Tests
// =========================================================================

mod quat_tests {
    use super::*;

    #[test]
    fn test_quat_identity() {
        let q = Quat::IDENTITY;
        assert_eq!(q.x, 0.0);
        assert_eq!(q.y, 0.0);
        assert_eq!(q.z, 0.0);
        assert_eq!(q.w, 1.0);
    }

    #[test]
    fn test_quat_new() {
        let q = Quat::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q.x, 1.0);
        assert_eq!(q.y, 2.0);
        assert_eq!(q.z, 3.0);
        assert_eq!(q.w, 4.0);
    }

    #[test]
    fn test_quat_from_axis_angle() {
        let q = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2);
        assert!((q.length() - 1.0).abs() < 0.0001);

        // Rotating unit_z by 90 degrees around Y should give unit_x
        let rotated = q.rotate_vector(Vec3::unit_z());
        assert!((rotated.x - 1.0).abs() < 0.0001);
        assert!(rotated.y.abs() < 0.0001);
        assert!(rotated.z.abs() < 0.0001);
    }

    #[test]
    fn test_quat_from_euler() {
        // 90 degree yaw rotation
        let q = Quat::from_euler(0.0, FRAC_PI_2, 0.0);
        assert!((q.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_quat_normalize() {
        let q = Quat::new(1.0, 2.0, 3.0, 4.0);
        let n = q.normalize();
        assert!((n.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_quat_conjugate() {
        let q = Quat::new(1.0, 2.0, 3.0, 4.0);
        let c = q.conjugate();
        assert_eq!(c.x, -1.0);
        assert_eq!(c.y, -2.0);
        assert_eq!(c.z, -3.0);
        assert_eq!(c.w, 4.0);
    }

    #[test]
    fn test_quat_mul_identity() {
        let q = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
        let result = q * Quat::IDENTITY;
        assert!((result.x - q.x).abs() < 0.0001);
        assert!((result.y - q.y).abs() < 0.0001);
        assert!((result.z - q.z).abs() < 0.0001);
        assert!((result.w - q.w).abs() < 0.0001);
    }

    #[test]
    fn test_quat_rotate_vector() {
        let q = Quat::from_axis_angle(Vec3::unit_y(), PI);
        let v = Vec3::new(1.0, 0.0, 0.0);
        let rotated = q.rotate_vector(v);
        // 180 degree rotation around Y should negate X
        assert!((rotated.x - (-1.0)).abs() < 0.0001);
        assert!(rotated.y.abs() < 0.0001);
        assert!(rotated.z.abs() < 0.0001);
    }

    #[test]
    fn test_quat_slerp() {
        // Test slerp with a smaller rotation (avoid 180-degree edge case)
        let a = Quat::IDENTITY;
        let b = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2);
        let mid = a.slerp(b, 0.5);
        // Midpoint should be 45 degrees
        let expected = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
        // Compare quaternion components (accounting for sign flip equivalence)
        let dot = mid.x * expected.x + mid.y * expected.y + mid.z * expected.z + mid.w * expected.w;
        assert!(
            dot.abs() > 0.999,
            "slerp midpoint should represent same rotation"
        );
    }

    #[test]
    fn test_quat_directions() {
        let q = Quat::IDENTITY;
        let fwd = q.forward();
        let right = q.right();
        let up = q.up();

        assert!((fwd - Vec3::new(0.0, 0.0, -1.0)).length() < 0.0001);
        assert!((right - Vec3::new(1.0, 0.0, 0.0)).length() < 0.0001);
        assert!((up - Vec3::new(0.0, 1.0, 0.0)).length() < 0.0001);
    }

    #[test]
    fn test_quat_default() {
        let q = Quat::default();
        assert_eq!(q, Quat::IDENTITY);
    }

    #[test]
    fn test_quat_cgmath_conversion() {
        let our_quat = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
        let cg_quat: Quaternion<f32> = our_quat.into();
        let back: Quat = cg_quat.into();

        assert!((back.x - our_quat.x).abs() < 0.0001);
        assert!((back.y - our_quat.y).abs() < 0.0001);
        assert!((back.z - our_quat.z).abs() < 0.0001);
        assert!((back.w - our_quat.w).abs() < 0.0001);
    }

    #[test]
    fn test_quat_inverse() {
        let q = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4).normalize();
        let inv = q.inverse();
        let result = q * inv;
        // q * q^-1 should be identity
        assert!((result.x).abs() < 0.0001);
        assert!((result.y).abs() < 0.0001);
        assert!((result.z).abs() < 0.0001);
        assert!((result.w - 1.0).abs() < 0.0001);
    }
}

// =========================================================================
// Transform Construction Tests
// =========================================================================

mod construction_tests {
    use super::*;

    #[test]
    fn test_transform_default() {
        let t = Transform::default();
        assert_eq!(t.position, Vec3::zero());
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::one());
    }

    #[test]
    fn test_transform_new() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let rot = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
        let scale = Vec3::new(2.0, 2.0, 2.0);

        let t = Transform::new(pos, rot, scale);
        assert_eq!(t.position, pos);
        assert_eq!(t.rotation, rot);
        assert_eq!(t.scale, scale);
    }

    #[test]
    fn test_transform_from_position() {
        let pos = Vec3::new(10.0, 20.0, 30.0);
        let t = Transform::from_position(pos);
        assert_eq!(t.position, pos);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::one());
    }

    #[test]
    fn test_transform_from_rotation() {
        let rot = Quat::from_axis_angle(Vec3::unit_x(), FRAC_PI_2);
        let t = Transform::from_rotation(rot);
        assert_eq!(t.position, Vec3::zero());
        assert_eq!(t.rotation, rot);
        assert_eq!(t.scale, Vec3::one());
    }

    #[test]
    fn test_transform_from_scale() {
        let scale = Vec3::new(2.0, 3.0, 4.0);
        let t = Transform::from_scale(scale);
        assert_eq!(t.position, Vec3::zero());
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, scale);
    }

    #[test]
    fn test_transform_from_scale_uniform() {
        let t = Transform::from_scale_uniform(5.0);
        assert_eq!(t.scale, Vec3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_transform_from_position_rotation() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let rot = Quat::from_axis_angle(Vec3::unit_z(), FRAC_PI_4);
        let t = Transform::from_position_rotation(pos, rot);
        assert_eq!(t.position, pos);
        assert_eq!(t.rotation, rot);
        assert_eq!(t.scale, Vec3::one());
    }

    #[test]
    fn test_transform_look_at() {
        let eye = Vec3::new(0.0, 0.0, 10.0);
        let target = Vec3::zero();
        let up = Vec3::unit_y();

        let t = Transform::look_at(eye, target, up);
        assert_eq!(t.position, eye);

        // Forward direction should point towards target
        let fwd = t.forward();
        let expected_fwd = (target - eye).normalize();
        assert!((fwd - expected_fwd).length() < 0.01);
    }
}

// =========================================================================
// Transform Mutation Tests
// =========================================================================

mod mutation_tests {
    use super::*;

    #[test]
    fn test_translate() {
        let mut t = Transform::default();
        t.translate(Vec3::new(5.0, 0.0, 0.0));
        assert_eq!(t.position, Vec3::new(5.0, 0.0, 0.0));

        t.translate(Vec3::new(0.0, 3.0, 0.0));
        assert_eq!(t.position, Vec3::new(5.0, 3.0, 0.0));
    }

    #[test]
    fn test_translate_local() {
        let mut t = Transform::default();
        // Rotate 90 degrees around Y, so local X becomes world Z
        t.rotate_y(FRAC_PI_2);
        t.translate_local(Vec3::new(1.0, 0.0, 0.0));

        // Local X (1,0,0) should become world Z direction after 90 degree Y rotation
        assert!(t.position.x.abs() < 0.0001);
        assert!(t.position.y.abs() < 0.0001);
        assert!((t.position.z - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_set_position() {
        let mut t = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
        t.set_position(Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(t.position, Vec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_rotate_x() {
        let mut t = Transform::default();
        t.rotate_x(FRAC_PI_2);

        // After +90 degree X rotation (counter-clockwise looking down X),
        // up (Y) should rotate towards +Z
        let up = t.up();
        assert!(up.x.abs() < 0.0001);
        assert!(up.y.abs() < 0.0001);
        assert!((up.z - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_rotate_y() {
        let mut t = Transform::default();
        t.rotate_y(FRAC_PI_2);

        // After +90 degree Y rotation (counter-clockwise looking down Y),
        // forward (-Z) should rotate towards -X
        let fwd = t.forward();
        assert!((fwd.x - (-1.0)).abs() < 0.0001);
        assert!(fwd.y.abs() < 0.0001);
        assert!(fwd.z.abs() < 0.0001);
    }

    #[test]
    fn test_rotate_z() {
        let mut t = Transform::default();
        t.rotate_z(FRAC_PI_2);

        // After 90 degree Z rotation, right (X) should become up (Y)
        let right = t.right();
        assert!(right.x.abs() < 0.0001);
        assert!((right.y - 1.0).abs() < 0.0001);
        assert!(right.z.abs() < 0.0001);
    }

    #[test]
    fn test_set_rotation() {
        let mut t = Transform::default();
        let rot = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
        t.set_rotation(rot);
        assert!((t.rotation.x - rot.x).abs() < 0.0001);
        assert!((t.rotation.y - rot.y).abs() < 0.0001);
        assert!((t.rotation.z - rot.z).abs() < 0.0001);
        assert!((t.rotation.w - rot.w).abs() < 0.0001);
    }

    #[test]
    fn test_set_scale() {
        let mut t = Transform::default();
        t.set_scale(Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(t.scale, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_set_scale_uniform() {
        let mut t = Transform::default();
        t.set_scale_uniform(3.0);
        assert_eq!(t.scale, Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn test_scale_by() {
        let mut t = Transform::from_scale(Vec3::new(2.0, 2.0, 2.0));
        t.scale_by(Vec3::new(3.0, 4.0, 5.0));
        assert_eq!(t.scale, Vec3::new(6.0, 8.0, 10.0));
    }

    #[test]
    fn test_look_at_target() {
        let mut t = Transform::from_position(Vec3::new(0.0, 0.0, 10.0));
        t.look_at_target(Vec3::zero(), Vec3::unit_y());

        let fwd = t.forward();
        let expected = Vec3::new(0.0, 0.0, -1.0);
        assert!((fwd - expected).length() < 0.01);
    }
}
