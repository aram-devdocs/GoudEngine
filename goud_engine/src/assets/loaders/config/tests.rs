//! Unit tests for config asset types and the config loader.

use serde::Deserialize;

use crate::assets::{Asset, AssetLoader, AssetPath, AssetType, LoadContext};

use super::{asset::ConfigAsset, format::ConfigFormat, loader::ConfigLoader};

// =============================================================================
// ConfigFormat Tests
// =============================================================================

mod config_format {
    use super::*;

    #[test]
    fn test_extension() {
        assert_eq!(ConfigFormat::Json.extension(), "json");
        assert_eq!(ConfigFormat::Toml.extension(), "toml");
    }

    #[test]
    fn test_name() {
        assert_eq!(ConfigFormat::Json.name(), "JSON");
        assert_eq!(ConfigFormat::Toml.name(), "TOML");
    }

    #[test]
    fn test_from_extension_json() {
        assert_eq!(
            ConfigFormat::from_extension("json"),
            Some(ConfigFormat::Json)
        );
        assert_eq!(
            ConfigFormat::from_extension("JSON"),
            Some(ConfigFormat::Json)
        );
    }

    #[test]
    fn test_from_extension_toml() {
        assert_eq!(
            ConfigFormat::from_extension("toml"),
            Some(ConfigFormat::Toml)
        );
        assert_eq!(
            ConfigFormat::from_extension("TOML"),
            Some(ConfigFormat::Toml)
        );
    }

    #[test]
    fn test_from_extension_unknown() {
        assert_eq!(ConfigFormat::from_extension("xml"), None);
        assert_eq!(ConfigFormat::from_extension("yaml"), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ConfigFormat::Json), "JSON");
        assert_eq!(format!("{}", ConfigFormat::Toml), "TOML");
    }

    #[test]
    fn test_default() {
        assert_eq!(ConfigFormat::default(), ConfigFormat::Json);
    }

    #[test]
    fn test_equality() {
        assert_eq!(ConfigFormat::Json, ConfigFormat::Json);
        assert_ne!(ConfigFormat::Json, ConfigFormat::Toml);
    }
}

// =============================================================================
// ConfigAsset Tests
// =============================================================================

mod config_asset {
    use super::*;

    #[test]
    fn test_new() {
        let value = serde_json::json!({"key": "value"});
        let asset = ConfigAsset::new(value.clone(), ConfigFormat::Json);
        assert_eq!(asset.format(), ConfigFormat::Json);
        assert_eq!(asset.data(), &value);
    }

    #[test]
    fn test_get_existing_key() {
        let value = serde_json::json!({"name": "test", "count": 42});
        let asset = ConfigAsset::new(value, ConfigFormat::Json);

        assert_eq!(asset.get("name").unwrap(), "test");
        assert_eq!(asset.get("count").unwrap(), 42);
    }

    #[test]
    fn test_get_missing_key() {
        let value = serde_json::json!({"name": "test"});
        let asset = ConfigAsset::new(value, ConfigFormat::Json);

        assert!(asset.get("missing").is_none());
    }

    #[test]
    fn test_is_object() {
        let obj = ConfigAsset::new(serde_json::json!({"a": 1}), ConfigFormat::Json);
        assert!(obj.is_object());

        let arr = ConfigAsset::new(serde_json::json!([1, 2, 3]), ConfigFormat::Json);
        assert!(!arr.is_object());
    }

    #[test]
    fn test_is_array() {
        let arr = ConfigAsset::new(serde_json::json!([1, 2, 3]), ConfigFormat::Json);
        assert!(arr.is_array());

        let obj = ConfigAsset::new(serde_json::json!({"a": 1}), ConfigFormat::Json);
        assert!(!obj.is_array());
    }

    #[test]
    fn test_into_data() {
        let value = serde_json::json!({"key": "value"});
        let asset = ConfigAsset::new(value.clone(), ConfigFormat::Json);
        assert_eq!(asset.into_data(), value);
    }

    #[test]
    fn test_deserialize_into_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct GameConfig {
            title: String,
            width: u32,
            fullscreen: bool,
        }

        let value = serde_json::json!({
            "title": "My Game",
            "width": 1920,
            "fullscreen": false
        });
        let asset = ConfigAsset::new(value, ConfigFormat::Json);

