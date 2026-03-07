//! [`KeyframeAnimation`] asset definition.

use crate::assets::{Asset, AssetType};
use serde::{Deserialize, Serialize};

use super::keyframe::AnimationChannel;

/// A keyframe-based property animation asset.
///
/// Unlike sprite-sheet based [`AnimationClip`](crate::ecs::components::sprite_animator),
/// this type drives arbitrary numeric properties over time using keyframes
/// with easing functions.
///
/// # Example
/// ```
/// use goud_engine::assets::loaders::animation::{
///     KeyframeAnimation,
///     keyframe::{AnimationChannel, Keyframe, EasingFunction},
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
        self.channels
            .iter()
            .find(|c| c.target_property == property)
    }

    /// Returns the total number of keyframes across all channels.
    pub fn total_keyframe_count(&self) -> usize {
        self.channels.iter().map(|c| c.keyframes.len()).sum()
    }
}

impl Asset for KeyframeAnimation {
    fn asset_type_name() -> &'static str {
        "KeyframeAnimation"
    }

    fn asset_type() -> AssetType {
        AssetType::Animation
    }

    fn extensions() -> &'static [&'static str] {
        &["anim.json"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loaders::animation::keyframe::{EasingFunction, Keyframe};

    fn sample_animation() -> KeyframeAnimation {
        KeyframeAnimation::new(
            "test_anim".to_string(),
            2.0,
            vec![
                AnimationChannel {
                    target_property: "transform.position.x".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.0,
                            value: 0.0,
                            easing: EasingFunction::Linear,
                        },
                        Keyframe {
                            time: 2.0,
                            value: 100.0,
                            easing: EasingFunction::Linear,
                        },
                    ],
                },
                AnimationChannel {
                    target_property: "transform.position.y".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 50.0,
                        easing: EasingFunction::EaseIn,
                    }],
                },
            ],
        )
    }

    #[test]
    fn test_keyframe_animation_new() {
        let anim = sample_animation();
        assert_eq!(anim.name(), "test_anim");
        assert!((anim.duration() - 2.0).abs() < f32::EPSILON);
        assert_eq!(anim.channel_count(), 2);
    }

    #[test]
    fn test_keyframe_animation_is_empty() {
        let empty = KeyframeAnimation::new("empty".to_string(), 0.0, vec![]);
        assert!(empty.is_empty());
        assert_eq!(empty.channel_count(), 0);
    }

    #[test]
    fn test_keyframe_animation_channel_by_property() {
        let anim = sample_animation();
        let ch = anim.channel_by_property("transform.position.x");
        assert!(ch.is_some());
        assert_eq!(ch.unwrap().keyframes.len(), 2);

        assert!(anim.channel_by_property("nonexistent").is_none());
    }

    #[test]
    fn test_keyframe_animation_total_keyframe_count() {
        let anim = sample_animation();
        assert_eq!(anim.total_keyframe_count(), 3);
    }

    #[test]
    fn test_keyframe_animation_asset_trait() {
        assert_eq!(KeyframeAnimation::asset_type_name(), "KeyframeAnimation");
        assert_eq!(KeyframeAnimation::asset_type(), AssetType::Animation);
        assert_eq!(KeyframeAnimation::extensions(), &["anim.json"]);
    }

    #[test]
    fn test_keyframe_animation_clone() {
        let a1 = sample_animation();
        let a2 = a1.clone();
        assert_eq!(a1, a2);
    }

    #[test]
    fn test_keyframe_animation_serde_roundtrip() {
        let anim = sample_animation();
        let json = serde_json::to_string(&anim).unwrap();
        let parsed: KeyframeAnimation = serde_json::from_str(&json).unwrap();
        assert_eq!(anim, parsed);
    }

    #[test]
    fn test_keyframe_animation_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<KeyframeAnimation>();
    }

    #[test]
    fn test_keyframe_animation_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<KeyframeAnimation>();
    }
}
