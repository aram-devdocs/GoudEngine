//! # FFI Type Definitions
//!
//! This module re-exports FFI-safe types from `core::types` for backward
//! compatibility. The canonical definitions live in `core::types`.

pub use crate::core::types::{FfiVec2, GoudEntityId, GoudResult};

/// FFI-safe per-frame render metrics.
///
/// Defined here (not in `renderer/`) so it is available on all platforms
/// including non-native builds where csbindgen needs the struct.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FfiRenderMetrics {
    /// Total draw calls across all render subsystems.
    pub draw_call_count: u32,
    /// Total sprites submitted before culling.
    pub sprites_submitted: u32,
    /// Sprites that passed culling and were drawn.
    pub sprites_drawn: u32,
    /// Sprites rejected by frustum culling.
    pub sprites_culled: u32,
    /// Number of sprite batches submitted.
    pub batches_submitted: u32,
    /// Average sprites per batch (batch efficiency).
    pub avg_sprites_per_batch: f32,
    /// Time spent rendering sprites (ms).
    pub sprite_render_ms: f32,
    /// Time spent rendering text (ms).
    pub text_render_ms: f32,
    /// Time spent rendering UI (ms).
    pub ui_render_ms: f32,
    /// Total render phase time (ms).
    pub total_render_ms: f32,
    /// Draw calls from text rendering.
    pub text_draw_calls: u32,
    /// Glyphs rendered this frame.
    pub text_glyph_count: u32,
    /// Draw calls from UI rendering.
    pub ui_draw_calls: u32,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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

        // SAFETY: Verifying repr(C) layout from a known-valid reference.
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
}
