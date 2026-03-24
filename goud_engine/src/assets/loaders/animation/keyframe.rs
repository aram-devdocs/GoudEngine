//! Keyframe types and interpolation for property animation.
//!
//! The struct definitions and `interpolate` function now live in
//! `core::types::keyframe_types`.  This module re-exports them for backward
//! compatibility.

// Re-export all keyframe types from the canonical core location.
pub use crate::core::types::keyframe_types::{
    interpolate, AnimationChannel, EasingFunction, Keyframe,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn kf(time: f32, value: f32) -> Keyframe {
        Keyframe {
            time,
            value,
            easing: EasingFunction::Linear,
        }
    }

    fn kf_eased(time: f32, value: f32, easing: EasingFunction) -> Keyframe {
        Keyframe {
            time,
            value,
            easing,
        }
    }

    // ====================================================================
    // Empty / single keyframe
    // ====================================================================

    #[test]
    fn test_interpolate_empty_keyframes_returns_zero() {
        assert!((interpolate(&[], 0.5) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_interpolate_single_keyframe_returns_value() {
        let kfs = vec![kf(0.0, 42.0)];
        assert!((interpolate(&kfs, 0.0) - 42.0).abs() < f32::EPSILON);
        assert!((interpolate(&kfs, 1.0) - 42.0).abs() < f32::EPSILON);
        assert!((interpolate(&kfs, -1.0) - 42.0).abs() < f32::EPSILON);
    }

    // ====================================================================
    // Linear interpolation
    // ====================================================================

    #[test]
    fn test_interpolate_linear_midpoint() {
        let kfs = vec![kf(0.0, 0.0), kf(1.0, 10.0)];
        let v = interpolate(&kfs, 0.5);
        assert!((v - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_linear_at_start() {
        let kfs = vec![kf(0.0, 0.0), kf(1.0, 10.0)];
        assert!((interpolate(&kfs, 0.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_interpolate_linear_at_end() {
        let kfs = vec![kf(0.0, 0.0), kf(1.0, 10.0)];
        assert!((interpolate(&kfs, 1.0) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_interpolate_linear_before_start_clamps() {
        let kfs = vec![kf(1.0, 5.0), kf(2.0, 15.0)];
        assert!((interpolate(&kfs, 0.0) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_interpolate_linear_after_end_clamps() {
        let kfs = vec![kf(0.0, 5.0), kf(1.0, 15.0)];
        assert!((interpolate(&kfs, 2.0) - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_interpolate_linear_multiple_segments() {
        let kfs = vec![kf(0.0, 0.0), kf(1.0, 10.0), kf(2.0, 0.0)];
        assert!((interpolate(&kfs, 0.5) - 5.0).abs() < 0.001);
        assert!((interpolate(&kfs, 1.5) - 5.0).abs() < 0.001);
    }

    // ====================================================================
    // Ease-in
    // ====================================================================

    #[test]
    fn test_interpolate_ease_in_slower_at_start() {
        let kfs = vec![kf_eased(0.0, 0.0, EasingFunction::EaseIn), kf(1.0, 10.0)];
        let v = interpolate(&kfs, 0.5);
        // EaseIn: t^2 at 0.5 => 0.25, value = 2.5
        assert!((v - 2.5).abs() < 0.001);
    }

    // ====================================================================
    // Ease-out
    // ====================================================================

    #[test]
    fn test_interpolate_ease_out_faster_at_start() {
        let kfs = vec![kf_eased(0.0, 0.0, EasingFunction::EaseOut), kf(1.0, 10.0)];
        let v = interpolate(&kfs, 0.5);
        // EaseOut: t*(2-t) at 0.5 => 0.75, value = 7.5
        assert!((v - 7.5).abs() < 0.001);
    }

    // ====================================================================
    // Ease-in-out
    // ====================================================================

    #[test]
    fn test_interpolate_ease_in_out_midpoint() {
        let kfs = vec![kf_eased(0.0, 0.0, EasingFunction::EaseInOut), kf(1.0, 10.0)];
        let v = interpolate(&kfs, 0.5);
        // EaseInOut at t=0.5: 2*0.5*0.5 = 0.5, value = 5.0
        assert!((v - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_ease_in_out_quarter() {
        let kfs = vec![kf_eased(0.0, 0.0, EasingFunction::EaseInOut), kf(1.0, 10.0)];
        let v = interpolate(&kfs, 0.25);
        // EaseInOut first half: 2*0.25*0.25 = 0.125, value = 1.25
        assert!((v - 1.25).abs() < 0.001);
    }

    // ====================================================================
    // Step
    // ====================================================================

    #[test]
    fn test_interpolate_step_holds_from_value() {
        let kfs = vec![kf_eased(0.0, 0.0, EasingFunction::Step), kf(1.0, 10.0)];
        // Step should hold from.value until reaching to.time
        assert!((interpolate(&kfs, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((interpolate(&kfs, 0.5) - 0.0).abs() < f32::EPSILON);
        assert!((interpolate(&kfs, 0.99) - 0.0).abs() < f32::EPSILON);
        assert!((interpolate(&kfs, 1.0) - 10.0).abs() < f32::EPSILON);
    }

    // ====================================================================
    // CubicBezier
    // ====================================================================

    #[test]
    fn test_interpolate_cubic_bezier_boundaries() {
        let kfs = vec![
            kf_eased(
                0.0,
                0.0,
                EasingFunction::CubicBezier {
                    points: [0.25, 0.1, 0.25, 1.0],
                },
            ),
            kf(1.0, 10.0),
        ];
        // At t=0 and t=1, bezier curve equals start/end
        assert!((interpolate(&kfs, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((interpolate(&kfs, 1.0) - 10.0).abs() < f32::EPSILON);
    }

    // ====================================================================
    // Easing defaults and serde
    // ====================================================================

    #[test]
    fn test_easing_function_default_is_linear() {
        assert_eq!(EasingFunction::default(), EasingFunction::Linear);
    }

    #[test]
    fn test_keyframe_serde_roundtrip() {
        let kf = Keyframe {
            time: 0.5,
            value: 3.14,
            easing: EasingFunction::EaseIn,
        };
        let json = serde_json::to_string(&kf).unwrap();
        let parsed: Keyframe = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, parsed);
    }

    #[test]
    fn test_animation_channel_serde_roundtrip() {
        let channel = AnimationChannel {
            target_property: "transform.position.x".to_string(),
            keyframes: vec![kf(0.0, 0.0), kf(1.0, 100.0)],
        };
        let json = serde_json::to_string(&channel).unwrap();
        let parsed: AnimationChannel = serde_json::from_str(&json).unwrap();
        assert_eq!(channel, parsed);
    }

    #[test]
    fn test_cubic_bezier_serde_roundtrip() {
        let kf = Keyframe {
            time: 0.0,
            value: 0.0,
            easing: EasingFunction::CubicBezier {
                points: [0.25, 0.1, 0.25, 1.0],
            },
        };
        let json = serde_json::to_string(&kf).unwrap();
        let parsed: Keyframe = serde_json::from_str(&json).unwrap();
        assert_eq!(kf, parsed);
    }
}
