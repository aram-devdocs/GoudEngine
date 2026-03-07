//! Tween utilities for interpolating values with easing.
//!
//! The [`Tweenable`] trait provides a generic interface for types that can be
//! interpolated. Built-in implementations cover `f32`, [`Vec2`], [`Vec3`],
//! and [`Color`].

use super::color::Color;
use super::easing::EasingFn;
use super::vec2::Vec2;
use super::vec3::Vec3;

/// A type that can be interpolated between two values.
///
/// The default `tween` method applies an easing function to the parameter `t`
/// before delegating to `lerp`.
pub trait Tweenable: Sized {
    /// Linearly interpolates between `self` and `other`.
    ///
    /// When `t = 0.0`, returns `self`. When `t = 1.0`, returns `other`.
    fn lerp(self, other: Self, t: f32) -> Self;

    /// Interpolates between `self` and `other` using an easing function.
    ///
    /// The easing function is applied to `t` before interpolation,
    /// producing non-linear motion curves.
    #[inline]
    fn tween(self, other: Self, t: f32, easing: EasingFn) -> Self {
        self.lerp(other, easing(t))
    }
}

impl Tweenable for f32 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Tweenable for Vec2 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self.lerp(other, t)
    }
}

impl Tweenable for Vec3 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self.lerp(other, t)
    }
}

impl Tweenable for Color {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self.lerp(other, t)
    }
}

/// Convenience function to tween a single `f32` value with an easing function.
#[inline]
pub fn tween(from: f32, to: f32, t: f32, easing: EasingFn) -> f32 {
    from + (to - from) * easing(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::easing::{ease_in, linear};

    const EPSILON: f32 = 1e-5;

    // =====================================================================
    // f32 Tweenable
    // =====================================================================

    #[test]
    fn test_f32_lerp_boundaries() {
        let result_0 = Tweenable::lerp(0.0_f32, 10.0, 0.0);
        let result_1 = Tweenable::lerp(0.0_f32, 10.0, 1.0);
        assert!((result_0).abs() < EPSILON);
        assert!((result_1 - 10.0).abs() < EPSILON);
    }

    #[test]
    fn test_f32_lerp_midpoint() {
        let result = Tweenable::lerp(0.0_f32, 10.0, 0.5);
        assert!((result - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_f32_tween_linear() {
        let result = 0.0_f32.tween(10.0, 0.5, linear);
        assert!((result - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_f32_tween_ease_in() {
        // ease_in(0.5) = 0.25, so result = 0 + 10 * 0.25 = 2.5
        let result = 0.0_f32.tween(10.0, 0.5, ease_in);
        assert!((result - 2.5).abs() < EPSILON);
    }

    // =====================================================================
    // Vec2 Tweenable
    // =====================================================================

    #[test]
    fn test_vec2_lerp_boundaries() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 20.0);
        let r0 = Tweenable::lerp(a, b, 0.0);
        let r1 = Tweenable::lerp(a, b, 1.0);
        assert_eq!(r0, a);
        assert_eq!(r1, b);
    }

    #[test]
    fn test_vec2_tween_linear() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 20.0);
        let result = a.tween(b, 0.5, linear);
        assert!((result.x - 5.0).abs() < EPSILON);
        assert!((result.y - 10.0).abs() < EPSILON);
    }

    #[test]
    fn test_vec2_tween_ease_in() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 20.0);
        let result = a.tween(b, 0.5, ease_in);
        // ease_in(0.5) = 0.25
        assert!((result.x - 2.5).abs() < EPSILON);
        assert!((result.y - 5.0).abs() < EPSILON);
    }

    // =====================================================================
    // Vec3 Tweenable
    // =====================================================================

    #[test]
    fn test_vec3_lerp_boundaries() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(10.0, 20.0, 30.0);
        let r0 = Tweenable::lerp(a, b, 0.0);
        let r1 = Tweenable::lerp(a, b, 1.0);
        assert_eq!(r0, a);
        assert_eq!(r1, b);
    }

    #[test]
    fn test_vec3_tween_linear() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(10.0, 20.0, 30.0);
        let result = a.tween(b, 0.5, linear);
        assert!((result.x - 5.0).abs() < EPSILON);
        assert!((result.y - 10.0).abs() < EPSILON);
        assert!((result.z - 15.0).abs() < EPSILON);
    }

    #[test]
    fn test_vec3_tween_ease_in() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(10.0, 20.0, 30.0);
        let result = a.tween(b, 0.5, ease_in);
        assert!((result.x - 2.5).abs() < EPSILON);
        assert!((result.y - 5.0).abs() < EPSILON);
        assert!((result.z - 7.5).abs() < EPSILON);
    }

    // =====================================================================
    // Color Tweenable
    // =====================================================================

    #[test]
    fn test_color_lerp_boundaries() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let r0 = Tweenable::lerp(a, b, 0.0);
        let r1 = Tweenable::lerp(a, b, 1.0);
        assert_eq!(r0, a);
        assert_eq!(r1, b);
    }

    #[test]
    fn test_color_tween_linear() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let result = a.tween(b, 0.5, linear);
        assert!((result.r - 0.5).abs() < EPSILON);
        assert!((result.g - 0.5).abs() < EPSILON);
        assert!((result.b - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_color_tween_ease_in() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let result = a.tween(b, 0.5, ease_in);
        // ease_in(0.5) = 0.25
        assert!((result.r - 0.25).abs() < EPSILON);
        assert!((result.g - 0.25).abs() < EPSILON);
        assert!((result.b - 0.25).abs() < EPSILON);
    }

    // =====================================================================
    // Standalone tween function
    // =====================================================================

    #[test]
    fn test_tween_fn_linear() {
        let result = tween(0.0, 10.0, 0.5, linear);
        assert!((result - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_tween_fn_ease_in() {
        let result = tween(0.0, 10.0, 0.5, ease_in);
        assert!((result - 2.5).abs() < EPSILON);
    }

    #[test]
    fn test_tween_fn_with_offset() {
        let result = tween(5.0, 15.0, 0.5, linear);
        assert!((result - 10.0).abs() < EPSILON);
    }
}
