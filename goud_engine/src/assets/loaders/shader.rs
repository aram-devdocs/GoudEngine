//! Shader asset loader.
//!
//! This module provides asset types and loaders for GLSL shaders.
//! Supports vertex, fragment, geometry, and compute shaders.
//!
//! # Example
//!
//! ```no_run
//! use goud_engine::assets::{AssetServer, loaders::shader::ShaderLoader, loaders::shader::ShaderAsset};
//!
//! let mut server = AssetServer::new();
//! server.register_loader(ShaderLoader::default());
//!
//! // Load a complete shader program
//! let handle = server.load::<ShaderAsset>("shaders/basic.shader");
//! ```
use crate::assets::{Asset, AssetLoadError, AssetLoader, AssetType, LoadContext};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

// =============================================================================
// ShaderStage
// =============================================================================

/// Type of shader stage.
///
/// Represents the different programmable stages of the graphics pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum ShaderStage {
    /// Vertex shader - processes vertex data
    #[default]
    Vertex = 0,
    /// Fragment (pixel) shader - processes fragments
    Fragment = 1,
    /// Geometry shader - generates geometry from primitives
    Geometry = 2,
    /// Compute shader - general-purpose GPU computation
    Compute = 3,
    /// Tessellation control shader
    TessellationControl = 4,
    /// Tessellation evaluation shader
    TessellationEvaluation = 5,
}

impl ShaderStage {
    /// Returns all shader stages.
    pub fn all() -> &'static [ShaderStage] {
        &[
            ShaderStage::Vertex,
            ShaderStage::Fragment,
            ShaderStage::Geometry,
            ShaderStage::Compute,
            ShaderStage::TessellationControl,
            ShaderStage::TessellationEvaluation,
        ]
    }

    /// Returns the count of shader stages.
    pub fn count() -> usize {
        6
    }

    /// Returns the stage name.
    pub fn name(&self) -> &'static str {
        match self {
            ShaderStage::Vertex => "vertex",
            ShaderStage::Fragment => "fragment",
            ShaderStage::Geometry => "geometry",
            ShaderStage::Compute => "compute",
            ShaderStage::TessellationControl => "tessellation_control",
            ShaderStage::TessellationEvaluation => "tessellation_evaluation",
        }
    }

    /// Returns the file extension for this stage.
    pub fn extension(&self) -> &'static str {
        match self {
            ShaderStage::Vertex => "vert",
            ShaderStage::Fragment => "frag",
            ShaderStage::Geometry => "geom",
            ShaderStage::Compute => "comp",
            ShaderStage::TessellationControl => "tesc",
            ShaderStage::TessellationEvaluation => "tese",
        }
    }

    /// Returns the GLSL directive for this stage.
    pub fn glsl_directive(&self) -> &'static str {
        match self {
            ShaderStage::Vertex => "VERTEX",
            ShaderStage::Fragment => "FRAGMENT",
            ShaderStage::Geometry => "GEOMETRY",
            ShaderStage::Compute => "COMPUTE",
            ShaderStage::TessellationControl => "TESSELLATION_CONTROL",
            ShaderStage::TessellationEvaluation => "TESSELLATION_EVALUATION",
        }
    }

    /// Parses a shader stage from an extension.
    pub fn from_extension(ext: &str) -> Option<ShaderStage> {
        match ext.to_lowercase().as_str() {
            "vert" | "vs" => Some(ShaderStage::Vertex),
            "frag" | "fs" => Some(ShaderStage::Fragment),
            "geom" | "gs" => Some(ShaderStage::Geometry),
            "comp" | "cs" => Some(ShaderStage::Compute),
            "tesc" => Some(ShaderStage::TessellationControl),
            "tese" => Some(ShaderStage::TessellationEvaluation),
            _ => None,
        }
    }

    /// Parses a shader stage from a directive.
    pub fn from_directive(directive: &str) -> Option<ShaderStage> {
        match directive.to_uppercase().as_str() {
            "VERTEX" => Some(ShaderStage::Vertex),
            "FRAGMENT" => Some(ShaderStage::Fragment),
            "GEOMETRY" => Some(ShaderStage::Geometry),
            "COMPUTE" => Some(ShaderStage::Compute),
            "TESSELLATION_CONTROL" => Some(ShaderStage::TessellationControl),
            "TESSELLATION_EVALUATION" => Some(ShaderStage::TessellationEvaluation),
            _ => None,
        }
    }
}

impl fmt::Display for ShaderStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// ShaderSource
// =============================================================================

