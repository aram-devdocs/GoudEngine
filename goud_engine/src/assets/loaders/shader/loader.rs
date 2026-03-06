//! Shader loader implementation.
//!
//! Defines [`ShaderSettings`] and [`ShaderLoader`] which implement the
//! [`AssetLoader`] trait for GLSL shader files.

use crate::assets::{asset::Asset, AssetLoadError, AssetLoader, LoadContext};
use std::collections::HashMap;
use std::path::Path;

use super::asset::{ShaderAsset, ShaderFormat, ShaderSource};
use super::stage::ShaderStage;

// =============================================================================
// ShaderSettings
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

// =============================================================================
// ShaderLoader
// =============================================================================

/// Loader for shader assets.
///
/// Supports multiple shader formats:
/// - Single-stage files: `shader.vert`, `shader.frag`, etc.
/// - Combined files: Single file with `#pragma stage <stage>` directives
/// - Manifest files: `.shader` files referencing other shader files
#[derive(Debug, Clone)]
pub struct ShaderLoader {
    pub(super) settings: ShaderSettings,
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
    pub(super) fn detect_format(&self, source: &str, extension: &str) -> ShaderFormat {
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
