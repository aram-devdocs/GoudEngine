//! Tests for direction, matrix, point-transform, interpolation, component trait, and FFI layout.

#[cfg(test)]
mod tests {
    use crate::core::math::Vec3;
    use crate::ecs::components::transform::core::Transform;
    use crate::ecs::components::transform::quat::Quat;
    use crate::ecs::Component;
    use std::f32::consts::FRAC_PI_2;
    use std::f32::consts::FRAC_PI_4;
    use std::f32::consts::PI;

    // =========================================================================
    // Transform Direction Tests
    // =========================================================================

    mod direction_tests {
        use super::*;

        #[test]
        fn test_directions_identity() {
            let t = Transform::default();

            assert!((t.forward() - Vec3::new(0.0, 0.0, -1.0)).length() < 0.0001);
            assert!((t.back() - Vec3::new(0.0, 0.0, 1.0)).length() < 0.0001);
            assert!((t.right() - Vec3::new(1.0, 0.0, 0.0)).length() < 0.0001);
            assert!((t.left() - Vec3::new(-1.0, 0.0, 0.0)).length() < 0.0001);
            assert!((t.up() - Vec3::new(0.0, 1.0, 0.0)).length() < 0.0001);
            assert!((t.down() - Vec3::new(0.0, -1.0, 0.0)).length() < 0.0001);
        }

        #[test]
        fn test_directions_rotated() {
            let mut t = Transform::default();
            t.rotate_y(FRAC_PI_2);

            // After +90 degree Y rotation (counter-clockwise looking down Y):
            // forward (-Z) -> -X
            // right (X) -> -Z
            let fwd = t.forward();
            assert!((fwd.x - (-1.0)).abs() < 0.0001);

            let right = t.right();
            assert!((right.z - (-1.0)).abs() < 0.0001);
        }
    }

    // =========================================================================
    // Matrix Tests
    // =========================================================================

    mod matrix_tests {
        use super::*;