/// Source code for a single shader stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderSource {
    /// The shader stage this source is for.
    pub stage: ShaderStage,
    /// The GLSL source code.
    pub source: String,
    /// The GLSL version (e.g., "330 core").
    pub version: String,
}

impl ShaderSource {
    /// Creates a new shader source.
    pub fn new(stage: ShaderStage, source: String) -> Self {
        let version = Self::extract_version(&source);
        Self {
            stage,
            source,
            version,
        }
    }

    /// Extracts the GLSL version from source code.
    fn extract_version(source: &str) -> String {
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#version") {
                return trimmed
                    .strip_prefix("#version")
                    .unwrap_or("")
                    .trim()
                    .to_string();
            }
        }
        "330 core".to_string() // Default to GLSL 330
    }

    /// Returns the byte size of the source code.
    pub fn size_bytes(&self) -> usize {
        self.source.len()
    }

    /// Returns the line count.
    pub fn line_count(&self) -> usize {
        self.source.lines().count()
    }

    /// Checks if source is empty.
    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    /// Validates the source code for common errors.
    pub fn validate(&self) -> Result<(), String> {
        if self.source.is_empty() {
            return Err("Shader source is empty".to_string());
        }

        // Check for version directive
        if !self.source.contains("#version") {
            return Err("Missing #version directive".to_string());
        }

        // Check for main function
        if !self.source.contains("void main()") && !self.source.contains("void main(") {
            return Err("Missing main() function".to_string());
        }

        Ok(())
    }
}

impl fmt::Display for ShaderSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ShaderSource({}, {} lines, version: {})",
            self.stage,
            self.line_count(),
            self.version
        )
    }
}

// =============================================================================
// ShaderAsset
// =============================================================================

/// A loaded shader asset containing one or more shader stages.
///
/// `ShaderAsset` can represent:
/// - A single shader stage (.vert, .frag, etc.)
/// - A complete shader program (.shader with multiple stages)
/// - A combined shader file with stage directives
///
/// # Example
///
/// ```
/// use goud_engine::assets::loaders::{ShaderAsset, ShaderStage, ShaderSource};
///
/// let vertex_source = ShaderSource::new(
///     ShaderStage::Vertex,
///     "#version 330 core\nvoid main() {}".to_string(),
/// );
///
/// let mut shader = ShaderAsset::new();
/// shader.add_stage(vertex_source);
///
/// assert!(shader.has_stage(ShaderStage::Vertex));
/// assert_eq!(shader.stage_count(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderAsset {
    /// Shader sources indexed by stage.
    stages: HashMap<ShaderStage, ShaderSource>,
    /// Optional shader name.
    name: Option<String>,
    /// Shader defines/macros.
    defines: HashMap<String, String>,
}

impl ShaderAsset {
    /// Creates a new empty shader asset.
    pub fn new() -> Self {
        Self {
            stages: HashMap::new(),
            name: None,
            defines: HashMap::new(),
        }
    }

    /// Creates a shader asset with a name.
    pub fn with_name(name: String) -> Self {
        Self {
            stages: HashMap::new(),
            name: Some(name),
            defines: HashMap::new(),
        }
    }

    /// Adds a shader stage.
    pub fn add_stage(&mut self, source: ShaderSource) {
        self.stages.insert(source.stage, source);
    }

    /// Removes a shader stage.
    pub fn remove_stage(&mut self, stage: ShaderStage) -> Option<ShaderSource> {
        self.stages.remove(&stage)
    }

    /// Gets a shader stage.
    pub fn get_stage(&self, stage: ShaderStage) -> Option<&ShaderSource> {
        self.stages.get(&stage)
    }

    /// Gets a mutable shader stage.
    pub fn get_stage_mut(&mut self, stage: ShaderStage) -> Option<&mut ShaderSource> {
        self.stages.get_mut(&stage)
    }

    /// Checks if a stage exists.
    pub fn has_stage(&self, stage: ShaderStage) -> bool {
        self.stages.contains_key(&stage)
    }

    /// Returns the number of stages.
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Returns an iterator over all stages.
    pub fn stages(&self) -> impl Iterator<Item = (&ShaderStage, &ShaderSource)> {
        self.stages.iter()
    }

    /// Returns the shader name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the shader name.
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    /// Adds a preprocessor define.
    pub fn add_define(&mut self, name: String, value: String) {
        self.defines.insert(name, value);
    }

    /// Gets a preprocessor define.
    pub fn get_define(&self, name: &str) -> Option<&str> {
        self.defines.get(name).map(|s| s.as_str())
    }

    /// Returns all defines.
    pub fn defines(&self) -> &HashMap<String, String> {
        &self.defines
    }

