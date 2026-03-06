//! Unit tests for [`GlobalTransform`].

#[cfg(test)]
mod tests {
    use crate::core::math::Vec3;
    use crate::ecs::components::global_transform::GlobalTransform;
    use crate::ecs::components::transform::{Quat, Transform};
    use crate::ecs::Component;
    use std::f32::consts::FRAC_PI_2;
    use std::f32::consts::FRAC_PI_4;

    mod construction_tests {
        use super::*;

        #[test]
        fn test_identity() {
            let global = GlobalTransform::IDENTITY;
            let pos = global.translation();
            let scale = global.scale();

            assert!((pos.x).abs() < 0.0001);
            assert!((pos.y).abs() < 0.0001);
            assert!((pos.z).abs() < 0.0001);
            assert!((scale.x - 1.0).abs() < 0.0001);
            assert!((scale.y - 1.0).abs() < 0.0001);
            assert!((scale.z - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_default() {
            let global: GlobalTransform = Default::default();
            assert_eq!(global, GlobalTransform::IDENTITY);
        }

        #[test]
        fn test_from_translation() {
            let global = GlobalTransform::from_translation(Vec3::new(10.0, 20.0, 30.0));
            let pos = global.translation();

            assert!((pos.x - 10.0).abs() < 0.0001);
            assert!((pos.y - 20.0).abs() < 0.0001);
            assert!((pos.z - 30.0).abs() < 0.0001);
        }

        #[test]
        fn test_from_translation_rotation_scale() {
            let rotation = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::new(5.0, 10.0, 15.0),
                rotation,
                Vec3::new(2.0, 3.0, 4.0),
            );

            let pos = global.translation();
            let scale = global.scale();

            assert!((pos.x - 5.0).abs() < 0.0001);
            assert!((pos.y - 10.0).abs() < 0.0001);
            assert!((pos.z - 15.0).abs() < 0.0001);
            assert!((scale.x - 2.0).abs() < 0.0001);
            assert!((scale.y - 3.0).abs() < 0.0001);
            assert!((scale.z - 4.0).abs() < 0.0001);
        }

        #[test]
        fn test_from_transform() {
            let transform = Transform::new(
                Vec3::new(1.0, 2.0, 3.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );
            let global: GlobalTransform = transform.into();

            let pos = global.translation();
            assert!((pos.x - 1.0).abs() < 0.0001);
            assert!((pos.y - 2.0).abs() < 0.0001);
            assert!((pos.z - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_from_transform_ref() {
            let transform = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
            let global: GlobalTransform = (&transform).into();
            let pos = global.translation();
            assert!((pos.x - 5.0).abs() < 0.0001);
        }
    }

    mod decomposition_tests {
        use super::*;

        #[test]
        fn test_translation_extraction() {
            let global = GlobalTransform::from_translation(Vec3::new(1.0, 2.0, 3.0));
            let pos = global.translation();
            assert!((pos.x - 1.0).abs() < 0.0001);
            assert!((pos.y - 2.0).abs() < 0.0001);
            assert!((pos.z - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_scale_extraction() {
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::IDENTITY,
                Vec3::new(2.0, 3.0, 4.0),
            );
            let scale = global.scale();
            assert!((scale.x - 2.0).abs() < 0.0001);
            assert!((scale.y - 3.0).abs() < 0.0001);
            assert!((scale.z - 4.0).abs() < 0.0001);
        }

        #[test]
        fn test_rotation_extraction() {
            let original = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                original,
                Vec3::one(),
            );
            let extracted = global.rotation();

            // Compare quaternions (accounting for sign flip)
            let dot = original.x * extracted.x
                + original.y * extracted.y
                + original.z * extracted.z
                + original.w * extracted.w;
            assert!(dot.abs() > 0.999);
        }

        #[test]
        fn test_decompose() {
            let original_t = Vec3::new(10.0, 5.0, 3.0);
            let original_r = Quat::from_axis_angle(Vec3::unit_x(), FRAC_PI_4);
            let original_s = Vec3::new(2.0, 3.0, 4.0);

            let global = GlobalTransform::from_translation_rotation_scale(
                original_t, original_r, original_s,
            );
            let (t, r, s) = global.decompose();

            assert!((t - original_t).length() < 0.001);
            assert!((s - original_s).length() < 0.001);

            let dot =
                original_r.x * r.x + original_r.y * r.y + original_r.z * r.z + original_r.w * r.w;
            assert!(dot.abs() > 0.999);
        }

        #[test]
        fn test_to_transform() {
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::new(5.0, 10.0, 15.0),
                Quat::IDENTITY,
                Vec3::new(2.0, 2.0, 2.0),
            );

            let transform = global.to_transform();
            assert!((transform.position - Vec3::new(5.0, 10.0, 15.0)).length() < 0.001);
            assert!((transform.scale - Vec3::new(2.0, 2.0, 2.0)).length() < 0.001);
        }
    }

    mod transform_tests {
        use super::*;

        #[test]
        fn test_mul_transform_translation() {
            let a = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let b = GlobalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0));
            let result = a.mul_transform(&b);

            let pos = result.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }

        #[test]
        fn test_mul_transform_scale() {
            let a = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::IDENTITY,
                Vec3::new(2.0, 2.0, 2.0),
            );
            let b = GlobalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0));
            let result = a.mul_transform(&b);

            let pos = result.translation();
            // Scale affects the child's translation
            assert!((pos.x - 10.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_by() {
            let parent = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let child = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
            let result = parent.transform_by(&child);

            let pos = result.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_point() {
            let global = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let local_point = Vec3::new(5.0, 3.0, 0.0);
            let world_point = global.transform_point(local_point);

            assert!((world_point.x - 15.0).abs() < 0.0001);
            assert!((world_point.y - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_direction() {
            let global = GlobalTransform::from_translation(Vec3::new(1000.0, 0.0, 0.0));
            let direction = Vec3::new(1.0, 0.0, 0.0);
            let world_dir = global.transform_direction(direction);

            // Direction should not be affected by translation
            assert!((world_dir.x - 1.0).abs() < 0.0001);
            assert!(world_dir.y.abs() < 0.0001);
        }

        #[test]
        fn test_inverse() {
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::new(5.0, 10.0, 15.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );

            let inverse = global.inverse().expect("Should be invertible");
            let identity = global.mul_transform(&inverse);

            // Should be close to identity
            let pos = identity.translation();
            assert!(pos.length() < 0.001);

            let scale = identity.scale();
            assert!((scale.x - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_mul_operator() {
            let a = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let b = GlobalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0));
            let result = a * b;

            let pos = result.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }

        #[test]
        fn test_mul_operator_with_transform() {
            let parent = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let child = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
            let result = parent * child;

            let pos = result.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }
    }

    mod direction_tests {
        use super::*;

        #[test]
        fn test_directions_identity() {
            let global = GlobalTransform::IDENTITY;

            assert!((global.forward() - Vec3::new(0.0, 0.0, -1.0)).length() < 0.0001);
            assert!((global.back() - Vec3::new(0.0, 0.0, 1.0)).length() < 0.0001);
            assert!((global.right() - Vec3::new(1.0, 0.0, 0.0)).length() < 0.0001);
            assert!((global.left() - Vec3::new(-1.0, 0.0, 0.0)).length() < 0.0001);
            assert!((global.up() - Vec3::new(0.0, 1.0, 0.0)).length() < 0.0001);
            assert!((global.down() - Vec3::new(0.0, -1.0, 0.0)).length() < 0.0001);
        }

        #[test]
        fn test_directions_rotated() {
            let global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2),
                Vec3::one(),
            );

            // After +90 degree Y rotation:
            // forward (-Z) -> -X
            let fwd = global.forward();
            assert!((fwd.x - (-1.0)).abs() < 0.0001);
            assert!(fwd.y.abs() < 0.0001);
            assert!(fwd.z.abs() < 0.0001);
        }
    }

    mod interpolation_tests {
        use super::*;

        #[test]
        fn test_lerp_translation() {
            let a = GlobalTransform::from_translation(Vec3::zero());
            let b = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));

            let mid = a.lerp(&b, 0.5);
            let pos = mid.translation();
            assert!((pos.x - 5.0).abs() < 0.0001);
        }

        #[test]
        fn test_lerp_endpoints() {
            let a = GlobalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0));
            let b = GlobalTransform::from_translation(Vec3::new(10.0, 10.0, 10.0));

            let start = a.lerp(&b, 0.0);
            assert!((start.translation() - a.translation()).length() < 0.0001);

            let end = a.lerp(&b, 1.0);
            assert!((end.translation() - b.translation()).length() < 0.0001);
        }
    }

    mod array_tests {
        use super::*;

        #[test]
        fn test_to_cols_array() {
            let global = GlobalTransform::IDENTITY;
            let cols = global.to_cols_array();

            // Identity matrix
            assert_eq!(cols[0], 1.0); // m00
            assert_eq!(cols[5], 1.0); // m11
            assert_eq!(cols[10], 1.0); // m22
            assert_eq!(cols[15], 1.0); // m33
        }

        #[test]
        fn test_to_rows_array() {
            let global = GlobalTransform::from_translation(Vec3::new(10.0, 20.0, 30.0));
            let rows = global.to_rows_array();

            // Translation is in the last row for row-major
            assert!((rows[3] - 10.0).abs() < 0.0001);
            assert!((rows[7] - 20.0).abs() < 0.0001);
            assert!((rows[11] - 30.0).abs() < 0.0001);
        }
    }

    mod component_tests {
        use super::*;

        #[test]
        fn test_is_component() {
            fn assert_component<T: Component>() {}
            assert_component::<GlobalTransform>();
        }

        #[test]
        fn test_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<GlobalTransform>();
        }

        #[test]
        fn test_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<GlobalTransform>();
        }

        #[test]
        fn test_clone() {
            let global = GlobalTransform::from_translation(Vec3::new(1.0, 2.0, 3.0));
            let cloned = global.clone();
            assert_eq!(global, cloned);
        }

        #[test]
        fn test_copy() {
            let global = GlobalTransform::IDENTITY;
            let copied = global;
            assert_eq!(global, copied);
        }

        #[test]
        fn test_debug() {
            let global = GlobalTransform::from_translation(Vec3::new(10.0, 5.0, 0.0));
            let debug = format!("{:?}", global);
            assert!(debug.contains("GlobalTransform"));
            assert!(debug.contains("translation"));
        }
    }

