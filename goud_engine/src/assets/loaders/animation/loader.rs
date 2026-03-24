//! [`AnimationLoader`] implementing the [`AssetLoader`] trait for animation files.

use crate::assets::{asset::Asset, AssetLoadError, AssetLoader, LoadContext};

use super::asset::KeyframeAnimation;

/// Animation asset loader for `.anim.json`, `.gltf`, and `.glb` files.
///
/// Parses keyframe animation data into a [`KeyframeAnimation`] asset.
///
/// # Supported Formats
/// - Custom JSON (`.anim.json`)
/// - GLTF/GLB (`.gltf`, `.glb`) — requires `native` feature
///
/// # Example
/// ```no_run
/// use goud_engine::assets::{AssetServer, loaders::animation::{AnimationLoader, KeyframeAnimation}};
///
/// let mut server = AssetServer::new();
/// server.register_loader(AnimationLoader::default());
/// ```
#[derive(Clone, Debug, Default)]
pub struct AnimationLoader;

impl AssetLoader for AnimationLoader {
    type Asset = KeyframeAnimation;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        KeyframeAnimation::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let ext = context.extension().unwrap_or("");

        match ext {
            #[cfg(feature = "native")]
            "gltf" | "glb" => super::gltf_parser::parse_gltf_animation(bytes),
            _ => self.load_json(bytes),
        }
    }
}

impl AnimationLoader {
    fn load_json(&self, bytes: &[u8]) -> Result<KeyframeAnimation, AssetLoadError> {
        let text = std::str::from_utf8(bytes).map_err(|e| {
            AssetLoadError::decode_failed(format!("Animation file is not valid UTF-8: {e}"))
        })?;

        let mut animation: KeyframeAnimation = serde_json::from_str(text).map_err(|e| {
            AssetLoadError::decode_failed(format!("Failed to parse animation JSON: {e}"))
        })?;
        animation.build_channel_index();

        Ok(animation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{AssetPath, LoadContext};

    const VALID_JSON: &str = r#"{
        "name": "bounce",
        "duration": 1.5,
        "channels": [
            {
                "target_property": "transform.position.y",
                "keyframes": [
                    { "time": 0.0, "value": 0.0, "easing": { "type": "linear" } },
                    { "time": 0.75, "value": 100.0, "easing": { "type": "ease_out" } },
                    { "time": 1.5, "value": 0.0, "easing": { "type": "ease_in" } }
                ]
            }
        ]
    }"#;

    const MINIMAL_JSON: &str = r#"{
        "name": "empty",
        "duration": 0.0,
        "channels": []
    }"#;

    #[test]
    fn test_animation_loader_extensions() {
        let loader = AnimationLoader::default();
        let exts = loader.extensions();
        assert!(exts.contains(&"anim.json"));
        assert!(exts.contains(&"gltf"));
        assert!(exts.contains(&"glb"));
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_animation_loader_invalid_gltf() {
        let loader = AnimationLoader::default();
        let path = AssetPath::from_string("bad.glb".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(b"not a glb file", &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_animation_loader_load_valid_json() {
        let loader = AnimationLoader::default();
        let path = AssetPath::from_string("bounce.anim.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(VALID_JSON.as_bytes(), &(), &mut context);
        assert!(result.is_ok(), "Valid JSON should load: {:?}", result.err());

        let anim = result.unwrap();
        assert_eq!(anim.name(), "bounce");
        assert!((anim.duration() - 1.5).abs() < f32::EPSILON);
        assert_eq!(anim.channel_count(), 1);
        assert_eq!(anim.total_keyframe_count(), 3);
    }

    #[test]
    fn test_animation_loader_load_empty_channels() {
        let loader = AnimationLoader::default();
        let path = AssetPath::from_string("empty.anim.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(MINIMAL_JSON.as_bytes(), &(), &mut context);
        assert!(result.is_ok());

        let anim = result.unwrap();
        assert_eq!(anim.name(), "empty");
        assert!(anim.is_empty());
        assert_eq!(anim.total_keyframe_count(), 0);
    }

    #[test]
    fn test_animation_loader_load_invalid_json() {
        let loader = AnimationLoader::default();
        let path = AssetPath::from_string("bad.anim.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(b"not json", &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_animation_loader_load_invalid_utf8() {
        let loader = AnimationLoader::default();
        let path = AssetPath::from_string("bad.anim.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(&[0xFF, 0xFE, 0xFD], &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_animation_loader_load_empty_bytes() {
        let loader = AnimationLoader::default();
        let path = AssetPath::from_string("empty.anim.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(b"", &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_animation_loader_load_with_default_easing() {
        // Easing defaults to linear when omitted
        let json = r#"{
            "name": "defaults",
            "duration": 1.0,
            "channels": [{
                "target_property": "alpha",
                "keyframes": [
                    { "time": 0.0, "value": 0.0 },
                    { "time": 1.0, "value": 1.0 }
                ]
            }]
        }"#;
        let loader = AnimationLoader::default();
        let path = AssetPath::from_string("defaults.anim.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(json.as_bytes(), &(), &mut context);
        assert!(result.is_ok());
        let anim = result.unwrap();
        assert_eq!(anim.total_keyframe_count(), 2);
    }

    #[test]
    fn test_animation_loader_clone() {
        let l1 = AnimationLoader::default();
        let l2 = l1.clone();
        assert_eq!(l1.extensions(), l2.extensions());
    }

    #[test]
    fn test_animation_loader_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AnimationLoader>();
    }

    #[test]
    fn test_animation_loader_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<AnimationLoader>();
    }
}