        let config: GameConfig = asset.deserialize().unwrap();
        assert_eq!(config.title, "My Game");
        assert_eq!(config.width, 1920);
        assert!(!config.fullscreen);
    }

    #[test]
    fn test_deserialize_type_mismatch() {
        #[derive(Deserialize, Debug)]
        struct Strict {
            count: u32,
        }

        let value = serde_json::json!({"count": "not_a_number"});
        let asset = ConfigAsset::new(value, ConfigFormat::Json);

        let result = asset.deserialize::<Strict>();
        assert!(result.is_err());
    }

    #[test]
    fn test_asset_trait() {
        assert_eq!(ConfigAsset::asset_type_name(), "Config");
        assert_eq!(ConfigAsset::asset_type(), AssetType::Config);
        assert!(ConfigAsset::extensions().contains(&"json"));
        assert!(ConfigAsset::extensions().contains(&"toml"));
    }

    #[test]
    fn test_clone() {
        let value = serde_json::json!({"a": 1});
        let asset1 = ConfigAsset::new(value, ConfigFormat::Json);
        let asset2 = asset1.clone();
        assert_eq!(asset1.data(), asset2.data());
        assert_eq!(asset1.format(), asset2.format());
    }

    #[test]
    fn test_debug() {
        let value = serde_json::json!({"x": 1});
        let asset = ConfigAsset::new(value, ConfigFormat::Json);
        let debug_str = format!("{:?}", asset);
        assert!(debug_str.contains("ConfigAsset"));
    }
}

// =============================================================================
// ConfigLoader Tests
// =============================================================================

mod config_loader {
    use super::*;

    #[test]
    fn test_new() {
        let loader = ConfigLoader::new();
        assert!(loader.extensions().len() >= 2);
    }

    #[test]
    fn test_default() {
        let loader = ConfigLoader::default();
        assert!(loader.supports_extension("json"));
        assert!(loader.supports_extension("toml"));
    }

    #[test]
    fn test_extensions() {
        let loader = ConfigLoader::new();
        assert!(loader.supports_extension("json"));
        assert!(loader.supports_extension("toml"));
        assert!(!loader.supports_extension("xml"));
        assert!(!loader.supports_extension("yaml"));
    }

    #[test]
    fn test_load_valid_json() {
        let loader = ConfigLoader::new();
        let json_bytes = br#"{"name": "test", "value": 42}"#;
        let path = AssetPath::from_string("config.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(json_bytes, &(), &mut context);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert_eq!(asset.format(), ConfigFormat::Json);
        assert_eq!(asset.get("name").unwrap(), "test");
        assert_eq!(asset.get("value").unwrap(), 42);
    }

    #[test]
    fn test_load_json_array() {
        let loader = ConfigLoader::new();
        let json_bytes = br#"[1, 2, 3]"#;
        let path = AssetPath::from_string("data.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(json_bytes, &(), &mut context);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert!(asset.is_array());
    }

    #[test]
    fn test_load_invalid_json() {
        let loader = ConfigLoader::new();
        let bad_bytes = b"{ not valid json !!!";
        let path = AssetPath::from_string("bad.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(bad_bytes, &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_load_empty_json_object() {
        let loader = ConfigLoader::new();
        let json_bytes = b"{}";
        let path = AssetPath::from_string("empty.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(json_bytes, &(), &mut context);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert!(asset.is_object());
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_load_valid_toml() {
        let loader = ConfigLoader::new();
        let toml_bytes = b"[game]\ntitle = \"Test\"\nwidth = 800\n";
        let path = AssetPath::from_string("config.toml".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(toml_bytes, &(), &mut context);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert_eq!(asset.format(), ConfigFormat::Toml);
        let game = asset.get("game").unwrap();
        assert_eq!(game.get("title").unwrap(), "Test");
        assert_eq!(game.get("width").unwrap(), 800);
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_load_invalid_toml() {
        let loader = ConfigLoader::new();
        let bad_bytes = b"[invalid\nnot = valid toml !!!";
        let path = AssetPath::from_string("bad.toml".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(bad_bytes, &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_load_toml_with_types() {
        let loader = ConfigLoader::new();
        let toml_bytes = b"name = \"hello\"\ncount = 5\npi = 3.14\nenabled = true\n";
        let path = AssetPath::from_string("types.toml".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(toml_bytes, &(), &mut context);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert_eq!(asset.get("name").unwrap(), "hello");
        assert_eq!(asset.get("count").unwrap(), 5);
        assert_eq!(asset.get("enabled").unwrap(), true);
    }

    #[test]
    fn test_load_json_then_deserialize() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Settings {
            volume: f64,
            muted: bool,
        }

        let loader = ConfigLoader::new();
        let json_bytes = br#"{"volume": 0.8, "muted": false}"#;
        let path = AssetPath::from_string("audio.json".to_string());
        let mut context = LoadContext::new(path);

        let asset = loader.load(json_bytes, &(), &mut context).unwrap();
        let settings: Settings = asset.deserialize().unwrap();
        assert!((settings.volume - 0.8).abs() < f64::EPSILON);
        assert!(!settings.muted);
    }

    #[test]
    fn test_clone() {
        let loader1 = ConfigLoader::new();
        let loader2 = loader1.clone();
        assert_eq!(loader1.extensions(), loader2.extensions());
    }

    #[test]
    fn test_debug() {
        let loader = ConfigLoader::new();
        let debug_str = format!("{:?}", loader);
        assert!(debug_str.contains("ConfigLoader"));
    }
}
