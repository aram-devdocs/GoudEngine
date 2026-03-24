//! [`KeyframeAnimation`] asset definition.
//!
//! The struct definition now lives in `core::types::keyframe_types`.  This
//! module re-exports it for backward compatibility and provides the [`Asset`]
//! trait implementation.

use crate::assets::{Asset, AssetType};

// Re-export from the canonical core location.
pub use crate::core::types::keyframe_types::KeyframeAnimation;

impl Asset for KeyframeAnimation {
    fn asset_type_name() -> &'static str {
        "KeyframeAnimation"
    }

    fn asset_type() -> AssetType {
        AssetType::Animation
    }

    fn extensions() -> &'static [&'static str] {
        &["anim.json", "gltf", "glb"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::keyframe_types::{AnimationChannel, EasingFunction, Keyframe};

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
        assert!(KeyframeAnimation::extensions().contains(&"anim.json"));
        assert!(KeyframeAnimation::extensions().contains(&"gltf"));
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
