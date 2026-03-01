//! # SDK Color Utilities
//!
//! Pure color utility functions exposed via `#[goud_api]` proc-macro.
//! These are stateless functions -- no game context or entity required.

use crate::core::math::Color;
use crate::core::types::FfiColor;

/// Zero-sized type that hosts color utility functions.
///
/// All methods are static (no `self` receiver) and are used by the
/// `#[goud_api]` proc-macro to auto-generate `#[no_mangle] extern "C"`
/// FFI wrappers.
pub struct ColorOps;

// NOTE: FFI wrappers are hand-written in ffi/component_sprite.rs. The
// `#[goud_api]` attribute is omitted to avoid duplicate symbol conflicts.
impl ColorOps {
    /// Returns the white color constant.
    pub fn white() -> FfiColor {
        FfiColor {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    /// Returns the black color constant.
    pub fn black() -> FfiColor {
        FfiColor {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    /// Returns the red color constant.
    pub fn red() -> FfiColor {
        FfiColor {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    /// Returns the green color constant.
    pub fn green() -> FfiColor {
        FfiColor {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }

    /// Returns the blue color constant.
    pub fn blue() -> FfiColor {
        FfiColor {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    /// Returns the yellow color constant.
    pub fn yellow() -> FfiColor {
        FfiColor {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }

    /// Returns the transparent color constant.
    pub fn transparent() -> FfiColor {
        FfiColor {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }

    /// Creates a color from RGBA components.
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> FfiColor {
        FfiColor { r, g, b, a }
    }

    /// Creates a color from RGB components with alpha = 1.0.
    pub fn rgb(r: f32, g: f32, b: f32) -> FfiColor {
        FfiColor { r, g, b, a: 1.0 }
    }

    /// Creates a color from 8-bit RGBA values (0-255).
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> FfiColor {
        FfiColor {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Creates a color from a hex value (0xRRGGBB or 0xRRGGBBAA).
    pub fn from_hex(hex: u32) -> FfiColor {
        Color::from_hex(hex).into()
    }

    /// Linearly interpolates between two colors.
    pub fn lerp(from: FfiColor, to: FfiColor, t: f32) -> FfiColor {
        let from_c: Color = from.into();
        let to_c: Color = to.into();
        from_c.lerp(to_c, t).into()
    }

    /// Returns a new color with the specified alpha.
    pub fn with_alpha(color: FfiColor, alpha: f32) -> FfiColor {
        FfiColor {
            r: color.r,
            g: color.g,
            b: color.b,
            a: alpha,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_white() {
        let c = ColorOps::white();
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 1.0);
        assert_eq!(c.b, 1.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_rgba() {
        let c = ColorOps::rgba(0.5, 0.6, 0.7, 0.8);
        assert_eq!(c.r, 0.5);
        assert_eq!(c.g, 0.6);
        assert_eq!(c.b, 0.7);
        assert_eq!(c.a, 0.8);
    }

    #[test]
    fn test_color_from_u8() {
        let c = ColorOps::from_u8(255, 0, 128, 255);
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.0);
        assert!((c.b - 128.0 / 255.0).abs() < 0.01);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_from_hex() {
        let c = ColorOps::from_hex(0xFF0000);
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_lerp() {
        let black = ColorOps::black();
        let white = ColorOps::white();
        let mid = ColorOps::lerp(black, white, 0.5);
        assert!((mid.r - 0.5).abs() < 0.001);
        assert!((mid.g - 0.5).abs() < 0.001);
        assert!((mid.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_color_with_alpha() {
        let c = ColorOps::red();
        let c2 = ColorOps::with_alpha(c, 0.5);
        assert_eq!(c2.r, 1.0);
        assert_eq!(c2.a, 0.5);
    }
}
