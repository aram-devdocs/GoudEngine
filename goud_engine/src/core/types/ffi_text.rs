//! FFI-safe Text component type.

// =============================================================================
// Text Type
// =============================================================================

/// FFI-safe Text representation.
///
/// This is a simplified version of the Text component suitable for FFI.
/// It uses a raw u64 font handle instead of the generic `AssetHandle` type.
/// The text content string is NOT included here since it is managed
/// separately via the entity/component system.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiText {
    /// Font asset handle (index and generation packed as u64).
    pub font_handle: u64,
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color red component (0.0 - 1.0).
    pub color_r: f32,
    /// Text color green component (0.0 - 1.0).
    pub color_g: f32,
    /// Text color blue component (0.0 - 1.0).
    pub color_b: f32,
    /// Text color alpha component (0.0 - 1.0).
    pub color_a: f32,
    /// Horizontal text alignment (0 = Left, 1 = Center, 2 = Right).
    pub alignment: u8,
    /// Maximum width for word-wrapping (valid only when has_max_width is true).
    pub max_width: f32,
    /// Whether max_width is set.
    pub has_max_width: bool,
    /// Line spacing multiplier (1.0 = default spacing).
    pub line_spacing: f32,
}