    mod hierarchy_tests {
        use super::*;

        #[test]
        fn test_parent_child_translation() {
            // Parent at (10, 0, 0)
            let parent_global = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));

            // Child at local (5, 0, 0)
            let child_local = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));

            // Child's global should be (15, 0, 0)
            let child_global = parent_global.transform_by(&child_local);
            let pos = child_global.translation();
            assert!((pos.x - 15.0).abs() < 0.0001);
        }

        #[test]
        fn test_parent_child_rotation() {
            // Parent rotated 90 degrees around Y
            let parent_global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2),
                Vec3::one(),
            );

            // Child at local (0, 0, -10) - in front of parent
            let child_local = Transform::from_position(Vec3::new(0.0, 0.0, -10.0));

            // After parent rotation, child should be at (-10, 0, 0)
            let child_global = parent_global.transform_by(&child_local);
            let pos = child_global.translation();
            assert!((pos.x - (-10.0)).abs() < 0.01);
            assert!(pos.y.abs() < 0.01);
            assert!(pos.z.abs() < 0.01);
        }

        #[test]
        fn test_parent_child_scale() {
            // Parent scaled 2x
            let parent_global = GlobalTransform::from_translation_rotation_scale(
                Vec3::zero(),
                Quat::IDENTITY,
                Vec3::new(2.0, 2.0, 2.0),
            );

            // Child at local (5, 0, 0)
            let child_local = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));

            // Child's global position should be (10, 0, 0)
            let child_global = parent_global.transform_by(&child_local);
            let pos = child_global.translation();
            assert!((pos.x - 10.0).abs() < 0.0001);
        }

        #[test]
        fn test_three_level_hierarchy() {
            // Grandparent at (10, 0, 0)
            let grandparent = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));

            // Parent at local (5, 0, 0)
            let parent_local = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
            let parent_global = grandparent.transform_by(&parent_local);

            // Child at local (3, 0, 0)
            let child_local = Transform::from_position(Vec3::new(3.0, 0.0, 0.0));
            let child_global = parent_global.transform_by(&child_local);

            // Child's global should be (18, 0, 0)
            let pos = child_global.translation();
            assert!((pos.x - 18.0).abs() < 0.0001);
        }
    }
}
