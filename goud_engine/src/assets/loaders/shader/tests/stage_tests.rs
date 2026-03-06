//! Tests for [`ShaderStage`] and [`ShaderSource`].

use crate::assets::loaders::shader::{ShaderSource, ShaderStage};

// -------------------------------------------------------------------------
// ShaderStage Tests
// -------------------------------------------------------------------------

#[test]
fn test_shader_stage_all() {
    let stages = ShaderStage::all();
    assert_eq!(stages.len(), 6);
    assert_eq!(ShaderStage::count(), 6);
}

#[test]
fn test_shader_stage_names() {
    assert_eq!(ShaderStage::Vertex.name(), "vertex");
    assert_eq!(ShaderStage::Fragment.name(), "fragment");
    assert_eq!(ShaderStage::Geometry.name(), "geometry");
    assert_eq!(ShaderStage::Compute.name(), "compute");
}

#[test]
fn test_shader_stage_extensions() {
    assert_eq!(ShaderStage::Vertex.extension(), "vert");
    assert_eq!(ShaderStage::Fragment.extension(), "frag");
    assert_eq!(ShaderStage::Geometry.extension(), "geom");
    assert_eq!(ShaderStage::Compute.extension(), "comp");
}

#[test]
fn test_shader_stage_from_extension() {
    assert_eq!(
        ShaderStage::from_extension("vert"),
        Some(ShaderStage::Vertex)
    );
    assert_eq!(
        ShaderStage::from_extension("frag"),
        Some(ShaderStage::Fragment)
    );
    assert_eq!(
        ShaderStage::from_extension("VERT"),
        Some(ShaderStage::Vertex)
    );
    assert_eq!(ShaderStage::from_extension("unknown"), None);
}

#[test]
fn test_shader_stage_from_directive() {
    assert_eq!(
        ShaderStage::from_directive("VERTEX"),
        Some(ShaderStage::Vertex)
    );
    assert_eq!(
        ShaderStage::from_directive("fragment"),
        Some(ShaderStage::Fragment)
    );
    assert_eq!(ShaderStage::from_directive("unknown"), None);
}

#[test]
fn test_shader_stage_default() {
    assert_eq!(ShaderStage::default(), ShaderStage::Vertex);
}

#[test]
fn test_shader_stage_display() {
    assert_eq!(format!("{}", ShaderStage::Vertex), "vertex");
    assert_eq!(format!("{}", ShaderStage::Fragment), "fragment");
}

#[test]
fn test_shader_stage_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ShaderStage>();
}

// -------------------------------------------------------------------------
// ShaderSource Tests
// -------------------------------------------------------------------------

#[test]
fn test_shader_source_new() {
    let source = ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    );
    assert_eq!(source.stage, ShaderStage::Vertex);
    assert_eq!(source.version, "330 core");
}

#[test]
fn test_shader_source_extract_version() {
    let source = "#version 450\nvoid main() {}";
    let shader = ShaderSource::new(ShaderStage::Vertex, source.to_string());
    assert_eq!(shader.version, "450");
}

#[test]
fn test_shader_source_default_version() {
    let source = "void main() {}";
    let shader = ShaderSource::new(ShaderStage::Vertex, source.to_string());
    assert_eq!(shader.version, "330 core");
}

#[test]
fn test_shader_source_size_bytes() {
    let source = ShaderSource::new(ShaderStage::Vertex, "hello".to_string());
    assert_eq!(source.size_bytes(), 5);
}

#[test]
fn test_shader_source_line_count() {
    let source = ShaderSource::new(ShaderStage::Vertex, "line1\nline2\nline3".to_string());
    assert_eq!(source.line_count(), 3);
}

#[test]
fn test_shader_source_is_empty() {
    let empty = ShaderSource::new(ShaderStage::Vertex, String::new());
    assert!(empty.is_empty());

    let non_empty = ShaderSource::new(ShaderStage::Vertex, "x".to_string());
    assert!(!non_empty.is_empty());
}

#[test]
fn test_shader_source_validate_empty() {
    let source = ShaderSource::new(ShaderStage::Vertex, String::new());
    assert!(source.validate().is_err());
}

#[test]
fn test_shader_source_validate_no_version() {
    let source = ShaderSource::new(ShaderStage::Vertex, "void main() {}".to_string());
    assert!(source.validate().is_err());
}

#[test]
fn test_shader_source_validate_no_main() {
    let source = ShaderSource::new(ShaderStage::Vertex, "#version 330\n".to_string());
    assert!(source.validate().is_err());
}

#[test]
fn test_shader_source_validate_success() {
    let source = ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330 core\nvoid main() {}".to_string(),
    );
    assert!(source.validate().is_ok());
}

#[test]
fn test_shader_source_display() {
    let source = ShaderSource::new(
        ShaderStage::Vertex,
        "#version 330\nline2\nline3".to_string(),
    );
    let display = format!("{}", source);
    assert!(display.contains("vertex"));
    assert!(display.contains("3 lines"));
}

#[test]
fn test_shader_source_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ShaderSource>();
}
