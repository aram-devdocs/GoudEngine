//! Tests for core FFI-safe types.

#[cfg(test)]
mod tests {
    use crate::core::math::{Color, Rect, Vec2};
    use crate::core::types::{FfiColor, FfiRect, FfiTransform2D, GoudEntityId, GoudResult};
    use crate::ecs::components::Transform2D;

    // ========================================================================
    // GoudEntityId Tests
    // ========================================================================

    #[test]
    fn test_entity_id_new() {
        let id = GoudEntityId::new(42);
        assert_eq!(id.bits(), 42);
        assert!(!id.is_invalid());
    }

    #[test]
    fn test_entity_id_invalid() {
        let id = GoudEntityId::INVALID;
        assert!(id.is_invalid());
        assert_eq!(id.bits(), u64::MAX);
    }

    #[test]
    fn test_entity_id_default() {
        let id = GoudEntityId::default();
        assert!(id.is_invalid());
    }

    #[test]
    fn test_entity_id_from_u64() {
        let id: GoudEntityId = 123u64.into();
        assert_eq!(id.bits(), 123);
    }

    #[test]
    fn test_entity_id_to_u64() {
        let id = GoudEntityId::new(456);
        let bits: u64 = id.into();
        assert_eq!(bits, 456);
    }

    #[test]
    fn test_entity_id_display() {
        let id = GoudEntityId::new(100);
        assert_eq!(format!("{}", id), "GoudEntityId(100)");

        let invalid = GoudEntityId::INVALID;
        assert_eq!(format!("{}", invalid), "GoudEntityId(INVALID)");
    }

