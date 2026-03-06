//! Shader asset types.
//!
//! Defines [`ShaderSource`], [`ShaderAsset`], and [`ShaderFormat`].

use crate::assets::{Asset, AssetType};
use std::collections::HashMap;
use std::fmt;

use super::stage::ShaderStage;

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
