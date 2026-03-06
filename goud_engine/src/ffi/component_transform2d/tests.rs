//! Tests for the Transform2D FFI functions.

#[cfg(test)]
mod tests {
    use crate::core::types::{FfiMat3x3, FfiTransform2D, FfiTransform2DBuilder};
    use crate::ffi::component_transform2d::builder::*;
    use crate::ffi::component_transform2d::direction::*;
    use crate::ffi::component_transform2d::factory::*;
    use crate::ffi::component_transform2d::matrix_ops::*;
    use crate::ffi::component_transform2d::position::*;
    use crate::ffi::component_transform2d::rotation::*;
    use crate::ffi::component_transform2d::scale::*;
    use crate::ffi::types::FfiVec2;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

    #[test]
    fn test_ffi_transform2d_default() {
        let t = goud_transform2d_default();
        assert_eq!(t.position_x, 0.0);
        assert_eq!(t.position_y, 0.0);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale_x, 1.0);
        assert_eq!(t.scale_y, 1.0);
    }

    #[test]
    fn test_ffi_transform2d_from_position() {
        let t = goud_transform2d_from_position(10.0, 20.0);
        assert_eq!(t.position_x, 10.0);
        assert_eq!(t.position_y, 20.0);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale_x, 1.0);
        assert_eq!(t.scale_y, 1.0);
    }

    #[test]
    fn test_ffi_transform2d_translate() {
        let mut t = goud_transform2d_from_position(10.0, 20.0);
        unsafe {
            goud_transform2d_translate(&mut t, 5.0, 10.0);
        }
        assert_eq!(t.position_x, 15.0);
        assert_eq!(t.position_y, 30.0);
    }

    #[test]
    fn test_ffi_transform2d_rotate() {
        let mut t = goud_transform2d_default();
        unsafe {
            goud_transform2d_rotate(&mut t, FRAC_PI_4);
        }
        assert!((t.rotation - FRAC_PI_4).abs() < 0.0001);
    }

    #[test]
    fn test_ffi_transform2d_forward() {
        let t = goud_transform2d_from_rotation(FRAC_PI_2);
        let forward = unsafe { goud_transform2d_forward(&t) };
        // 90 degree rotation: forward (1, 0) -> (0, 1)
        assert!(forward.x.abs() < 0.0001);
        assert!((forward.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_ffi_transform2d_lerp() {
        let a = goud_transform2d_from_position(0.0, 0.0);
        let b = goud_transform2d_from_position(10.0, 20.0);
        let mid = goud_transform2d_lerp(a, b, 0.5);
        assert_eq!(mid.position_x, 5.0);
        assert_eq!(mid.position_y, 10.0);
    }

    #[test]
    fn test_ffi_transform2d_null_safety() {
        // Test that null pointer functions don't crash
        unsafe {
            goud_transform2d_translate(std::ptr::null_mut(), 1.0, 2.0);
            goud_transform2d_rotate(std::ptr::null_mut(), 1.0);
            goud_transform2d_set_position(std::ptr::null_mut(), 1.0, 2.0);
            goud_transform2d_set_scale(std::ptr::null_mut(), 1.0, 2.0);

            let pos = goud_transform2d_get_position(std::ptr::null());
            assert_eq!(pos.x, 0.0);
            assert_eq!(pos.y, 0.0);

            let rot = goud_transform2d_get_rotation(std::ptr::null());
            assert_eq!(rot, 0.0);

            let fwd = goud_transform2d_forward(std::ptr::null());
            assert_eq!(fwd.x, 1.0);
            assert_eq!(fwd.y, 0.0);
        }
    }

    #[test]
    fn test_ffi_transform2d_size() {
        // Verify FFI type has expected size
        assert_eq!(std::mem::size_of::<FfiTransform2D>(), 20);
        assert_eq!(std::mem::size_of::<FfiVec2>(), 8);
        assert_eq!(std::mem::size_of::<FfiMat3x3>(), 36);
    }

    // =========================================================================
    // Builder Pattern Tests
    // =========================================================================

    #[test]
    fn test_builder_new_and_build() {
        let builder = goud_transform2d_builder_new();
        assert!(!builder.is_null());

        let transform = unsafe { goud_transform2d_builder_build(builder) };
        assert_eq!(transform.position_x, 0.0);
        assert_eq!(transform.position_y, 0.0);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale_x, 1.0);
        assert_eq!(transform.scale_y, 1.0);
    }

    #[test]
    fn test_builder_at_position() {
        let builder = goud_transform2d_builder_at_position(100.0, 50.0);
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.position_x, 100.0);
        assert_eq!(transform.position_y, 50.0);
    }

    #[test]
    fn test_builder_with_position() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_position(builder, 200.0, 150.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.position_x, 200.0);
        assert_eq!(transform.position_y, 150.0);
    }

    #[test]
    fn test_builder_with_rotation() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_rotation(builder, FRAC_PI_4) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert!((transform.rotation - FRAC_PI_4).abs() < 0.0001);
    }

    #[test]
    fn test_builder_with_rotation_degrees() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_rotation_degrees(builder, 90.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert!((transform.rotation - FRAC_PI_2).abs() < 0.0001);
    }

    #[test]
    fn test_builder_with_scale() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_scale(builder, 2.0, 3.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.scale_x, 2.0);
        assert_eq!(transform.scale_y, 3.0);
    }

    #[test]
    fn test_builder_with_scale_uniform() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_scale_uniform(builder, 5.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.scale_x, 5.0);
        assert_eq!(transform.scale_y, 5.0);
    }

    #[test]
    fn test_builder_looking_at() {
        let builder = goud_transform2d_builder_at_position(0.0, 0.0);
        let builder = unsafe { goud_transform2d_builder_looking_at(builder, 0.0, 10.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        // Looking straight up (0, 10) from origin should be 90 degrees
        assert!((transform.rotation - FRAC_PI_2).abs() < 0.0001);
    }

    #[test]
    fn test_builder_translate() {
        let builder = goud_transform2d_builder_at_position(10.0, 20.0);
        let builder = unsafe { goud_transform2d_builder_translate(builder, 5.0, 10.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.position_x, 15.0);
        assert_eq!(transform.position_y, 30.0);
    }

    #[test]
    fn test_builder_rotate() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_rotation(builder, FRAC_PI_4) };
        let builder = unsafe { goud_transform2d_builder_rotate(builder, FRAC_PI_4) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert!((transform.rotation - FRAC_PI_2).abs() < 0.0001);
    }

    #[test]
    fn test_builder_scale_by() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_scale(builder, 2.0, 3.0) };
        let builder = unsafe { goud_transform2d_builder_scale_by(builder, 2.0, 2.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.scale_x, 4.0);
        assert_eq!(transform.scale_y, 6.0);
    }

    #[test]
    fn test_builder_chain() {
        let builder = goud_transform2d_builder_new();
        let builder = unsafe { goud_transform2d_builder_with_position(builder, 100.0, 50.0) };
        let builder = unsafe { goud_transform2d_builder_with_rotation(builder, FRAC_PI_4) };
        let builder = unsafe { goud_transform2d_builder_with_scale(builder, 2.0, 2.0) };
        let builder = unsafe { goud_transform2d_builder_translate(builder, 10.0, 10.0) };
        let transform = unsafe { goud_transform2d_builder_build(builder) };

        assert_eq!(transform.position_x, 110.0);
        assert_eq!(transform.position_y, 60.0);
        assert!((transform.rotation - FRAC_PI_4).abs() < 0.0001);
        assert_eq!(transform.scale_x, 2.0);
        assert_eq!(transform.scale_y, 2.0);
    }

    #[test]
    fn test_builder_free() {
        let builder = goud_transform2d_builder_new();
        unsafe { goud_transform2d_builder_free(builder) };
        // Should not crash - just testing memory is freed properly
    }

    #[test]
    fn test_builder_null_safety() {
        // All builder functions should handle null safely
        unsafe {
            let null_builder: *mut FfiTransform2DBuilder = std::ptr::null_mut();

            assert!(goud_transform2d_builder_with_position(null_builder, 1.0, 2.0).is_null());
            assert!(goud_transform2d_builder_with_rotation(null_builder, 1.0).is_null());
            assert!(goud_transform2d_builder_with_rotation_degrees(null_builder, 90.0).is_null());
            assert!(goud_transform2d_builder_with_scale(null_builder, 2.0, 2.0).is_null());
            assert!(goud_transform2d_builder_with_scale_uniform(null_builder, 2.0).is_null());
            assert!(goud_transform2d_builder_looking_at(null_builder, 0.0, 10.0).is_null());
            assert!(goud_transform2d_builder_translate(null_builder, 1.0, 2.0).is_null());
            assert!(goud_transform2d_builder_rotate(null_builder, 1.0).is_null());
            assert!(goud_transform2d_builder_scale_by(null_builder, 2.0, 2.0).is_null());

            // Build with null should return default
            let transform = goud_transform2d_builder_build(null_builder);
            assert_eq!(transform.position_x, 0.0);
            assert_eq!(transform.position_y, 0.0);
            assert_eq!(transform.rotation, 0.0);

            // Free null should not crash
            goud_transform2d_builder_free(null_builder);
        }
    }
}
