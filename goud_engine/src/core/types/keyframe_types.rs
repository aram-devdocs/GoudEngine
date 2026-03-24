//! Keyframe animation types and interpolation shared between the asset pipeline and the renderer.
//!
//! These types live in the foundation layer so that both `libs/` (Layer 2) and
//! `assets/` (Layer 3) can depend on them without creating upward imports.

use serde::{Deserialize, Serialize};

// =============================================================================
// EasingFunction
// =============================================================================

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

// =============================================================================
// Keyframe
// =============================================================================

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

// =============================================================================
// AnimationChannel
// =============================================================================

/// An animation channel targeting a specific property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationChannel {
    /// Property path targeted by this channel (e.g. "transform.position.x").
    pub target_property: String,
    /// Keyframes sorted by time.
    pub keyframes: Vec<Keyframe>,
}

// =============================================================================
// Interpolation
// =============================================================================

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

// =============================================================================
// KeyframeAnimation
// =============================================================================

/// A keyframe-based property animation asset.
///
/// Unlike sprite-sheet based [`AnimationClip`](crate::ecs::components::sprite_animator),
/// this type drives arbitrary numeric properties over time using keyframes
/// with easing functions.
///
/// # Example
/// ```
/// use goud_engine::core::types::keyframe_types::{
///     KeyframeAnimation, AnimationChannel, Keyframe, EasingFunction,
/// };
///
/// let anim = KeyframeAnimation::new(
///     "bounce".to_string(),
///     1.0,
///     vec![AnimationChannel {
///         target_property: "transform.position.y".to_string(),
///         keyframes: vec![
///             Keyframe { time: 0.0, value: 0.0, easing: EasingFunction::EaseOut },
///             Keyframe { time: 1.0, value: 100.0, easing: EasingFunction::Linear },
///         ],
///     }],
/// );
/// assert_eq!(anim.name(), "bounce");
/// assert_eq!(anim.channel_count(), 1);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyframeAnimation {
    /// Human-readable name for this animation.
    pub name: String,
    /// Total duration in seconds.
    pub duration: f32,
    /// Property channels with keyframes.
    pub channels: Vec<AnimationChannel>,
}

impl KeyframeAnimation {
    /// Creates a new keyframe animation.
    pub fn new(name: String, duration: f32, channels: Vec<AnimationChannel>) -> Self {
        Self {
            name,
            duration,
            channels,
        }
    }

    /// Returns the animation name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the total duration in seconds.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Returns a slice of all animation channels.
    pub fn channels(&self) -> &[AnimationChannel] {
        &self.channels
    }

    /// Returns the number of channels.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Returns true if this animation has no channels.
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }

    /// Finds a channel by target property name.
    pub fn channel_by_property(&self, property: &str) -> Option<&AnimationChannel> {
        self.channels.iter().find(|c| c.target_property == property)
    }

    /// Returns the total number of keyframes across all channels.
    pub fn total_keyframe_count(&self) -> usize {
        self.channels.iter().map(|c| c.keyframes.len()).sum()
    }
}
