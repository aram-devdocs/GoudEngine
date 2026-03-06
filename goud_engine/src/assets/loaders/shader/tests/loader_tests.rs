//! Tests for [`ShaderLoader`] and integration scenarios.

use crate::assets::loaders::shader::{ShaderFormat, ShaderStage};
use crate::assets::loaders::shader::{ShaderLoader, ShaderSettings};
use crate::assets::{AssetLoader, LoadContext};

// -------------------------------------------------------------------------
// ShaderLoader Tests
// -------------------------------------------------------------------------

#[test]
fn test_shader_loader_new() {
    let loader = ShaderLoader::new();
    assert!(loader.settings.validate);
}

#[test]
fn test_shader_loader_with_settings() {
    let mut settings = ShaderSettings::default();
    settings.validate = false;
    let loader = ShaderLoader::with_settings(settings);
    assert!(!loader.settings.validate);
}

#[test]
fn test_shader_loader_extensions() {
    let loader = ShaderLoader::new();
    let extensions = loader.extensions();
    assert!(extensions.contains(&"vert"));
    assert!(extensions.contains(&"frag"));
    assert!(extensions.contains(&"shader"));
}

#[test]
fn test_shader_loader_load_single_stage_vertex() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = b"#version 330 core\nvoid main() {}";
    let mut context = LoadContext::new("shader.vert".into());

    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_ok());
    let shader = result.unwrap();
    assert!(shader.has_stage(ShaderStage::Vertex));
    assert_eq!(shader.stage_count(), 1);
}

#[test]
fn test_shader_loader_load_single_stage_fragment() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = b"#version 330 core\nvoid main() {}";
    let mut context = LoadContext::new("shader.frag".into());

    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_ok());
    let shader = result.unwrap();
    assert!(shader.has_stage(ShaderStage::Fragment));
}

#[test]
fn test_shader_loader_load_combined() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = b"#pragma stage vertex\n#version 330 core\nvoid main() {}\n\n#pragma stage fragment\n#version 330 core\nvoid main() {}";
    let mut context = LoadContext::new("shader.shader".into());

    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_ok());
    let shader = result.unwrap();
    assert!(shader.has_stage(ShaderStage::Vertex));
    assert!(shader.has_stage(ShaderStage::Fragment));
    assert_eq!(shader.stage_count(), 2);
}

#[test]
fn test_shader_loader_load_combined_case_insensitive() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source =
        b"#pragma STAGE VERTEX\n#version 330 core\nvoid main() {}\n\n#pragma stage FRAGMENT\n#version 330 core\nvoid main() {}";
    let mut context = LoadContext::new("shader.shader".into());

    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_ok());
    let shader = result.unwrap();
    assert_eq!(shader.stage_count(), 2);
}

#[test]
fn test_shader_loader_load_invalid_utf8() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = &[0xFF, 0xFF, 0xFF]; // Invalid UTF-8
    let mut context = LoadContext::new("shader.vert".into());

    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_err());
}

#[test]
fn test_shader_loader_load_no_extension() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = b"#version 330 core\nvoid main() {}";
    let mut context = LoadContext::new("shader".into());

    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_err());
}

#[test]
fn test_shader_loader_load_unsupported_extension() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = b"#version 330 core\nvoid main() {}";
    let mut context = LoadContext::new("shader.txt".into());

    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_err());
}

#[test]
fn test_shader_loader_load_validation_failure() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = b"not valid shader code"; // Missing #version and main()
    let mut context = LoadContext::new("shader.vert".into());

    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_err());
}

#[test]
fn test_shader_loader_load_no_validation() {
    use crate::assets::AssetLoader;

    let mut settings = ShaderSettings::default();
    settings.validate = false;
    let loader = ShaderLoader::with_settings(settings.clone());
    let source = b"not valid shader code";
    let mut context = LoadContext::new("shader.vert".into());

    // Should succeed because validation is disabled
    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_ok());
}

#[test]
fn test_shader_loader_sets_name() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = b"#version 330 core\nvoid main() {}";
    let mut context = LoadContext::new("my_shader.vert".into());

    let result = loader.load(source, &settings, &mut context);
    assert!(result.is_ok());
    let shader = result.unwrap();
    assert_eq!(shader.name(), Some("my_shader"));
}

#[test]
fn test_shader_loader_detect_format_single_stage() {
    let loader = ShaderLoader::new();
    let source = "#version 330 core\nvoid main() {}";
    let format = loader.detect_format(source, "vert");
    assert_eq!(format, ShaderFormat::SingleStage);
}

#[test]
fn test_shader_loader_detect_format_combined() {
    let loader = ShaderLoader::new();
    let source = "#pragma stage vertex\n#version 330 core\nvoid main() {}";
    let format = loader.detect_format(source, "shader");
    assert_eq!(format, ShaderFormat::Combined);
}

#[test]
fn test_shader_loader_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ShaderLoader>();
}

// -------------------------------------------------------------------------
// Integration Tests
// -------------------------------------------------------------------------

#[test]
fn test_full_workflow_single_stage() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = b"#version 330 core\nlayout(location = 0) in vec3 position;\nvoid main() { gl_Position = vec4(position, 1.0); }";
    let mut context = LoadContext::new("shader.vert".into());

    let shader = loader.load(source, &settings, &mut context).unwrap();
    assert!(shader.has_stage(ShaderStage::Vertex));
    assert_eq!(shader.name(), Some("shader"));
    // Single-stage shaders don't pass full validation (missing fragment)
    // But the individual stage source is validated during loading
    assert!(shader
        .get_stage(ShaderStage::Vertex)
        .unwrap()
        .validate()
        .is_ok());
}

#[test]
fn test_full_workflow_combined() {
    let loader = ShaderLoader::new();
    let settings = ShaderSettings::default();
    let source = b"#pragma stage vertex\n#version 330 core\nvoid main() {}\n\n#pragma stage fragment\n#version 330 core\nvoid main() {}";
    let mut context = LoadContext::new("shader.shader".into());

    let shader = loader.load(source, &settings, &mut context).unwrap();
    assert!(shader.is_graphics_shader());
    assert_eq!(shader.stage_count(), 2);
}

#[test]
fn test_shader_with_defines() {
    use crate::assets::AssetLoader;

    let mut settings = ShaderSettings::default();
    settings
        .defines
        .insert("MAX_LIGHTS".to_string(), "8".to_string());
    let loader = ShaderLoader::with_settings(settings.clone());

    let source = b"#version 330 core\nvoid main() {}";
    let mut context = LoadContext::new("shader.vert".into());

    let shader = loader.load(source, &settings, &mut context).unwrap();
    assert_eq!(shader.get_define("MAX_LIGHTS"), Some("8"));
}
