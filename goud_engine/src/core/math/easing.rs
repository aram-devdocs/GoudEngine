//! Easing functions for smooth animations and transitions.
//!
//! Provides common easing curves (linear, quadratic, back, bounce) plus
//! a cubic Bezier evaluator for custom curves.

/// Type alias for easing functions: maps normalized time `t` in [0, 1] to a curved value.
pub type EasingFn = fn(f32) -> f32;

/// Linear easing (identity).
#[inline]
pub fn linear(t: f32) -> f32 {
    t
}

/// Quadratic ease-in (slow start).
#[inline]
pub fn ease_in(t: f32) -> f32 {
    t * t
}

/// Quadratic ease-out (slow end).
#[inline]
pub fn ease_out(t: f32) -> f32 {
    t * (2.0 - t)
}

/// Quadratic ease-in-out (slow start and end).
#[inline]
pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

/// Ease-in with overshoot (back easing).
///
/// The curve dips below zero before accelerating to the target.
#[inline]
pub fn ease_in_back(t: f32) -> f32 {
    const S: f32 = 1.70158;
    t * t * ((S + 1.0) * t - S)
}

/// Bounce ease-out.
///
/// Simulates a bouncing effect at the end of the transition.
#[inline]
pub fn ease_out_bounce(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

/// Cubic Bezier easing with two control points.
///
/// The curve starts at (0, 0) and ends at (1, 1). The control points
/// `(x1, y1)` and `(x2, y2)` shape the curve between those endpoints.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BezierEasing {
    /// X coordinate of the first control point.
    pub x1: f32,
    /// Y coordinate of the first control point.
    pub y1: f32,
    /// X coordinate of the second control point.
    pub x2: f32,
    /// Y coordinate of the second control point.
    pub y2: f32,
}

impl BezierEasing {
    /// Creates a new cubic Bezier easing curve.
    #[inline]
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Evaluates the easing curve at time `t` in [0, 1].
    ///
    /// Uses Newton's method to find the parameter on the Bezier curve
    /// whose X coordinate matches `t`, then returns the corresponding Y.
    pub fn evaluate(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }

        // Find the Bezier parameter `u` such that bezier_x(u) == t
        // using Newton's method with binary search fallback.
        let mut u = t; // initial guess
        for _ in 0..8 {
            let x = self.sample_x(u) - t;
            if x.abs() < 1e-6 {
                return self.sample_y(u);
            }
            let dx = self.sample_dx(u);
            if dx.abs() < 1e-6 {
                break;
            }
            u -= x / dx;
        }

        // Binary search fallback
        let mut lo = 0.0_f32;
        let mut hi = 1.0_f32;
        u = t;
        for _ in 0..20 {
            let x = self.sample_x(u);
            if (x - t).abs() < 1e-6 {
                return self.sample_y(u);
            }
            if x < t {
                lo = u;
            } else {
                hi = u;
            }
            u = (lo + hi) * 0.5;
        }

        self.sample_y(u)
    }

    /// Sample the X coordinate of the cubic Bezier at parameter `u`.
    #[inline]
    fn sample_x(&self, u: f32) -> f32 {
        // B(u) = 3(1-u)^2 u P1 + 3(1-u) u^2 P2 + u^3
        let u1 = 1.0 - u;
        3.0 * u1 * u1 * u * self.x1 + 3.0 * u1 * u * u * self.x2 + u * u * u
    }

    /// Sample the Y coordinate of the cubic Bezier at parameter `u`.
    #[inline]
    fn sample_y(&self, u: f32) -> f32 {
        let u1 = 1.0 - u;
        3.0 * u1 * u1 * u * self.y1 + 3.0 * u1 * u * u * self.y2 + u * u * u
    }

    /// Derivative of the X coordinate with respect to `u`.
    #[inline]
    fn sample_dx(&self, u: f32) -> f32 {
        let u1 = 1.0 - u;
        3.0 * u1 * u1 * self.x1 + 6.0 * u1 * u * (self.x2 - self.x1) + 3.0 * u * u * (1.0 - self.x2)
    }
}

/// Enumeration of built-in easing types plus custom cubic Bezier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    /// Linear interpolation (no easing).
    Linear,
    /// Quadratic ease-in.
    EaseIn,
    /// Quadratic ease-out.
    EaseOut,
    /// Quadratic ease-in-out.
    EaseInOut,
    /// Back ease-in (overshoot).
    EaseInBack,
    /// Bounce ease-out.
    EaseOutBounce,
    /// Custom cubic Bezier curve.
    CubicBezier(BezierEasing),
}