    #[test]
    fn test_entity_id_equality() {
        let id1 = GoudEntityId::new(10);
        let id2 = GoudEntityId::new(10);
        let id3 = GoudEntityId::new(20);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_entity_id_hash() {
        use std::collections::HashSet;

        let id1 = GoudEntityId::new(10);
        let id2 = GoudEntityId::new(10);
        let id3 = GoudEntityId::new(20);

        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&id2));
        assert!(!set.contains(&id3));
    }

    #[test]
    fn test_entity_id_copy_clone() {
        let id1 = GoudEntityId::new(5);
        let id2 = id1; // Copy
        let id3 = id1.clone(); // Clone

        assert_eq!(id1, id2);
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_entity_id_size() {
        // Must be exactly 8 bytes for FFI
        assert_eq!(std::mem::size_of::<GoudEntityId>(), 8);
        assert_eq!(std::mem::align_of::<GoudEntityId>(), 8);
    }

    // ========================================================================
    // GoudResult Tests
    // ========================================================================

    #[test]
    fn test_result_ok() {
        let result = GoudResult::ok();
        assert!(result.is_ok());
        assert!(!result.is_err());
        assert_eq!(result.code, 0);
        assert!(result.success);
    }

    #[test]
    fn test_result_err() {
        let result = GoudResult::err(100);
        assert!(!result.is_ok());
        assert!(result.is_err());
        assert_eq!(result.code, 100);
        assert!(!result.success);
    }

    #[test]
    fn test_result_default() {
        let result = GoudResult::default();
        assert!(result.is_ok());
    }

    #[test]
    fn test_result_from_error_code() {
        let result: GoudResult = 0.into();
        assert!(result.is_ok());

        let result: GoudResult = 200.into();
        assert!(result.is_err());
        assert_eq!(result.code, 200);
    }

    #[test]
    fn test_result_display() {
        let ok = GoudResult::ok();
        assert_eq!(format!("{}", ok), "GoudResult::Success");

        let err = GoudResult::err(404);
        assert_eq!(format!("{}", err), "GoudResult::Error(code=404)");
    }

    #[test]
    fn test_result_equality() {
        let ok1 = GoudResult::ok();
        let ok2 = GoudResult::ok();
        let err1 = GoudResult::err(100);
        let err2 = GoudResult::err(100);
        let err3 = GoudResult::err(200);

        assert_eq!(ok1, ok2);
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
        assert_ne!(ok1, err1);
    }

    #[test]
    fn test_result_copy_clone() {
        let result1 = GoudResult::err(42);
        let result2 = result1; // Copy
        let result3 = result1.clone(); // Clone

        assert_eq!(result1, result2);
        assert_eq!(result1, result3);
    }

    #[test]
    fn test_result_size() {
        // Should be 8 bytes (i32 + bool + padding)
        let size = std::mem::size_of::<GoudResult>();
        assert!(size <= 8, "GoudResult is {} bytes, expected <= 8", size);
    }

    #[test]
    fn test_result_repr_c() {
        // Verify #[repr(C)] layout is stable
        let result = GoudResult::err(42);
        let ptr = &result as *const GoudResult as *const u8;

        // SAFETY: Verifying that the #[repr(C)] layout matches expectations.
        // We read raw bytes from a known-valid reference.
        unsafe {
            // code field (first 4 bytes)
            let code_bytes = std::slice::from_raw_parts(ptr, 4);
            let code =
                i32::from_ne_bytes([code_bytes[0], code_bytes[1], code_bytes[2], code_bytes[3]]);
            assert_eq!(code, 42);

            // success field (next byte)
            let success = *ptr.add(4);
            assert_eq!(success, 0); // false
        }
    }

    // ========================================================================
    // Thread Safety Tests
    // ========================================================================

    #[test]
    fn test_entity_id_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<GoudEntityId>();
    }

    #[test]
    fn test_entity_id_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<GoudEntityId>();
    }

    #[test]
    fn test_result_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<GoudResult>();
    }

    #[test]
    fn test_result_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<GoudResult>();
    }

    // ========================================================================
    // FfiColor Tests
    // ========================================================================

    #[test]
    fn test_ffi_color_from_color() {
        let color = Color::rgba(0.5, 0.6, 0.7, 0.8);
        let ffi: FfiColor = color.into();
        assert_eq!(ffi.r, 0.5);
        assert_eq!(ffi.g, 0.6);
        assert_eq!(ffi.b, 0.7);
        assert_eq!(ffi.a, 0.8);
    }

    #[test]
    fn test_color_from_ffi_color() {
        let ffi = FfiColor {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 0.4,
        };
        let color: Color = ffi.into();
        assert_eq!(color.r, 0.1);
        assert_eq!(color.g, 0.2);
        assert_eq!(color.b, 0.3);
        assert_eq!(color.a, 0.4);
    }

    // ========================================================================
    // FfiRect Tests
    // ========================================================================

    #[test]
    fn test_ffi_rect_from_rect() {
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
        let ffi: FfiRect = rect.into();
        assert_eq!(ffi.x, 10.0);
        assert_eq!(ffi.y, 20.0);
        assert_eq!(ffi.width, 30.0);
        assert_eq!(ffi.height, 40.0);
    }

    #[test]
    fn test_rect_from_ffi_rect() {
        let ffi = FfiRect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        };
        let rect: Rect = ffi.into();
        assert_eq!(rect.x, 1.0);
        assert_eq!(rect.y, 2.0);
        assert_eq!(rect.width, 3.0);
        assert_eq!(rect.height, 4.0);
    }

    // ========================================================================
    // FfiTransform2D Tests
    // ========================================================================

    #[test]
    fn test_ffi_transform2d_roundtrip() {
        let t = Transform2D::new(Vec2::new(10.0, 20.0), 1.5, Vec2::new(2.0, 3.0));
        let ffi: FfiTransform2D = t.into();
        let roundtrip: Transform2D = ffi.into();
        assert_eq!(roundtrip.position.x, 10.0);
        assert_eq!(roundtrip.position.y, 20.0);
        assert_eq!(roundtrip.rotation, 1.5);
        assert_eq!(roundtrip.scale.x, 2.0);
        assert_eq!(roundtrip.scale.y, 3.0);
    }
}
