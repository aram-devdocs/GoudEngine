//! Tests for [`ShaderAsset`], [`ShaderFormat`], and [`ShaderSettings`].

use crate::assets::loaders::shader::ShaderSettings;
use crate::assets::{
    asset::Asset,
    loaders::shader::{ShaderAsset, ShaderFormat, ShaderSource, ShaderStage},
    AssetType,
};

// -------------------------------------------------------------------------
// ShaderAsset Tests
// -------------------------------------------------------------------------

#[test]
fn test_shader_asset_new() {
    let shader = ShaderAsset::new();
    assert_eq!(shader.stage_count(), 0);
    assert!(shader.name().is_none());
}

#[test]
fn test_shader_asset_with_name() {
    let shader = ShaderAsset::with_name("test".to_string());
    assert_eq!(shader.name(), Some("test"));
}

#[test]
fn test_shader_asset_add_stage() {
    let mut shader = ShaderAsset::new();
    let source = ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    );
    shader.add_stage(source);

    assert_eq!(shader.stage_count(), 1);
    assert!(shader.has_stage(ShaderStage::Vertex));
}

#[test]
fn test_shader_asset_remove_stage() {
    let mut shader = ShaderAsset::new();
    let source = ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    );
    shader.add_stage(source);

    let removed = shader.remove_stage(ShaderStage::Vertex);
    assert!(removed.is_some());
    assert_eq!(shader.stage_count(), 0);
}

#[test]
fn test_shader_asset_get_stage() {
    let mut shader = ShaderAsset::new();
    let source = ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    );
    shader.add_stage(source);

    let retrieved = shader.get_stage(ShaderStage::Vertex);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().stage, ShaderStage::Vertex);
}

#[test]
fn test_shader_asset_stages_iterator() {
    let mut shader = ShaderAsset::new();
    shader.add_stage(ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    ));
    shader.add_stage(ShaderSource::new(
        ShaderStage::Fragment,
        "#version 330 core\nvoid main() {}".to_string(),
    ));

    let count = shader.stages().count();
    assert_eq!(count, 2);
}

#[test]
fn test_shader_asset_defines() {
    let mut shader = ShaderAsset::new();
    shader.add_define("USE_LIGHTING".to_string(), "1".to_string());

    assert_eq!(shader.get_define("USE_LIGHTING"), Some("1"));
    assert_eq!(shader.defines().len(), 1);
}

#[test]
fn test_shader_asset_validate_empty() {
    let shader = ShaderAsset::new();
    assert!(shader.validate().is_err());
}

#[test]
fn test_shader_asset_validate_compute_only() {
    let mut shader = ShaderAsset::new();
    shader.add_stage(ShaderSource::new(
        ShaderStage::Compute,
        "#version 430 core\nvoid main() {}".to_string(),
    ));

    assert!(shader.validate().is_ok());
    assert!(shader.is_compute_shader());
}

#[test]
fn test_shader_asset_validate_compute_with_others() {
    let mut shader = ShaderAsset::new();
    shader.add_stage(ShaderSource::new(
        ShaderStage::Compute,
        "#version 430 core\nvoid main() {}".to_string(),
    ));
    shader.add_stage(ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    ));

    assert!(shader.validate().is_err());
}

#[test]
fn test_shader_asset_validate_graphics_missing_vertex() {
    let mut shader = ShaderAsset::new();
    shader.add_stage(ShaderSource::new(
        ShaderStage::Fragment,
        "#version 330 core\nvoid main() {}".to_string(),
    ));

    assert!(shader.validate().is_err());
}

#[test]
fn test_shader_asset_validate_graphics_missing_fragment() {
    let mut shader = ShaderAsset::new();
    shader.add_stage(ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    ));

    assert!(shader.validate().is_err());
}

#[test]
fn test_shader_asset_validate_graphics_complete() {
    let mut shader = ShaderAsset::new();
    shader.add_stage(ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    ));
    shader.add_stage(ShaderSource::new(
        ShaderStage::Fragment,
        "#version 330 core\nvoid main() {}".to_string(),
    ));

    assert!(shader.validate().is_ok());
    assert!(shader.is_graphics_shader());
}

#[test]
fn test_shader_asset_total_size_bytes() {
    let mut shader = ShaderAsset::new();
    shader.add_stage(ShaderSource::new(ShaderStage::Vertex, "12345".to_string()));
    shader.add_stage(ShaderSource::new(ShaderStage::Fragment, "6789".to_string()));

    assert_eq!(shader.total_size_bytes(), 9);
}

#[test]
fn test_shader_asset_is_compute_shader() {
    let mut shader = ShaderAsset::new();
    shader.add_stage(ShaderSource::new(
        ShaderStage::Compute,
        "#version 430 core\nvoid main() {}".to_string(),
    ));

    assert!(shader.is_compute_shader());
    assert!(!shader.is_graphics_shader());
}

#[test]
fn test_shader_asset_is_graphics_shader() {
    let mut shader = ShaderAsset::new();
    shader.add_stage(ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    ));
    shader.add_stage(ShaderSource::new(
        ShaderStage::Fragment,
        "#version 330 core\nvoid main() {}".to_string(),
    ));

    assert!(shader.is_graphics_shader());
    assert!(!shader.is_compute_shader());
}

#[test]
fn test_shader_asset_default() {
    let shader = ShaderAsset::default();
    assert_eq!(shader.stage_count(), 0);
}

#[test]
fn test_shader_asset_implements_asset() {
    assert_eq!(ShaderAsset::asset_type(), AssetType::Shader);
    assert_eq!(ShaderAsset::asset_type_name(), "ShaderAsset");
    assert!(!ShaderAsset::extensions().is_empty());
}

#[test]
fn test_shader_asset_display() {
    let mut shader = ShaderAsset::with_name("test".to_string());
    shader.add_stage(ShaderSource::new(ShaderStage::Vertex, "12345".to_string()));

    let display = format!("{}", shader);
    assert!(display.contains("1 stages"));
    assert!(display.contains("test"));
}

#[test]
fn test_shader_asset_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ShaderAsset>();
}

// -------------------------------------------------------------------------
// ShaderFormat Tests
// -------------------------------------------------------------------------

#[test]
fn test_shader_format_name() {
    assert_eq!(ShaderFormat::Combined.name(), "combined");
    assert_eq!(ShaderFormat::SingleStage.name(), "single_stage");
    assert_eq!(ShaderFormat::Manifest.name(), "manifest");
}

#[test]
fn test_shader_format_default() {
    assert_eq!(ShaderFormat::default(), ShaderFormat::SingleStage);
}

#[test]
fn test_shader_format_display() {
    assert_eq!(format!("{}", ShaderFormat::Combined), "combined");
}

// -------------------------------------------------------------------------
// ShaderSettings Tests
// -------------------------------------------------------------------------

#[test]
fn test_shader_settings_default() {
    let settings = ShaderSettings::default();
    assert!(settings.validate);
    assert!(!settings.strip_comments);
    assert!(settings.defines.is_empty());
}

#[test]
fn test_shader_settings_clone() {
    let mut settings = ShaderSettings::default();
    settings.validate = false;
    let cloned = settings.clone();
    assert!(!cloned.validate);
}