impl Easing {
    /// Applies this easing function to `t`.
    #[inline]
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => linear(t),
            Easing::EaseIn => ease_in(t),
            Easing::EaseOut => ease_out(t),
            Easing::EaseInOut => ease_in_out(t),
            Easing::EaseInBack => ease_in_back(t),
            Easing::EaseOutBounce => ease_out_bounce(t),
            Easing::CubicBezier(bezier) => bezier.evaluate(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn assert_boundaries(f: EasingFn, name: &str) {
        assert!(
            (f(0.0)).abs() < EPSILON,
            "{name}: f(0) should be 0, got {}",
            f(0.0)
        );
        assert!(
            (f(1.0) - 1.0).abs() < EPSILON,
            "{name}: f(1) should be 1, got {}",
            f(1.0)
        );
    }

    #[test]
    fn test_linear_boundaries() {
        assert_boundaries(linear, "linear");
    }

    #[test]
    fn test_linear_midpoint() {
        assert!((linear(0.5) - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_ease_in_boundaries() {
        assert_boundaries(ease_in, "ease_in");
    }

    #[test]
    fn test_ease_in_is_slow_start() {
        // At t=0.5, quadratic ease_in gives 0.25 (below linear 0.5)
        assert!((ease_in(0.5) - 0.25).abs() < EPSILON);
    }

    #[test]
    fn test_ease_out_boundaries() {
        assert_boundaries(ease_out, "ease_out");
    }

    #[test]
    fn test_ease_out_is_slow_end() {
        // At t=0.5, ease_out gives 0.75 (above linear 0.5)
        assert!((ease_out(0.5) - 0.75).abs() < EPSILON);
    }

    #[test]
    fn test_ease_in_out_boundaries() {
        assert_boundaries(ease_in_out, "ease_in_out");
    }

    #[test]
    fn test_ease_in_out_midpoint() {
        assert!((ease_in_out(0.5) - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_ease_in_back_boundaries() {
        assert_boundaries(ease_in_back, "ease_in_back");
    }

    #[test]
    fn test_ease_in_back_overshoots_negative() {
        // ease_in_back dips below zero near the start
        assert!(ease_in_back(0.25) < 0.0);
    }

    #[test]
    fn test_ease_out_bounce_boundaries() {
        assert_boundaries(ease_out_bounce, "ease_out_bounce");
    }

    #[test]
    fn test_ease_out_bounce_midpoint_above_half() {
        // Bounce ease-out reaches above 0.5 before t=0.5
        assert!(ease_out_bounce(0.5) > 0.5);
    }

    #[test]
    fn test_bezier_linear() {
        // Control points on the diagonal produce linear easing
        let bezier = BezierEasing::new(0.0, 0.0, 1.0, 1.0);
        assert!((bezier.evaluate(0.0)).abs() < EPSILON);
        assert!((bezier.evaluate(1.0) - 1.0).abs() < EPSILON);
        assert!((bezier.evaluate(0.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_bezier_css_ease() {
        // CSS "ease" curve: cubic-bezier(0.25, 0.1, 0.25, 1.0)
        let bezier = BezierEasing::new(0.25, 0.1, 0.25, 1.0);
        assert!((bezier.evaluate(0.0)).abs() < EPSILON);
        assert!((bezier.evaluate(1.0) - 1.0).abs() < EPSILON);
        // Midpoint should be above 0.5 for this curve
        let mid = bezier.evaluate(0.5);
        assert!(mid > 0.5, "CSS ease at 0.5 should be > 0.5, got {mid}");
    }

    #[test]
    fn test_easing_enum_delegates() {
        let easings = [
            (Easing::Linear, linear as EasingFn),
            (Easing::EaseIn, ease_in as EasingFn),
            (Easing::EaseOut, ease_out as EasingFn),
            (Easing::EaseInOut, ease_in_out as EasingFn),
            (Easing::EaseInBack, ease_in_back as EasingFn),
            (Easing::EaseOutBounce, ease_out_bounce as EasingFn),
        ];
        for (easing, func) in &easings {
            for &t in &[0.0, 0.25, 0.5, 0.75, 1.0] {
                assert!(
                    (easing.apply(t) - func(t)).abs() < EPSILON,
                    "Easing enum mismatch at t={t}"
                );
            }
        }
    }

    #[test]
    fn test_easing_enum_cubic_bezier() {
        let bezier = BezierEasing::new(0.42, 0.0, 0.58, 1.0);
        let easing = Easing::CubicBezier(bezier);
        assert!((easing.apply(0.0)).abs() < EPSILON);
        assert!((easing.apply(1.0) - 1.0).abs() < EPSILON);
    }
}
