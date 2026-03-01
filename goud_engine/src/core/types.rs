//! # Core FFI-Safe Type Definitions
//!
//! This module defines FFI-safe types used throughout the engine for
//! cross-language interoperability. All types use `#[repr(C)]` for
//! predictable memory layout and primitive types for ABI stability.
//!
//! These types are the canonical definitions. The `ffi/` layer re-exports
//! them to preserve backward compatibility with generated bindings.

use crate::core::error::GoudErrorCode;
use crate::core::math::{Color, Rect, Vec2};
use crate::ecs::components::transform2d::Mat3x3;
use crate::ecs::components::Transform2D;

// =============================================================================
// Common Math Types
// =============================================================================

/// FFI-safe 2D vector representation.
///
/// This is the canonical FFI Vec2 type - all modules should use this
/// instead of defining their own to avoid duplicate definitions.
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

// =============================================================================
// Entity ID
// =============================================================================

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

// =============================================================================
// Result Type
// =============================================================================

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

// =============================================================================
// Contact Type
// =============================================================================

/// FFI-compatible contact information from a collision.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GoudContact {
    /// Contact point X coordinate
    pub point_x: f32,
    /// Contact point Y coordinate
    pub point_y: f32,
    /// Collision normal X component (unit vector pointing from A to B)
    pub normal_x: f32,
    /// Collision normal Y component
    pub normal_y: f32,
    /// Penetration depth (positive = overlapping)
    pub penetration: f32,
}

impl From<crate::ecs::collision::Contact> for GoudContact {
    fn from(contact: crate::ecs::collision::Contact) -> Self {
        Self {
            point_x: contact.point.x,
            point_y: contact.point.y,
            normal_x: contact.normal.x,
            normal_y: contact.normal.y,
            penetration: contact.penetration,
        }
    }
}

// =============================================================================
// Sprite Types
// =============================================================================

/// FFI-safe Sprite representation.
///
/// This is a simplified version of the Sprite component suitable for FFI.
/// It uses raw u64 texture handles instead of generic AssetHandle types.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiSprite {
    /// Texture handle (index and generation packed as u64).
    pub texture_handle: u64,

    /// Color tint red component (0.0 - 1.0).
    pub color_r: f32,
    /// Color tint green component (0.0 - 1.0).
    pub color_g: f32,
    /// Color tint blue component (0.0 - 1.0).
    pub color_b: f32,
    /// Color tint alpha component (0.0 - 1.0).
    pub color_a: f32,

    /// Source rectangle X position (if has_source_rect is true).
    pub source_rect_x: f32,
    /// Source rectangle Y position.
    pub source_rect_y: f32,
    /// Source rectangle width.
    pub source_rect_width: f32,
    /// Source rectangle height.
    pub source_rect_height: f32,
    /// Whether source_rect is set.
    pub has_source_rect: bool,

    /// Flip horizontally flag.
    pub flip_x: bool,
    /// Flip vertically flag.
    pub flip_y: bool,

    /// Anchor point X (normalized 0-1).
    pub anchor_x: f32,
    /// Anchor point Y (normalized 0-1).
    pub anchor_y: f32,

    /// Custom size width (if has_custom_size is true).
    pub custom_size_x: f32,
    /// Custom size height.
    pub custom_size_y: f32,
    /// Whether custom_size is set.
    pub has_custom_size: bool,
}

/// FFI-safe Color representation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiColor {
    /// Red component (0.0 - 1.0).
    pub r: f32,
    /// Green component (0.0 - 1.0).
    pub g: f32,
    /// Blue component (0.0 - 1.0).
    pub b: f32,
    /// Alpha component (0.0 - 1.0).
    pub a: f32,
}

impl From<Color> for FfiColor {
    fn from(c: Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

impl From<FfiColor> for Color {
    fn from(c: FfiColor) -> Self {
        Color::rgba(c.r, c.g, c.b, c.a)
    }
}

/// FFI-safe Rect representation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiRect {
    /// X position of the rectangle.
    pub x: f32,
    /// Y position of the rectangle.
    pub y: f32,
    /// Width of the rectangle.
    pub width: f32,
    /// Height of the rectangle.
    pub height: f32,
}

impl From<Rect> for FfiRect {
    fn from(r: Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

impl From<FfiRect> for Rect {
    fn from(r: FfiRect) -> Self {
        Rect::new(r.x, r.y, r.width, r.height)
    }
}

/// Heap-allocated sprite builder for FFI use.
///
/// This builder allows constructing a sprite by setting properties one at
/// a time without copying the entire struct on each modification.
#[repr(C)]
pub struct FfiSpriteBuilder {
    /// The sprite being built.
    pub sprite: FfiSprite,
}

// =============================================================================
// Transform2D Types
// =============================================================================

/// FFI-safe Transform2D representation.
///
/// This matches the memory layout of `Transform2D` exactly and is used
/// for passing transforms across FFI boundaries.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiTransform2D {
    /// Position X in world space.
    pub position_x: f32,
    /// Position Y in world space.
    pub position_y: f32,
    /// Rotation angle in radians.
    pub rotation: f32,
    /// Scale along X axis.
    pub scale_x: f32,
    /// Scale along Y axis.
    pub scale_y: f32,
}

impl From<Transform2D> for FfiTransform2D {
    fn from(t: Transform2D) -> Self {
        Self {
            position_x: t.position.x,
            position_y: t.position.y,
            rotation: t.rotation,
            scale_x: t.scale.x,
            scale_y: t.scale.y,
        }
    }
}

impl From<FfiTransform2D> for Transform2D {
    fn from(t: FfiTransform2D) -> Self {
        Self {
            position: Vec2::new(t.position_x, t.position_y),
            rotation: t.rotation,
            scale: Vec2::new(t.scale_x, t.scale_y),
        }
    }
}

/// FFI-safe Mat3x3 representation (9 floats in column-major order).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FfiMat3x3 {
    /// Matrix elements in column-major order.
    pub m: [f32; 9],
}

impl From<Mat3x3> for FfiMat3x3 {
    fn from(m: Mat3x3) -> Self {
        Self { m: m.m }
    }
}

/// Heap-allocated transform builder for FFI use.
///
/// This builder allows constructing a transform by setting properties one
/// at a time without copying the entire struct on each modification.
#[repr(C)]
pub struct FfiTransform2DBuilder {
    /// The transform being built.
    pub transform: FfiTransform2D,
}

// =============================================================================
// Tests
// =============================================================================

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
