//! Unit tests for [`GlobalTransform2D`].

use crate::core::math::Vec2;
use crate::ecs::components::global_transform2d::operations::lerp_angle as lerp_angle_fn;
use crate::ecs::components::global_transform2d::GlobalTransform2D;
use crate::ecs::components::transform2d::Transform2D;
use crate::ecs::Component;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

mod construction_tests {
    use super::*;

    #[test]
    fn test_identity() {
        let global = GlobalTransform2D::IDENTITY;
        let pos = global.translation();
        let scale = global.scale();

        assert!((pos.x).abs() < 0.0001);
        assert!((pos.y).abs() < 0.0001);
        assert!((scale.x - 1.0).abs() < 0.0001);
        assert!((scale.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_default() {
        let global: GlobalTransform2D = Default::default();
        assert_eq!(global, GlobalTransform2D::IDENTITY);
    }

    #[test]
    fn test_from_translation() {
        let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 200.0));
        let pos = global.translation();

        assert!((pos.x - 100.0).abs() < 0.0001);
        assert!((pos.y - 200.0).abs() < 0.0001);
    }

    #[test]
    fn test_from_translation_rotation_scale() {
        let global = GlobalTransform2D::from_translation_rotation_scale(
            Vec2::new(50.0, 100.0),
            FRAC_PI_4,
            Vec2::new(2.0, 3.0),
        );

        let pos = global.translation();
        let scale = global.scale();

        assert!((pos.x - 50.0).abs() < 0.0001);
        assert!((pos.y - 100.0).abs() < 0.0001);
        assert!((scale.x - 2.0).abs() < 0.0001);
        assert!((scale.y - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_from_transform() {
        let transform = Transform2D::new(Vec2::new(10.0, 20.0), FRAC_PI_4, Vec2::new(2.0, 2.0));
        let global: GlobalTransform2D = transform.into();

        let pos = global.translation();
        assert!((pos.x - 10.0).abs() < 0.0001);
        assert!((pos.y - 20.0).abs() < 0.0001);
    }

    #[test]
    fn test_from_transform_ref() {
        let transform = Transform2D::from_position(Vec2::new(50.0, 0.0));
        let global: GlobalTransform2D = (&transform).into();
        let pos = global.translation();
        assert!((pos.x - 50.0).abs() < 0.0001);
    }
}

mod decomposition_tests {
    use super::*;

    #[test]
    fn test_translation_extraction() {
        let global = GlobalTransform2D::from_translation(Vec2::new(10.0, 20.0));
        let pos = global.translation();
        assert!((pos.x - 10.0).abs() < 0.0001);
        assert!((pos.y - 20.0).abs() < 0.0001);
    }

    #[test]
    fn test_scale_extraction() {
        let global = GlobalTransform2D::from_translation_rotation_scale(
            Vec2::zero(),
            0.0,
            Vec2::new(2.0, 3.0),
        );
        let scale = global.scale();
        assert!((scale.x - 2.0).abs() < 0.0001);
        assert!((scale.y - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_rotation_extraction() {
        let original = FRAC_PI_4;
        let global =
            GlobalTransform2D::from_translation_rotation_scale(Vec2::zero(), original, Vec2::one());
        let extracted = global.rotation();

        assert!((extracted - original).abs() < 0.001);
    }

    #[test]
    fn test_rotation_degrees() {
        let global = GlobalTransform2D::from_translation_rotation_scale(
            Vec2::zero(),
            FRAC_PI_2,
            Vec2::one(),
        );
        let degrees = global.rotation_degrees();
        assert!((degrees - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_decompose() {
        let original_t = Vec2::new(100.0, 50.0);
        let original_r = FRAC_PI_4;
        let original_s = Vec2::new(2.0, 3.0);

        let global =
            GlobalTransform2D::from_translation_rotation_scale(original_t, original_r, original_s);
        let (t, r, s) = global.decompose();

        assert!((t - original_t).length() < 0.001);
        assert!((r - original_r).abs() < 0.001);
        assert!((s - original_s).length() < 0.001);
    }

    #[test]
    fn test_to_transform() {
        let global = GlobalTransform2D::from_translation_rotation_scale(
            Vec2::new(50.0, 100.0),
            0.0,
            Vec2::new(2.0, 2.0),
        );

        let transform = global.to_transform();
        assert!((transform.position - Vec2::new(50.0, 100.0)).length() < 0.001);
        assert!((transform.scale - Vec2::new(2.0, 2.0)).length() < 0.001);
    }
}

mod transform_tests {
    use super::*;

    #[test]
    fn test_mul_transform_translation() {
        let a = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
        let b = GlobalTransform2D::from_translation(Vec2::new(50.0, 0.0));
        let result = a.mul_transform(&b);

        let pos = result.translation();
        assert!((pos.x - 150.0).abs() < 0.0001);
    }

    #[test]
    fn test_mul_transform_scale() {
        let a = GlobalTransform2D::from_translation_rotation_scale(
            Vec2::zero(),
            0.0,
            Vec2::new(2.0, 2.0),
        );
        let b = GlobalTransform2D::from_translation(Vec2::new(50.0, 0.0));
        let result = a.mul_transform(&b);

        let pos = result.translation();
        // Scale affects the child's translation
        assert!((pos.x - 100.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform_by() {
        let parent = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
        let child = Transform2D::from_position(Vec2::new(50.0, 0.0));
        let result = parent.transform_by(&child);

        let pos = result.translation();
        assert!((pos.x - 150.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform_point() {
        let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
        let local_point = Vec2::new(50.0, 30.0);
        let world_point = global.transform_point(local_point);

        assert!((world_point.x - 150.0).abs() < 0.0001);
        assert!((world_point.y - 30.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform_direction() {
        let global = GlobalTransform2D::from_translation(Vec2::new(1000.0, 0.0));
        let direction = Vec2::new(1.0, 0.0);
        let world_dir = global.transform_direction(direction);

        // Direction should not be affected by translation
        assert!((world_dir.x - 1.0).abs() < 0.0001);
        assert!(world_dir.y.abs() < 0.0001);
    }

    #[test]
    fn test_inverse() {
        let global = GlobalTransform2D::from_translation_rotation_scale(
            Vec2::new(50.0, 100.0),
            FRAC_PI_4,
            Vec2::new(2.0, 2.0),
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
        let a = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
        let b = GlobalTransform2D::from_translation(Vec2::new(50.0, 0.0));
        let result = a * b;

        let pos = result.translation();
        assert!((pos.x - 150.0).abs() < 0.0001);
    }

    #[test]
    fn test_mul_operator_with_transform() {
        let parent = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
        let child = Transform2D::from_position(Vec2::new(50.0, 0.0));
        let result = parent * child;

        let pos = result.translation();
        assert!((pos.x - 150.0).abs() < 0.0001);
    }
}

mod direction_tests {
    use super::*;

    #[test]
    fn test_directions_identity() {
        let global = GlobalTransform2D::IDENTITY;

        assert!((global.forward() - Vec2::new(0.0, 1.0)).length() < 0.0001);
        assert!((global.backward() - Vec2::new(0.0, -1.0)).length() < 0.0001);
        assert!((global.right() - Vec2::new(1.0, 0.0)).length() < 0.0001);
        assert!((global.left() - Vec2::new(-1.0, 0.0)).length() < 0.0001);
    }

    #[test]
    fn test_directions_rotated() {
        let global = GlobalTransform2D::from_translation_rotation_scale(
            Vec2::zero(),
            FRAC_PI_2, // 90 degrees
            Vec2::one(),
        );

        // After 90 degree rotation:
        // forward (0, 1) -> (-1, 0)
        let fwd = global.forward();
        assert!((fwd.x - (-1.0)).abs() < 0.0001);
        assert!(fwd.y.abs() < 0.0001);
    }
}

mod interpolation_tests {
    use super::*;

    #[test]
    fn test_lerp_translation() {
        let a = GlobalTransform2D::from_translation(Vec2::zero());
        let b = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));

        let mid = a.lerp(&b, 0.5);
        let pos = mid.translation();
        assert!((pos.x - 50.0).abs() < 0.0001);
    }

    #[test]
    fn test_lerp_endpoints() {
        let a = GlobalTransform2D::from_translation(Vec2::new(0.0, 0.0));
        let b = GlobalTransform2D::from_translation(Vec2::new(100.0, 100.0));

        let start = a.lerp(&b, 0.0);
        assert!((start.translation() - a.translation()).length() < 0.0001);

        let end = a.lerp(&b, 1.0);
        assert!((end.translation() - b.translation()).length() < 0.0001);
    }

    #[test]
    fn test_lerp_angle() {
        // Test shortest path angle interpolation
        let result = lerp_angle_fn(0.0, PI, 0.5);
        assert!((result - FRAC_PI_2).abs() < 0.0001);

        // Test wrapping around
        let result = lerp_angle_fn(0.1, -0.1, 0.5);
        assert!(result.abs() < 0.0001);
    }
}

mod array_tests {
    use super::*;

    #[test]
    fn test_to_cols_array() {
        let global = GlobalTransform2D::IDENTITY;
        let cols = global.to_cols_array();

        // Identity matrix
        assert_eq!(cols[0], 1.0); // m00
        assert_eq!(cols[4], 1.0); // m11
        assert_eq!(cols[8], 1.0); // m22
    }

    #[test]
    fn test_to_mat4_array() {
        let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 200.0));
        let mat4 = global.to_mat4_array();

        // Translation is in column 4
        assert!((mat4[12] - 100.0).abs() < 0.0001);
        assert!((mat4[13] - 200.0).abs() < 0.0001);
        // Z row/column should be identity-like
        assert_eq!(mat4[10], 1.0);
        assert_eq!(mat4[15], 1.0);
    }
}

mod component_tests {
    use super::*;

    #[test]
    fn test_is_component() {
        fn assert_component<T: Component>() {}
        assert_component::<GlobalTransform2D>();
    }

    #[test]
    fn test_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<GlobalTransform2D>();
    }

    #[test]
    fn test_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<GlobalTransform2D>();
    }

    #[test]
    fn test_clone() {
        let global = GlobalTransform2D::from_translation(Vec2::new(10.0, 20.0));
        let cloned = global.clone();
        assert_eq!(global, cloned);
    }

    #[test]
    fn test_copy() {
        let global = GlobalTransform2D::IDENTITY;
        let copied = global;
        assert_eq!(global, copied);
    }

    #[test]
    fn test_debug() {
        let global = GlobalTransform2D::from_translation(Vec2::new(100.0, 50.0));
        let debug = format!("{:?}", global);
        assert!(debug.contains("GlobalTransform2D"));
        assert!(debug.contains("translation"));
    }
}

mod hierarchy_tests {
    use super::*;

    #[test]
    fn test_parent_child_translation() {
        // Parent at (100, 0)
        let parent_global = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));

        // Child at local (50, 0)
        let child_local = Transform2D::from_position(Vec2::new(50.0, 0.0));

        // Child's global should be (150, 0)
        let child_global = parent_global.transform_by(&child_local);
        let pos = child_global.translation();
        assert!((pos.x - 150.0).abs() < 0.0001);
    }

    #[test]
    fn test_parent_child_rotation() {
        // Parent rotated 90 degrees
        let parent_global = GlobalTransform2D::from_translation_rotation_scale(
            Vec2::zero(),
            FRAC_PI_2,
            Vec2::one(),
        );

        // Child at local (0, 100) - above parent in local space
        let child_local = Transform2D::from_position(Vec2::new(0.0, 100.0));

        // After parent rotation, child should be at (-100, 0)
        let child_global = parent_global.transform_by(&child_local);
        let pos = child_global.translation();
        assert!((pos.x - (-100.0)).abs() < 0.01);
        assert!(pos.y.abs() < 0.01);
    }

    #[test]
    fn test_parent_child_scale() {
        // Parent scaled 2x
        let parent_global = GlobalTransform2D::from_translation_rotation_scale(
            Vec2::zero(),
            0.0,
            Vec2::new(2.0, 2.0),
        );

        // Child at local (50, 0)
        let child_local = Transform2D::from_position(Vec2::new(50.0, 0.0));

        // Child's global position should be (100, 0)
        let child_global = parent_global.transform_by(&child_local);
        let pos = child_global.translation();
        assert!((pos.x - 100.0).abs() < 0.0001);
    }

    #[test]
    fn test_three_level_hierarchy() {
        // Grandparent at (100, 0)
        let grandparent = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));

        // Parent at local (50, 0)
        let parent_local = Transform2D::from_position(Vec2::new(50.0, 0.0));
        let parent_global = grandparent.transform_by(&parent_local);

        // Child at local (30, 0)
        let child_local = Transform2D::from_position(Vec2::new(30.0, 0.0));
        let child_global = parent_global.transform_by(&child_local);

        // Child's global should be (180, 0)
        let pos = child_global.translation();
        assert!((pos.x - 180.0).abs() < 0.0001);
    }
}

mod ffi_tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn test_size() {
        // Mat3x3 is 9 * 4 = 36 bytes
        assert_eq!(size_of::<GlobalTransform2D>(), 36);
    }

    #[test]
    fn test_align() {
        assert_eq!(align_of::<GlobalTransform2D>(), 4);
    }
}