    /// Validates all shader stages.
    pub fn validate(&self) -> Result<(), String> {
        if self.stages.is_empty() {
            return Err("Shader has no stages".to_string());
        }

        // Validate each stage
        for (stage, source) in &self.stages {
            source
                .validate()
                .map_err(|e| format!("{} shader: {}", stage.name(), e))?;
        }

        // Check for required stages in graphics pipelines
        if self.has_stage(ShaderStage::Compute) {
            // Compute shaders are standalone
            if self.stage_count() > 1 {
                return Err("Compute shaders cannot be combined with other stages".to_string());
            }
        } else {
            // Graphics pipeline requires vertex + fragment at minimum
            if !self.has_stage(ShaderStage::Vertex) {
                return Err("Graphics shader missing vertex stage".to_string());
            }
            if !self.has_stage(ShaderStage::Fragment) {
                return Err("Graphics shader missing fragment stage".to_string());
            }
        }

        Ok(())
    }

    /// Returns total source code size in bytes.
    pub fn total_size_bytes(&self) -> usize {
        self.stages.values().map(|s| s.size_bytes()).sum()
    }

    /// Checks if this is a compute shader.
    pub fn is_compute_shader(&self) -> bool {
        self.has_stage(ShaderStage::Compute) && self.stage_count() == 1
    }

    /// Checks if this is a graphics shader.
    pub fn is_graphics_shader(&self) -> bool {
        !self.is_compute_shader()
            && self.has_stage(ShaderStage::Vertex)
            && self.has_stage(ShaderStage::Fragment)
    }
}

impl Default for ShaderAsset {
    fn default() -> Self {
        Self::new()
    }
}

impl Asset for ShaderAsset {
    fn asset_type_name() -> &'static str {
        "ShaderAsset"
    }

    fn asset_type() -> AssetType {
        AssetType::Shader
    }

    fn extensions() -> &'static [&'static str] {
        &[
            "shader", "glsl", "vert", "frag", "geom", "comp", "tesc", "tese", "vs", "fs", "gs",
            "cs",
        ]
    }
}

impl fmt::Display for ShaderAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ShaderAsset({} stages, {} bytes",
            self.stage_count(),
            self.total_size_bytes()
        )?;
        if let Some(name) = &self.name {
            write!(f, ", name: {}", name)?;
        }
        write!(f, ")")
    }
}

// =============================================================================
// ShaderFormat
// =============================================================================

/// Shader file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum ShaderFormat {
    /// Combined shader file with stage directives (e.g., #pragma stage vertex)
    Combined,
    /// Single-stage shader file (.vert, .frag, etc.)
    #[default]
    SingleStage,
    /// Separate files referenced in a .shader manifest
    Manifest,
}

impl ShaderFormat {
    /// Returns the format name as a lowercase string.
    pub fn name(&self) -> &'static str {
        match self {
            ShaderFormat::Combined => "combined",
            ShaderFormat::SingleStage => "single_stage",
            ShaderFormat::Manifest => "manifest",
        }
    }
}

impl fmt::Display for ShaderFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// ShaderLoader
// =============================================================================

/// Settings for shader loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderSettings {
    /// Validate shader syntax (default: true)
    pub validate: bool,
    /// Strip comments from source (default: false)
    pub strip_comments: bool,
    /// Preprocessor defines to add
    pub defines: HashMap<String, String>,
}

impl Default for ShaderSettings {
    fn default() -> Self {
        Self {
            validate: true,
            strip_comments: false,
            defines: HashMap::new(),
        }
    }
}

/// Loader for shader assets.
///
/// Supports multiple shader formats:
/// - Single-stage files: `shader.vert`, `shader.frag`, etc.
/// - Combined files: Single file with `#pragma stage <stage>` directives
/// - Manifest files: `.shader` files referencing other shader files
#[derive(Debug, Clone)]
pub struct ShaderLoader {
    settings: ShaderSettings,
}

impl ShaderLoader {
    /// Creates a new shader loader with default settings.
    pub fn new() -> Self {
        Self {
            settings: ShaderSettings::default(),
        }
    }

    /// Creates a shader loader with custom settings.
    pub fn with_settings(settings: ShaderSettings) -> Self {
        Self { settings }
    }

    /// Parses a single-stage shader file.
    fn parse_single_stage(
        &self,
        source: &str,
        stage: ShaderStage,
    ) -> Result<ShaderAsset, AssetLoadError> {
        let mut shader = ShaderAsset::new();
        let shader_source = ShaderSource::new(stage, source.to_string());

        if self.settings.validate {
            shader_source
                .validate()
                .map_err(AssetLoadError::decode_failed)?;
        }

        shader.add_stage(shader_source);

        // Add user-defined preprocessor defines
        for (key, value) in &self.settings.defines {
            shader.add_define(key.clone(), value.clone());
        }

        Ok(shader)
    }

