//! Shader stage types and utilities.
//!
//! Defines [`ShaderStage`] and associated parsing/display helpers.

use std::fmt;

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
