//! # FFI Type Definitions
//!
//! This module defines FFI-safe types for cross-language interoperability.
//! All types use `#[repr(C)]` for predictable memory layout and primitive
//! types for ABI stability.

use crate::core::error::GoudErrorCode;
use crate::core::math::Vec2;

// =============================================================================
// Common Math Types
// =============================================================================

/// FFI-safe 2D vector representation.
///
/// This is the canonical FFI Vec2 type - all FFI modules should use this
/// instead of defining their own to avoid duplicate definitions in generated bindings.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiVec2 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
}

impl FfiVec2 {
    /// Creates a new FfiVec2.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero vector.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// One vector.
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
}

impl From<Vec2> for FfiVec2 {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<FfiVec2> for Vec2 {
    fn from(v: FfiVec2) -> Self {
        Vec2::new(v.x, v.y)
    }
}

impl Default for FfiVec2 {
    fn default() -> Self {
        Self::ZERO
    }
}

/// FFI-safe entity identifier.
///
/// This is a raw u64 that packs entity index and generation.
/// It's a direct representation of `Entity::to_bits()`.
///
/// # FFI Safety
///
/// - `#[repr(transparent)]` ensures same layout as u64
/// - Can be passed by value on all platforms
/// - u64::MAX is the INVALID sentinel value
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GoudEntityId(pub u64);

impl GoudEntityId {
    /// Sentinel value for an invalid entity.
    pub const INVALID: Self = Self(u64::MAX);

    /// Creates a new entity ID from a u64 bit pattern.
    pub fn new(bits: u64) -> Self {
        Self(bits)
    }

    /// Returns the underlying u64 bit pattern.
    pub fn bits(self) -> u64 {
        self.0
    }

    /// Returns true if this is the invalid sentinel.
    pub fn is_invalid(self) -> bool {
        self.0 == u64::MAX
    }
}

impl Default for GoudEntityId {
    fn default() -> Self {
        Self::INVALID
    }
}

impl From<u64> for GoudEntityId {
    fn from(bits: u64) -> Self {
        Self(bits)
    }
}

impl From<GoudEntityId> for u64 {
    fn from(id: GoudEntityId) -> u64 {
        id.0
    }
}

impl std::fmt::Display for GoudEntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_invalid() {
            write!(f, "GoudEntityId(INVALID)")
        } else {
            write!(f, "GoudEntityId({})", self.0)
        }
    }
}

/// FFI-safe result type for operations that can fail.
///
/// This is returned by FFI functions instead of throwing exceptions.
/// Callers must check the error code and retrieve error details via
/// `goud_get_last_error_message()` if needed.
///
/// # FFI Safety
///
/// - `#[repr(C)]` ensures predictable field layout
/// - Uses primitive types only (i32, bool)
/// - Always 8 bytes on all platforms (i32 + bool + padding)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoudResult {
    /// Error code (0 = success, non-zero = error).
    pub code: i32,

    /// True if operation succeeded (code == 0).
    pub success: bool,
}

impl GoudResult {
    /// Creates a success result.
    pub fn ok() -> Self {
        Self {
            code: 0,
            success: true,
        }
    }

    /// Creates an error result with the given code.
    pub fn err(code: GoudErrorCode) -> Self {
        Self {
            code,
            success: false,
        }
    }

    /// Returns true if this result is success.
    pub fn is_ok(&self) -> bool {
        self.success
    }

    /// Returns true if this result is an error.
    pub fn is_err(&self) -> bool {
        !self.success
    }
}

impl Default for GoudResult {
    fn default() -> Self {
        Self::ok()
    }
}

impl From<GoudErrorCode> for GoudResult {
    fn from(code: GoudErrorCode) -> Self {
        if code == 0 {
            Self::ok()
        } else {
            Self::err(code)
        }
    }
}

impl From<Result<(), crate::core::error::GoudError>> for GoudResult {
    fn from(result: Result<(), crate::core::error::GoudError>) -> Self {
        match result {
            Ok(()) => Self::ok(),
            Err(err) => Self::err(err.error_code()),
        }
    }
}

impl std::fmt::Display for GoudResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(f, "GoudResult::Success")
        } else {
            write!(f, "GoudResult::Error(code={})", self.code)
        }
    }
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
        assert_eq!(format!("{id}"), "GoudEntityId(100)");

        let invalid = GoudEntityId::INVALID;
        assert_eq!(format!("{invalid}"), "GoudEntityId(INVALID)");
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
        assert_eq!(format!("{ok}"), "GoudResult::Success");

        let err = GoudResult::err(404);
        assert_eq!(format!("{err}"), "GoudResult::Error(code=404)");
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
        assert!(size <= 8, "GoudResult is {size} bytes, expected <= 8");
    }

    #[test]
    fn test_result_repr_c() {
        // Verify #[repr(C)] layout is stable
        let result = GoudResult::err(42);
        let ptr = &result as *const GoudResult as *const u8;

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
