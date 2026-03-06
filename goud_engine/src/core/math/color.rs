//! RGBA color type with FFI-safe memory layout.

use super::vec3::Vec3;
use super::vec4::Vec4;

/// An RGBA color with FFI-safe memory layout.
///
/// Components are stored as f32 values, typically in the range [0.0, 1.0].
/// Values outside this range are allowed for HDR rendering.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    /// The red component of the color.
    pub r: f32,
    /// The green component of the color.
    pub g: f32,
    /// The blue component of the color.
    pub b: f32,
    /// The alpha (transparency) component of the color.
    pub a: f32,
}

impl Color {
    /// The color white (1.0, 1.0, 1.0, 1.0).
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    /// The color black (0.0, 0.0, 0.0, 1.0).
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    /// The color red (1.0, 0.0, 0.0, 1.0).
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    /// The color green (0.0, 1.0, 0.0, 1.0).
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    /// The color blue (0.0, 0.0, 1.0, 1.0).
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    /// The color yellow (1.0, 1.0, 0.0, 1.0).
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    /// The color cyan (0.0, 1.0, 1.0, 1.0).
    pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
    /// The color magenta (1.0, 0.0, 1.0, 1.0).
    pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
    /// Transparent black (0.0, 0.0, 0.0, 0.0).
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
    /// The color gray (0.5, 0.5, 0.5, 1.0).
    pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);

    /// Creates a new RGBA color.
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an RGB color with alpha = 1.0.
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Creates an RGBA color.
    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from 8-bit RGBA values (0-255).
    #[inline]
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Creates a color from a hex value (0xRRGGBB or 0xRRGGBBAA).
    #[inline]
    pub fn from_hex(hex: u32) -> Self {
        if hex > 0xFFFFFF {
            // Has alpha (0xRRGGBBAA)
            Self::from_u8(
                ((hex >> 24) & 0xFF) as u8,
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
            )
        } else {
            // No alpha (0xRRGGBB)
            Self::from_u8(
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
                255,
            )
        }
    }

    /// Returns the RGB components as a Vec3.
    #[inline]
    pub const fn to_vec3(&self) -> Vec3 {
        Vec3 {
            x: self.r,
            y: self.g,
            z: self.b,
        }
    }

    /// Returns all RGBA components as a Vec4.
    #[inline]
    pub const fn to_vec4(&self) -> Vec4 {
        Vec4 {
            x: self.r,
            y: self.g,
            z: self.b,
            w: self.a,
        }
    }

    /// Creates a color from a Vec3 (RGB) with alpha = 1.0.
    #[inline]
    pub const fn from_vec3(v: Vec3) -> Self {
        Self {
            r: v.x,
            g: v.y,
            b: v.z,
            a: 1.0,
        }
    }

    /// Creates a color from a Vec4 (RGBA).
    #[inline]
    pub const fn from_vec4(v: Vec4) -> Self {
        Self {
            r: v.x,
            g: v.y,
            b: v.z,
            a: v.w,
        }
    }

    /// Linearly interpolates between two colors.
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Returns a new color with the specified alpha.
    #[inline]
    pub const fn with_alpha(self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    /// Clamps all components to the [0.0, 1.0] range.
    #[inline]
    pub fn clamp(self) -> Self {
        Self {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
            a: self.a.clamp(0.0, 1.0),
        }
    }
}
