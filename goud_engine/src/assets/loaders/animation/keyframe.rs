//! Keyframe types and interpolation for property animation.

use serde::{Deserialize, Serialize};

/// Easing function applied between keyframes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EasingFunction {
    /// Constant speed interpolation.
    #[default]
    Linear,
    /// Slow start, accelerating.
    EaseIn,
    /// Fast start, decelerating.
    EaseOut,
    /// Slow start and end with acceleration in between.
    EaseInOut,
    /// Custom cubic bezier curve defined by four control points.
    CubicBezier {
        /// The four bezier control values `[x1, y1, x2, y2]`.
        points: [f32; 4],
    },
    /// Discrete step with no interpolation (jumps to next value).
    Step,
}

/// A single keyframe in an animation channel.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Time of this keyframe in seconds from animation start.
    pub time: f32,
    /// The value at this keyframe.
    pub value: f32,
    /// Easing function to use when interpolating toward the next keyframe.
    #[serde(default)]
    pub easing: EasingFunction,
}

/// An animation channel targeting a specific property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationChannel {
    /// Property path targeted by this channel (e.g. "transform.position.x").
    pub target_property: String,
    /// Keyframes sorted by time.
    pub keyframes: Vec<Keyframe>,
}

/// Applies the easing function to a linear parameter `t` in `[0, 1]`.
fn apply_easing(easing: &EasingFunction, t: f32) -> f32 {
    match easing {
        EasingFunction::Linear => t,
        EasingFunction::EaseIn => t * t,
        EasingFunction::EaseOut => t * (2.0 - t),
        EasingFunction::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
        EasingFunction::CubicBezier { points } => cubic_bezier_y(points, t),
        EasingFunction::Step => 0.0,
    }
}

/// Evaluates a cubic bezier curve's y value at parameter `t`.
///
/// The bezier is defined with implicit start (0,0) and end (1,1),
/// with control points `[x1, y1, x2, y2]`.
fn cubic_bezier_y(points: &[f32; 4], t: f32) -> f32 {
    let [_x1, y1, _x2, y2] = *points;
    // Simplified bezier: use t directly as the curve parameter
    // B(t) = 3(1-t)^2*t*y1 + 3(1-t)*t^2*y2 + t^3
    let mt = 1.0 - t;
    3.0 * mt * mt * t * y1 + 3.0 * mt * t * t * y2 + t * t * t
}

/// Interpolates between keyframes at the given time `t`.
///
/// Returns the interpolated value for the given time. If `t` is before the
/// first keyframe, the first keyframe's value is returned. If after the last,
/// the last value is returned. For empty keyframe slices, returns `0.0`.
///
/// # Arguments
/// * `keyframes` - Slice of keyframes, assumed sorted by time.
/// * `t` - Time in seconds to evaluate at.
pub fn interpolate(keyframes: &[Keyframe], t: f32) -> f32 {
    if keyframes.is_empty() {
        return 0.0;
    }

    // Before first keyframe
    if t <= keyframes[0].time {
        return keyframes[0].value;
    }

    // After last keyframe
    let last = &keyframes[keyframes.len() - 1];
    if t >= last.time {
        return last.value;
    }

    // Find the two surrounding keyframes
    for window in keyframes.windows(2) {
        let from = &window[0];
        let to = &window[1];

        if t >= from.time && t <= to.time {
            let duration = to.time - from.time;
            if duration <= f32::EPSILON {
                return to.value;
            }

            let local_t = (t - from.time) / duration;

            // Step easing jumps to `to.value` only at exactly `to.time`
            if matches!(from.easing, EasingFunction::Step) {
                return from.value;
            }

            let eased_t = apply_easing(&from.easing, local_t);
            return from.value + (to.value - from.value) * eased_t;
        }
    }

    // Fallback (should not reach here with sorted keyframes)
    last.value
}

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
        let kfs = vec![
            kf_eased(0.0, 0.0, EasingFunction::EaseIn),
            kf(1.0, 10.0),
        ];
        let v = interpolate(&kfs, 0.5);
        // EaseIn: t^2 at 0.5 => 0.25, value = 2.5
        assert!((v - 2.5).abs() < 0.001);
    }

    // ====================================================================
    // Ease-out
    // ====================================================================

    #[test]
    fn test_interpolate_ease_out_faster_at_start() {
        let kfs = vec![
            kf_eased(0.0, 0.0, EasingFunction::EaseOut),
            kf(1.0, 10.0),
        ];
        let v = interpolate(&kfs, 0.5);
        // EaseOut: t*(2-t) at 0.5 => 0.75, value = 7.5
        assert!((v - 7.5).abs() < 0.001);
    }

    // ====================================================================
    // Ease-in-out
    // ====================================================================

    #[test]
    fn test_interpolate_ease_in_out_midpoint() {
        let kfs = vec![
            kf_eased(0.0, 0.0, EasingFunction::EaseInOut),
            kf(1.0, 10.0),
        ];
        let v = interpolate(&kfs, 0.5);
        // EaseInOut at t=0.5: 2*0.5*0.5 = 0.5, value = 5.0
        assert!((v - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_ease_in_out_quarter() {
        let kfs = vec![
            kf_eased(0.0, 0.0, EasingFunction::EaseInOut),
            kf(1.0, 10.0),
        ];
        let v = interpolate(&kfs, 0.25);
        // EaseInOut first half: 2*0.25*0.25 = 0.125, value = 1.25
        assert!((v - 1.25).abs() < 0.001);
    }

    // ====================================================================
    // Step
    // ====================================================================

    #[test]
    fn test_interpolate_step_holds_from_value() {
        let kfs = vec![
            kf_eased(0.0, 0.0, EasingFunction::Step),
            kf(1.0, 10.0),
        ];
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
