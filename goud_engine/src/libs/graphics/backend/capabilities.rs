//! Backend capability and information types.
//!
//! This module defines [`BackendCapabilities`] and [`BackendInfo`], which describe
//! what features a render backend supports and identify the underlying implementation.

/// Capabilities supported by a render backend.
///
/// Different graphics APIs support different feature sets. This struct
/// describes what features the current backend supports, allowing the
/// engine to gracefully degrade or choose alternative code paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendCapabilities {
    /// Maximum number of texture units that can be bound simultaneously
    pub max_texture_units: u32,

    /// Maximum texture size (width or height) in pixels
    pub max_texture_size: u32,

    /// Maximum number of vertex attributes
    pub max_vertex_attributes: u32,

    /// Maximum uniform buffer size in bytes
    pub max_uniform_buffer_size: u32,

    /// Whether instanced rendering is supported
    pub supports_instancing: bool,

    /// Whether compute shaders are supported
    pub supports_compute_shaders: bool,

    /// Whether geometry shaders are supported
    pub supports_geometry_shaders: bool,

    /// Whether tessellation shaders are supported
    pub supports_tessellation: bool,

    /// Whether multisampling (MSAA) is supported
    pub supports_multisampling: bool,

    /// Whether anisotropic filtering is supported
    pub supports_anisotropic_filtering: bool,
}

impl Default for BackendCapabilities {
    fn default() -> Self {
        // Conservative defaults for OpenGL 3.3 Core
        Self {
            max_texture_units: 16,
            max_texture_size: 8192,
            max_vertex_attributes: 16,
            max_uniform_buffer_size: 16384,
            supports_instancing: true,
            supports_compute_shaders: false,
            supports_geometry_shaders: false,
            supports_tessellation: false,
            supports_multisampling: true,
            supports_anisotropic_filtering: false,
        }
    }
}

/// Information about the render backend implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendInfo {
    /// Backend name (e.g., "OpenGL", "Vulkan", "Metal")
    pub name: &'static str,

    /// Backend version string (e.g., "OpenGL 3.3 Core")
    pub version: String,

    /// Vendor (e.g., "NVIDIA", "AMD", "Intel")
    pub vendor: String,

    /// Renderer name (e.g., "GeForce GTX 1080")
    pub renderer: String,

    /// Supported capabilities
    pub capabilities: BackendCapabilities,
}
