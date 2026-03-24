//! Type-safe configuration for the 3D renderer.
//!
//! Every optimization is **configurable and opt-out**.  Game developers control
//! behavior via [`Render3DConfig`] which propagates through the renderer.
//! Defaults are sensible for the common case; everything is overridable.

/// Top-level 3D renderer configuration.
#[derive(Debug, Clone, Default)]
pub struct Render3DConfig {
    /// Frustum culling settings.
    pub frustum_culling: FrustumCullingConfig,
    /// Draw call batching and instancing settings.
    pub batching: BatchingConfig,
    /// Skeletal animation skinning settings.
    pub skinning: SkinningConfig,
    /// Shadow mapping settings.
    pub shadows: ShadowConfig,
    /// Level-of-detail settings.
    pub lod: LodConfig,
    /// Performance monitoring settings.
    pub performance: PerformanceConfig,
}

/// Frustum culling configuration.
#[derive(Debug, Clone)]
pub struct FrustumCullingConfig {
    /// Whether frustum culling is enabled (default: `true`).
    pub enabled: bool,
    /// Near clipping plane distance (default: `0.1`).
    pub near_plane: f32,
    /// Far clipping plane distance (default: `1000.0`).
    pub far_plane: f32,
    /// Vertical field of view in degrees (default: `45.0`).
    pub fov_degrees: f32,
}

impl Default for FrustumCullingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            near_plane: 0.1,
            far_plane: 1000.0,
            fov_degrees: 45.0,
        }
    }
}

/// Draw call batching and instancing configuration.
#[derive(Debug, Clone)]
pub struct BatchingConfig {
    /// Whether static object batching is enabled (default: `true`).
    pub static_batching_enabled: bool,
    /// Whether GPU instancing is used for same-mesh objects (default: `true`).
    pub instancing_enabled: bool,
    /// Whether objects are sorted by material before drawing (default: `true`).
    pub material_sorting_enabled: bool,
    /// Maximum vertices in a single static batch (default: `50_000`).
    pub max_static_batch_vertices: usize,
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            static_batching_enabled: true,
            instancing_enabled: true,
            material_sorting_enabled: true,
            max_static_batch_vertices: 50_000,
        }
    }
}

/// Skeletal animation skinning configuration.
#[derive(Debug, Clone)]
pub struct SkinningConfig {
    /// Skinning mode (default: [`SkinningMode::Gpu`]).
    pub mode: SkinningMode,
    /// Maximum bones per mesh (default: `128`).
    pub max_bones_per_mesh: u32,
    /// Whether animation LOD is enabled (default: `true`).
    ///
    /// When enabled, distant animated models update at reduced rates.
    pub animation_lod_enabled: bool,
    /// Distance beyond which animation updates at half rate (default: `50.0`).
    pub animation_lod_distance: f32,
    /// Distance beyond which animation freezes (default: `100.0`).
    pub animation_lod_skip_distance: f32,
    /// Whether identical animation states are evaluated once and shared (default: `true`).
    pub shared_animation_eval: bool,
}

impl Default for SkinningConfig {
    fn default() -> Self {
        Self {
            mode: SkinningMode::Gpu,
            max_bones_per_mesh: 128,
            animation_lod_enabled: true,
            animation_lod_distance: 50.0,
            animation_lod_skip_distance: 100.0,
            shared_animation_eval: true,
        }
    }
}

/// Skinning execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinningMode {
    /// CPU vertex deformation with per-frame buffer re-upload.
    Cpu,
    /// GPU vertex shader skinning via storage buffer bone matrices.
    Gpu,
}

/// Shadow mapping configuration.
#[derive(Debug, Clone)]
pub struct ShadowConfig {
    /// Whether shadows are enabled (default: `false` — opt-in).
    pub enabled: bool,
    /// Shadow map resolution (default: `256`).
    pub map_size: u32,
    /// Depth bias to reduce shadow acne (default: `0.005`).
    pub bias: f32,
    /// Vertex count threshold above which shadows auto-disable (default: `10_000`).
    pub auto_disable_vertex_threshold: usize,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            map_size: 256,
            bias: 0.005,
            auto_disable_vertex_threshold: 10_000,
        }
    }
}

/// Level-of-detail configuration.
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Whether LOD is enabled (default: `false` — opt-in).
    pub enabled: bool,
    /// Game-defined distance thresholds for LOD transitions.
    pub distance_thresholds: Vec<f32>,
    /// Objects smaller than this pixel size are skipped (default: `2.0`).
    pub pixel_size_threshold: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            distance_thresholds: Vec::new(),
            pixel_size_threshold: 2.0,
        }
    }
}

/// Performance monitoring configuration.
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Whether render stats are collected (default: `true`).
    pub stats_enabled: bool,
    /// Optional cap on draw calls per frame (`None` = unlimited).
    pub max_draw_calls_per_frame: Option<u32>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            stats_enabled: true,
            max_draw_calls_per_frame: None,
        }
    }
}
