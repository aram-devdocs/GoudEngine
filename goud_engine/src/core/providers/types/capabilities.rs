//! Provider capability reports.

/// Capabilities reported by a render provider.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct RenderCapabilities {
    /// Maximum number of texture units available.
    pub max_texture_units: u32,
    /// Maximum texture dimension (width or height).
    pub max_texture_size: u32,
    /// Whether hardware instancing is supported.
    pub supports_instancing: bool,
    /// Whether compute shaders are supported.
    pub supports_compute: bool,
    /// Whether multisample anti-aliasing is supported.
    pub supports_msaa: bool,
}

/// Capabilities reported by a physics provider.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct PhysicsCapabilities {
    /// Whether continuous collision detection is supported.
    pub supports_continuous_collision: bool,
    /// Whether joints are supported.
    pub supports_joints: bool,
    /// Maximum number of physics bodies.
    pub max_bodies: u32,
}

/// Capabilities reported by an audio provider.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct AudioCapabilities {
    /// Whether spatial/3D audio is supported.
    pub supports_spatial: bool,
    /// Maximum number of simultaneous audio channels.
    pub max_channels: u32,
}
