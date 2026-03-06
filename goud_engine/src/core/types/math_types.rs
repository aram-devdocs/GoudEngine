//! FFI-safe math types: vectors, colors, and rectangles.

use crate::core::math::{Color, Rect, Vec2};

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
// Color
// =============================================================================

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

// =============================================================================
// Rect
// =============================================================================

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
