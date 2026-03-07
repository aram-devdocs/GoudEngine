//! Unit tests for material asset types, uniform values, and the material loader.

use std::collections::HashMap;

use crate::assets::{Asset, AssetLoader, AssetPath, AssetType, LoadContext};

use super::{asset::MaterialAsset, loader::MaterialLoader, uniform::UniformValue};

// =============================================================================
// UniformValue Tests
// =============================================================================

mod uniform_value {
    use super::*;

    #[test]
    fn test_float_variant() {
        let v = UniformValue::Float(1.5);
        assert_eq!(v, UniformValue::Float(1.5));
        assert_eq!(v.type_name(), "Float");
    }

    #[test]
    fn test_vec2_variant() {
        let v = UniformValue::Vec2([1.0, 2.0]);
        assert_eq!(v, UniformValue::Vec2([1.0, 2.0]));
        assert_eq!(v.type_name(), "Vec2");
    }

    #[test]
    fn test_vec3_variant() {
        let v = UniformValue::Vec3([1.0, 2.0, 3.0]);
        assert_eq!(v, UniformValue::Vec3([1.0, 2.0, 3.0]));
        assert_eq!(v.type_name(), "Vec3");
    }

    #[test]
    fn test_vec4_variant() {
        let v = UniformValue::Vec4([1.0, 0.0, 0.0, 1.0]);
        assert_eq!(v, UniformValue::Vec4([1.0, 0.0, 0.0, 1.0]));
        assert_eq!(v.type_name(), "Vec4");
    }

    #[test]
    fn test_int_variant() {
        let v = UniformValue::Int(42);
        assert_eq!(v, UniformValue::Int(42));
        assert_eq!(v.type_name(), "Int");
    }

    #[test]
    fn test_mat4_variant() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let v = UniformValue::Mat4(identity);
        assert_eq!(v, UniformValue::Mat4(identity));
        assert_eq!(v.type_name(), "Mat4");
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let values = vec![
            UniformValue::Float(3.14),
            UniformValue::Vec2([1.0, 2.0]),
            UniformValue::Vec3([1.0, 2.0, 3.0]),
            UniformValue::Vec4([1.0, 0.5, 0.0, 1.0]),
            UniformValue::Int(-7),
            UniformValue::Mat4([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]),
        ];

        for original in &values {
            let json = serde_json::to_string(original).expect("serialize");
            let restored: UniformValue = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(
                original,
                &restored,
                "roundtrip failed for {}",
                original.type_name()
            );
        }
    }

    #[test]
    fn test_clone() {
        let v = UniformValue::Vec3([1.0, 2.0, 3.0]);
        let cloned = v.clone();
        assert_eq!(v, cloned);
    }

    #[test]
    fn test_debug() {
        let v = UniformValue::Float(1.0);
        let debug_str = format!("{:?}", v);
        assert!(debug_str.contains("Float"));
    }

    #[test]
    fn test_inequality() {
        assert_ne!(UniformValue::Float(1.0), UniformValue::Float(2.0));
        assert_ne!(UniformValue::Float(1.0), UniformValue::Int(1));
    }
}

// =============================================================================
// MaterialAsset Tests
// =============================================================================

mod material_asset {
    use super::*;

    fn sample_asset() -> MaterialAsset {
        let mut uniforms = HashMap::new();
        uniforms.insert(
            "color".to_string(),
            UniformValue::Vec4([1.0, 0.0, 0.0, 1.0]),
        );
        uniforms.insert("roughness".to_string(), UniformValue::Float(0.5));

        let mut textures = HashMap::new();
        textures.insert("albedo".to_string(), "textures/brick.png".to_string());

        MaterialAsset::new(
            "brick".to_string(),
            "shaders/pbr.glsl".to_string(),
            uniforms,
            textures,
        )
    }

    #[test]
    fn test_new_and_accessors() {
        let asset = sample_asset();
        assert_eq!(asset.name(), "brick");
        assert_eq!(asset.shader_path(), "shaders/pbr.glsl");
        assert_eq!(asset.uniforms().len(), 2);
        assert_eq!(asset.texture_slots().len(), 1);
    }

    #[test]
    fn test_get_uniform_existing() {
        let asset = sample_asset();
        let roughness = asset.get_uniform("roughness").unwrap();
        assert_eq!(roughness, &UniformValue::Float(0.5));
    }