        #[test]
        fn test_matrix_identity() {
            let t = Transform::default();
            let m = t.matrix();

            // Identity transform should produce identity matrix
            assert!((m.x.x - 1.0).abs() < 0.0001);
            assert!((m.y.y - 1.0).abs() < 0.0001);
            assert!((m.z.z - 1.0).abs() < 0.0001);
            assert!((m.w.w - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_translation() {
            let t = Transform::from_position(Vec3::new(10.0, 20.0, 30.0));
            let m = t.matrix();

            // Translation should be in the last column
            assert!((m.w.x - 10.0).abs() < 0.0001);
            assert!((m.w.y - 20.0).abs() < 0.0001);
            assert!((m.w.z - 30.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_scale() {
            let t = Transform::from_scale(Vec3::new(2.0, 3.0, 4.0));
            let m = t.matrix();

            // Scale should affect diagonal elements
            assert!((m.x.x - 2.0).abs() < 0.0001);
            assert!((m.y.y - 3.0).abs() < 0.0001);
            assert!((m.z.z - 4.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_inverse() {
            let t = Transform::new(
                Vec3::new(5.0, 10.0, 15.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );

            let m = t.matrix();
            let m_inv = t.matrix_inverse();

            // M * M^-1 should be identity
            let identity = m * m_inv;

            assert!((identity.x.x - 1.0).abs() < 0.001);
            assert!((identity.y.y - 1.0).abs() < 0.001);
            assert!((identity.z.z - 1.0).abs() < 0.001);
            assert!((identity.w.w - 1.0).abs() < 0.001);
        }
    }

    // =========================================================================
    // Point Transformation Tests
    // =========================================================================

    mod point_transform_tests {
        use super::*;

        #[test]
        fn test_transform_point_translation() {
            let t = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
            let p = Vec3::zero();
            let transformed = t.transform_point(p);
            assert_eq!(transformed, Vec3::new(10.0, 0.0, 0.0));
        }

        #[test]
        fn test_transform_point_scale() {
            let t = Transform::from_scale(Vec3::new(2.0, 2.0, 2.0));
            let p = Vec3::new(5.0, 5.0, 5.0);
            let transformed = t.transform_point(p);
            assert_eq!(transformed, Vec3::new(10.0, 10.0, 10.0));
        }

        #[test]
        fn test_transform_point_rotation() {
            let t = Transform::from_rotation(Quat::from_axis_angle(Vec3::unit_y(), PI));
            let p = Vec3::new(1.0, 0.0, 0.0);
            let transformed = t.transform_point(p);
            // 180 degree rotation should negate X
            assert!((transformed.x - (-1.0)).abs() < 0.0001);
            assert!(transformed.y.abs() < 0.0001);
            assert!(transformed.z.abs() < 0.0001);
        }

        #[test]
        fn test_transform_direction() {
            let t = Transform::new(
                Vec3::new(100.0, 0.0, 0.0), // Translation should not affect direction
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2),
                Vec3::one(),
            );

            let dir = Vec3::new(0.0, 0.0, 1.0);
            let transformed = t.transform_direction(dir);

            // After +90 degree Y rotation (counter-clockwise looking down Y),
            // +Z direction should rotate towards +X
            assert!((transformed.x - 1.0).abs() < 0.0001);
            assert!(transformed.y.abs() < 0.0001);
            assert!(transformed.z.abs() < 0.0001);
        }

        #[test]
        fn test_inverse_transform_point() {
            let t = Transform::new(
                Vec3::new(10.0, 20.0, 30.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );

            let world_point = Vec3::new(5.0, 5.0, 5.0);
            let local = t.inverse_transform_point(world_point);
            let back_to_world = t.transform_point(local);

            assert!((back_to_world - world_point).length() < 0.001);
        }

        #[test]
        fn test_inverse_transform_direction() {
            let t = Transform::from_rotation(Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2));

            let world_dir = Vec3::new(1.0, 0.0, 0.0);
            let local = t.inverse_transform_direction(world_dir);
            let back = t.transform_direction(local);

            assert!((back - world_dir).length() < 0.0001);
        }
    }

    // =========================================================================
    // Interpolation Tests
    // =========================================================================

    mod interpolation_tests {
        use super::*;

        #[test]
        fn test_lerp_position() {
            let a = Transform::from_position(Vec3::zero());
            let b = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));

            let mid = a.lerp(b, 0.5);
            assert_eq!(mid.position, Vec3::new(5.0, 0.0, 0.0));
        }

        #[test]
        fn test_lerp_scale() {
            let a = Transform::from_scale(Vec3::one());
            let b = Transform::from_scale(Vec3::new(3.0, 3.0, 3.0));

            let mid = a.lerp(b, 0.5);
            assert_eq!(mid.scale, Vec3::new(2.0, 2.0, 2.0));
        }

        #[test]
        fn test_lerp_rotation() {
            // Test lerp with a smaller rotation (avoid 180-degree edge case)
            let a = Transform::default();
            let b = Transform::from_rotation(Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2));

            let mid = a.lerp(b, 0.5);
            // Midpoint should be 45 degrees
            let expected = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            // Compare using dot product (handles sign flip)
            let dot = mid.rotation.x * expected.x
                + mid.rotation.y * expected.y
                + mid.rotation.z * expected.z
                + mid.rotation.w * expected.w;
            assert!(
                dot.abs() > 0.999,
                "lerp midpoint rotation should match expected"
            );
        }

        #[test]
        fn test_lerp_endpoints() {
            let a = Transform::new(
                Vec3::new(0.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::new(1.0, 1.0, 1.0),
            );
            let b = Transform::new(
                Vec3::new(10.0, 10.0, 10.0),
                Quat::from_axis_angle(Vec3::unit_y(), PI),
                Vec3::new(2.0, 2.0, 2.0),
            );

            let start = a.lerp(b, 0.0);
            assert_eq!(start.position, a.position);
            assert_eq!(start.scale, a.scale);

            let end = a.lerp(b, 1.0);
            assert_eq!(end.position, b.position);
            assert_eq!(end.scale, b.scale);
        }
    }

    // =========================================================================
    // Component Trait Tests
    // =========================================================================

    mod component_tests {
        use super::*;

        #[test]
        fn test_transform_is_component() {
            fn assert_component<T: Component>() {}
            assert_component::<Transform>();
        }

        #[test]
        fn test_transform_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<Transform>();
        }

        #[test]
        fn test_transform_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<Transform>();
        }

        #[test]
        fn test_transform_clone() {
            let t = Transform::new(
                Vec3::new(1.0, 2.0, 3.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );
            let cloned = t.clone();
            assert_eq!(t, cloned);
        }

        #[test]
        fn test_transform_copy() {
            let t = Transform::default();
            let copied = t;
            assert_eq!(t, copied);
        }
    }

    // =========================================================================
    // FFI Layout Tests
    // =========================================================================

    mod ffi_tests {
        use super::*;
        use std::mem::{align_of, size_of};

        #[test]
        fn test_quat_size() {
            assert_eq!(size_of::<Quat>(), 16); // 4 * f32
        }

        #[test]
        fn test_quat_align() {
            assert_eq!(align_of::<Quat>(), 4); // f32 alignment
        }

        #[test]
        fn test_transform_size() {
            // Vec3 (12) + Quat (16) + Vec3 (12) = 40 bytes
            assert_eq!(size_of::<Transform>(), 40);
        }

        #[test]
        fn test_transform_align() {
            assert_eq!(align_of::<Transform>(), 4); // f32 alignment
        }

        #[test]
        fn test_quat_field_layout() {
            let q = Quat::new(1.0, 2.0, 3.0, 4.0);
            let ptr = &q as *const Quat as *const f32;
            // SAFETY: Quat is #[repr(C)] with 4 consecutive f32 fields; pointer arithmetic
            // within the struct's bounds is valid.
            unsafe {
                assert_eq!(*ptr, 1.0);
                assert_eq!(*ptr.add(1), 2.0);
                assert_eq!(*ptr.add(2), 3.0);
                assert_eq!(*ptr.add(3), 4.0);
            }
        }
    }
}