    /// Parses a combined shader file with stage directives.
    fn parse_combined(&self, source: &str) -> Result<ShaderAsset, AssetLoadError> {
        let mut shader = ShaderAsset::new();
        let mut current_stage: Option<ShaderStage> = None;
        let mut current_source = String::new();

        for line in source.lines() {
            let trimmed = line.trim();

            // Check for stage directive: #pragma stage <stage>
            if trimmed.starts_with("#pragma stage") || trimmed.starts_with("#pragma STAGE") {
                // Save previous stage if exists
                if let Some(stage) = current_stage {
                    let shader_source = ShaderSource::new(stage, current_source.clone());
                    if self.settings.validate {
                        shader_source
                            .validate()
                            .map_err(AssetLoadError::decode_failed)?;
                    }
                    shader.add_stage(shader_source);
                    current_source.clear();
                }

                // Parse new stage
                let stage_name = trimmed
                    .split_whitespace()
                    .nth(2)
                    .ok_or_else(|| AssetLoadError::decode_failed("Missing stage name"))?;
                current_stage = Some(ShaderStage::from_directive(stage_name).ok_or_else(|| {
                    AssetLoadError::decode_failed(format!("Unknown shader stage: {}", stage_name))
                })?);
            } else {
                // Add line to current source
                current_source.push_str(line);
                current_source.push('\n');
            }
        }

        // Save final stage
        if let Some(stage) = current_stage {
            let shader_source = ShaderSource::new(stage, current_source);
            if self.settings.validate {
                shader_source
                    .validate()
                    .map_err(AssetLoadError::decode_failed)?;
            }
            shader.add_stage(shader_source);
        } else {
            return Err(AssetLoadError::decode_failed("No shader stages found"));
        }

        // Add user-defined preprocessor defines
        for (key, value) in &self.settings.defines {
            shader.add_define(key.clone(), value.clone());
        }

        Ok(shader)
    }

    /// Detects the shader format from file content and extension.
    fn detect_format(&self, source: &str, extension: &str) -> ShaderFormat {
        // Check for stage directives
        if source.contains("#pragma stage") || source.contains("#pragma STAGE") {
            return ShaderFormat::Combined;
        }

        // Check if it's a known single-stage extension
        if ShaderStage::from_extension(extension).is_some() {
            return ShaderFormat::SingleStage;
        }

        // Default to combined for .shader and .glsl files
        ShaderFormat::Combined
    }
}

impl Default for ShaderLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLoader for ShaderLoader {
    type Asset = ShaderAsset;
    type Settings = ShaderSettings;

    fn extensions(&self) -> &[&str] {
        ShaderAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        // Decode UTF-8
        let source = std::str::from_utf8(bytes)
            .map_err(|e| AssetLoadError::decode_failed(format!("Invalid UTF-8: {}", e)))?;

        // Get file extension
        let extension = context
            .extension()
            .ok_or_else(|| AssetLoadError::decode_failed("Missing file extension"))?;

        // Detect format
        let format = self.detect_format(source, extension);

        let mut shader = match format {
            ShaderFormat::SingleStage => {
                let stage = ShaderStage::from_extension(extension).ok_or_else(|| {
                    AssetLoadError::unsupported_format(format!(
                        "Unknown shader extension: {}",
                        extension
                    ))
                })?;
                self.parse_single_stage(source, stage)?
            }
            ShaderFormat::Combined => self.parse_combined(source)?,
            ShaderFormat::Manifest => {
                // Manifest loading would require file system access
                return Err(AssetLoadError::unsupported_format(
                    "Manifest format not yet implemented",
                ));
            }
        };

        // Set name from file stem
        if let Some(file_name) = context.file_name() {
            if let Some(stem) = Path::new(file_name).file_stem() {
                shader.set_name(stem.to_string_lossy().to_string());
            }
        }

        // Validate complete shader only for combined shaders
        // Single-stage shaders are validated individually in parse_single_stage
        if self.settings.validate && format == ShaderFormat::Combined {
            shader.validate().map_err(AssetLoadError::decode_failed)?;
        }

        Ok(shader)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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

    // -------------------------------------------------------------------------
    // Thread Safety Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_shader_stage_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ShaderStage>();
    }

    #[test]
    fn test_shader_source_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ShaderSource>();
    }

    #[test]
    fn test_shader_asset_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ShaderAsset>();
    }

    #[test]
    fn test_shader_loader_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ShaderLoader>();
    }
}