    #[test]
    fn test_get_uniform_missing() {
        let asset = sample_asset();
        assert!(asset.get_uniform("nonexistent").is_none());
    }

    #[test]
    fn test_get_texture_slot_existing() {
        let asset = sample_asset();
        assert_eq!(asset.get_texture_slot("albedo"), Some("textures/brick.png"));
    }

    #[test]
    fn test_get_texture_slot_missing() {
        let asset = sample_asset();
        assert!(asset.get_texture_slot("normal").is_none());
    }

    #[test]
    fn test_asset_trait() {
        assert_eq!(MaterialAsset::asset_type_name(), "Material");
        assert_eq!(MaterialAsset::asset_type(), AssetType::Material);
        assert!(MaterialAsset::extensions().contains(&"mat.json"));
    }

    #[test]
    fn test_clone() {
        let asset1 = sample_asset();
        let asset2 = asset1.clone();
        assert_eq!(asset1.name(), asset2.name());
        assert_eq!(asset1.shader_path(), asset2.shader_path());
        assert_eq!(asset1.uniforms().len(), asset2.uniforms().len());
    }

    #[test]
    fn test_debug() {
        let asset = sample_asset();
        let debug_str = format!("{:?}", asset);
        assert!(debug_str.contains("MaterialAsset"));
    }

    #[test]
    fn test_empty_uniforms_and_textures() {
        let asset = MaterialAsset::new(
            "minimal".to_string(),
            "shaders/basic.glsl".to_string(),
            HashMap::new(),
            HashMap::new(),
        );
        assert!(asset.uniforms().is_empty());
        assert!(asset.texture_slots().is_empty());
    }
}

// =============================================================================
// MaterialLoader Tests
// =============================================================================

mod material_loader {
    use super::*;

    fn valid_material_json() -> &'static [u8] {
        br#"{
            "name": "test_material",
            "shader_path": "shaders/test.glsl",
            "uniforms": {
                "color": { "type": "Vec4", "value": [1.0, 0.0, 0.0, 1.0] },
                "intensity": { "type": "Float", "value": 0.8 }
            },
            "texture_slots": {
                "diffuse": "textures/diffuse.png",
                "normal": "textures/normal.png"
            }
        }"#
    }

    fn minimal_material_json() -> &'static [u8] {
        br#"{
            "name": "minimal",
            "shader_path": "shaders/basic.glsl"
        }"#
    }

    #[test]
    fn test_new() {
        let loader = MaterialLoader::new();
        assert!(!loader.extensions().is_empty());
    }

    #[test]
    fn test_default() {
        let loader = MaterialLoader::default();
        assert!(loader.supports_extension("mat.json"));
    }

    #[test]
    fn test_extensions() {
        let loader = MaterialLoader::new();
        assert!(loader.supports_extension("mat.json"));
        assert!(!loader.supports_extension("json"));
        assert!(!loader.supports_extension("material"));
    }

    #[test]
    fn test_load_valid_material() {
        let loader = MaterialLoader::new();
        let path = AssetPath::from_string("materials/test.mat.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(valid_material_json(), &(), &mut context);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert_eq!(asset.name(), "test_material");
        assert_eq!(asset.shader_path(), "shaders/test.glsl");
        assert_eq!(asset.uniforms().len(), 2);
        assert_eq!(
            asset.get_uniform("color"),
            Some(&UniformValue::Vec4([1.0, 0.0, 0.0, 1.0]))
        );
        assert_eq!(
            asset.get_uniform("intensity"),
            Some(&UniformValue::Float(0.8))
        );
        assert_eq!(asset.texture_slots().len(), 2);
        assert_eq!(
            asset.get_texture_slot("diffuse"),
            Some("textures/diffuse.png")
        );
        assert_eq!(
            asset.get_texture_slot("normal"),
            Some("textures/normal.png")
        );
    }

    #[test]
    fn test_load_minimal_material_defaults_optional_fields() {
        let loader = MaterialLoader::new();
        let path = AssetPath::from_string("materials/minimal.mat.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(minimal_material_json(), &(), &mut context);
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert_eq!(asset.name(), "minimal");
        assert_eq!(asset.shader_path(), "shaders/basic.glsl");
        assert!(asset.uniforms().is_empty());
        assert!(asset.texture_slots().is_empty());
    }

    #[test]
    fn test_load_invalid_json() {
        let loader = MaterialLoader::new();
        let bad_bytes = b"{ not valid json !!!";
        let path = AssetPath::from_string("bad.mat.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(bad_bytes, &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_load_missing_required_name_field() {
        let loader = MaterialLoader::new();
        let bytes = br#"{"shader_path": "shaders/test.glsl"}"#;
        let path = AssetPath::from_string("missing_name.mat.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(bytes, &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_load_missing_required_shader_path_field() {
        let loader = MaterialLoader::new();
        let bytes = br#"{"name": "test"}"#;
        let path = AssetPath::from_string("missing_shader.mat.json".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(bytes, &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_dependency_declaration_shader() {
        let loader = MaterialLoader::new();
        let path = AssetPath::from_string("materials/test.mat.json".to_string());
        let mut context = LoadContext::new(path);

        loader
            .load(valid_material_json(), &(), &mut context)
            .unwrap();

        let deps = context.dependencies();
        assert!(
            deps.contains(&"shaders/test.glsl".to_string()),
            "shader dependency should be declared"
        );
    }

    #[test]
    fn test_dependency_declaration_textures() {
        let loader = MaterialLoader::new();
        let path = AssetPath::from_string("materials/test.mat.json".to_string());
        let mut context = LoadContext::new(path);

        loader
            .load(valid_material_json(), &(), &mut context)
            .unwrap();

        let deps = context.dependencies();
        assert!(
            deps.contains(&"textures/diffuse.png".to_string()),
            "diffuse texture dependency should be declared"
        );
        assert!(
            deps.contains(&"textures/normal.png".to_string()),
            "normal texture dependency should be declared"
        );
    }

    #[test]
    fn test_dependency_count() {
        let loader = MaterialLoader::new();
        let path = AssetPath::from_string("materials/test.mat.json".to_string());
        let mut context = LoadContext::new(path);

        loader
            .load(valid_material_json(), &(), &mut context)
            .unwrap();

        // 1 shader + 2 textures = 3 dependencies
        assert_eq!(context.dependencies().len(), 3);
    }

    #[test]
    fn test_minimal_material_has_only_shader_dependency() {
        let loader = MaterialLoader::new();
        let path = AssetPath::from_string("materials/minimal.mat.json".to_string());
        let mut context = LoadContext::new(path);

        loader
            .load(minimal_material_json(), &(), &mut context)
            .unwrap();

        let deps = context.dependencies();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "shaders/basic.glsl");
    }

    #[test]
    fn test_all_uniform_variants_in_json() {
        let json = br#"{
            "name": "all_uniforms",
            "shader_path": "shaders/test.glsl",
            "uniforms": {
                "f": { "type": "Float", "value": 1.5 },
                "v2": { "type": "Vec2", "value": [1.0, 2.0] },
                "v3": { "type": "Vec3", "value": [1.0, 2.0, 3.0] },
                "v4": { "type": "Vec4", "value": [1.0, 2.0, 3.0, 4.0] },
                "i": { "type": "Int", "value": -5 },
                "m": { "type": "Mat4", "value": [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0]
                ]}
            }
        }"#;

        let loader = MaterialLoader::new();
        let path = AssetPath::from_string("uniforms.mat.json".to_string());
        let mut context = LoadContext::new(path);

        let asset = loader.load(json, &(), &mut context).unwrap();

        assert_eq!(asset.get_uniform("f"), Some(&UniformValue::Float(1.5)));
        assert_eq!(
            asset.get_uniform("v2"),
            Some(&UniformValue::Vec2([1.0, 2.0]))
        );
        assert_eq!(
            asset.get_uniform("v3"),
            Some(&UniformValue::Vec3([1.0, 2.0, 3.0]))
        );
        assert_eq!(
            asset.get_uniform("v4"),
            Some(&UniformValue::Vec4([1.0, 2.0, 3.0, 4.0]))
        );
        assert_eq!(asset.get_uniform("i"), Some(&UniformValue::Int(-5)));
        assert_eq!(
            asset.get_uniform("m"),
            Some(&UniformValue::Mat4([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]))
        );
    }

    #[test]
    fn test_clone() {
        let loader1 = MaterialLoader::new();
        let loader2 = loader1.clone();
        assert_eq!(loader1.extensions(), loader2.extensions());
    }

    #[test]
    fn test_debug() {
        let loader = MaterialLoader::new();
        let debug_str = format!("{:?}", loader);
        assert!(debug_str.contains("MaterialLoader"));
    }
}
