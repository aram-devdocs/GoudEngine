//! Type-safe configuration for the 3D renderer.
//!
//! Every optimization is **configurable and opt-out**.  Game developers control
//! behavior via [`Render3DConfig`] which propagates through the renderer.
//! Defaults are sensible for the common case; everything is overridable.

/// Top-level 3D renderer configuration.
#[derive(Debug, Clone)]
pub struct Render3DConfig {
    /// Frustum culling settings.
    pub frustum_culling: FrustumCullingConfig,
    /// Spatial index settings used to accelerate frustum culling.
    pub spatial_index: SpatialIndexConfig,
    /// Draw call batching and instancing settings.
    pub batching: BatchingConfig,
    /// Skeletal animation skinning settings.
    pub skinning: SkinningConfig,
    /// Shadow mapping settings.
    pub shadows: ShadowConfig,
    /// Fallback RGBA color used when a mesh has no assigned material (default: light gray).
    pub default_material_color: [f32; 4],
}

/// Spatial index configuration. The renderer uses a sparse uniform grid to
/// shrink the per-frame frustum-cull candidate set from "every scene object"
/// down to "objects whose grid cell touches the frustum AABB".
#[derive(Debug, Clone)]
pub struct SpatialIndexConfig {
    /// Whether the spatial index is consulted during frustum culling
    /// (default: `true`). When `false` the renderer falls back to a linear
    /// scan over every registered object — same behavior as before #678.
    pub enabled: bool,
    /// Grid cell size in world units (default: `32.0`). Cells smaller than
    /// `0.5` are clamped up to keep grid coordinates finite. Tune this for
    /// scenes where most objects fit inside one cell.
    ///
    /// Note: changing this at runtime forces a rebuild of the spatial index
    /// over every registered scene object. That is cheap for small scenes
    /// but is `O(n)` work over the whole `objects` registry, so prefer to
    /// set it once at startup for large scenes.
    pub cell_size: f32,
}

impl Default for SpatialIndexConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cell_size: 32.0,
        }
    }
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
    /// Minimum instances required to use instanced rendering (default: `2`).
    pub min_instances_for_batching: usize,
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            static_batching_enabled: true,
            instancing_enabled: true,
            material_sorting_enabled: true,
            max_static_batch_vertices: 50_000,
            min_instances_for_batching: 2,
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
    /// Sample rate for pre-baked animation cache in frames per second (default: `30.0`).
    pub baked_animation_sample_rate: f32,
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
            baked_animation_sample_rate: 30.0,
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
    /// Shadow intensity multiplier (default: `0.65`).
    ///
    /// Controls how dark shadows appear. `1.0` = fully black shadows,
    /// `0.0` = no shadow darkening.
    pub shadow_strength: f32,
    /// Vertex count threshold above which shadows auto-disable (default: `10_000`).
    pub auto_disable_vertex_threshold: usize,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            map_size: 256,
            bias: 0.005,
            shadow_strength: 0.65,
            auto_disable_vertex_threshold: 10_000,
        }
    }
}

impl Default for Render3DConfig {
    fn default() -> Self {
        Self {
            frustum_culling: FrustumCullingConfig::default(),
            spatial_index: SpatialIndexConfig::default(),
            batching: BatchingConfig::default(),
            skinning: SkinningConfig::default(),
            shadows: ShadowConfig::default(),
            default_material_color: [0.8, 0.8, 0.8, 1.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_render3d_config_values() {
        let config = Render3DConfig::default();

        // Frustum culling defaults
        assert!(config.frustum_culling.enabled);
        assert!(
            (config.frustum_culling.fov_degrees - 45.0).abs() < f32::EPSILON,
            "Default FOV should be 45.0"
        );
        assert!(
            (config.frustum_culling.near_plane - 0.1).abs() < f32::EPSILON,
            "Default near plane should be 0.1"
        );
        assert!(
            (config.frustum_culling.far_plane - 1000.0).abs() < f32::EPSILON,
            "Default far plane should be 1000.0"
        );

        // Batching defaults
        assert!(config.batching.static_batching_enabled);
        assert!(config.batching.instancing_enabled);
        assert!(config.batching.material_sorting_enabled);
        assert_eq!(config.batching.max_static_batch_vertices, 50_000);
        assert_eq!(config.batching.min_instances_for_batching, 2);

        // Skinning defaults
        assert_eq!(config.skinning.mode, SkinningMode::Gpu);
        assert_eq!(config.skinning.max_bones_per_mesh, 128);
        assert!(config.skinning.animation_lod_enabled);
        assert!(
            (config.skinning.animation_lod_distance - 50.0).abs() < f32::EPSILON,
            "Default animation LOD distance should be 50.0"
        );
        assert!(
            (config.skinning.animation_lod_skip_distance - 100.0).abs() < f32::EPSILON,
            "Default animation LOD skip distance should be 100.0"
        );
        assert!(config.skinning.shared_animation_eval);
        assert!(
            (config.skinning.baked_animation_sample_rate - 30.0).abs() < f32::EPSILON,
            "Default baked animation sample rate should be 30.0"
        );

        // Shadow defaults
        assert!(!config.shadows.enabled);
        assert_eq!(config.shadows.map_size, 256);
        assert!(
            (config.shadows.bias - 0.005).abs() < f32::EPSILON,
            "Default shadow bias should be 0.005"
        );
        assert_eq!(config.shadows.auto_disable_vertex_threshold, 10_000);

        // Material color default
        assert_eq!(config.default_material_color, [0.8, 0.8, 0.8, 1.0]);
    }

    #[test]
    fn test_spatial_index_config_default() {
        let si = SpatialIndexConfig::default();
        assert!(si.enabled);
        assert!((si.cell_size - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frustum_culling_config_default() {
        let fc = FrustumCullingConfig::default();
        assert!(fc.enabled);
        assert!((fc.near_plane - 0.1).abs() < f32::EPSILON);
        assert!((fc.far_plane - 1000.0).abs() < f32::EPSILON);
        assert!((fc.fov_degrees - 45.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_batching_config_default() {
        let bc = BatchingConfig::default();
        assert!(bc.static_batching_enabled);
        assert!(bc.instancing_enabled);
        assert!(bc.material_sorting_enabled);
        assert_eq!(bc.max_static_batch_vertices, 50_000);
        assert_eq!(bc.min_instances_for_batching, 2);
    }

    #[test]
    fn test_skinning_config_default() {
        let sc = SkinningConfig::default();
        assert_eq!(sc.mode, SkinningMode::Gpu);
        assert_eq!(sc.max_bones_per_mesh, 128);
        assert!(sc.animation_lod_enabled);
        assert!((sc.animation_lod_distance - 50.0).abs() < f32::EPSILON);
        assert!((sc.animation_lod_skip_distance - 100.0).abs() < f32::EPSILON);
        assert!(sc.shared_animation_eval);
        assert!((sc.baked_animation_sample_rate - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_shadow_config_default() {
        let sc = ShadowConfig::default();
        assert!(!sc.enabled);
        assert_eq!(sc.map_size, 256);
        assert!((sc.bias - 0.005).abs() < f32::EPSILON);
        assert_eq!(sc.auto_disable_vertex_threshold, 10_000);
    }

    #[test]
    fn test_skinning_mode_equality() {
        assert_eq!(SkinningMode::Cpu, SkinningMode::Cpu);
        assert_eq!(SkinningMode::Gpu, SkinningMode::Gpu);
        assert_ne!(SkinningMode::Cpu, SkinningMode::Gpu);
    }

    #[test]
    fn test_render3d_config_clone() {
        let config = Render3DConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.default_material_color, config.default_material_color);
        assert_eq!(
            cloned.frustum_culling.fov_degrees,
            config.frustum_culling.fov_degrees
        );
        assert_eq!(
            cloned.batching.min_instances_for_batching,
            config.batching.min_instances_for_batching
        );
        assert_eq!(cloned.skinning.mode, config.skinning.mode);
        assert_eq!(cloned.shadows.map_size, config.shadows.map_size);
    }

    #[test]
    fn test_render3d_config_mutation() {
        let mut config = Render3DConfig::default();

        // Mutate and verify changes stick.
        config.frustum_culling.enabled = false;
        config.frustum_culling.fov_degrees = 90.0;
        config.batching.min_instances_for_batching = 10;
        config.skinning.mode = SkinningMode::Cpu;
        config.shadows.enabled = true;
        config.shadows.map_size = 2048;
        config.default_material_color = [1.0, 0.0, 0.0, 1.0];

        assert!(!config.frustum_culling.enabled);
        assert!((config.frustum_culling.fov_degrees - 90.0).abs() < f32::EPSILON);
        assert_eq!(config.batching.min_instances_for_batching, 10);
        assert_eq!(config.skinning.mode, SkinningMode::Cpu);
        assert!(config.shadows.enabled);
        assert_eq!(config.shadows.map_size, 2048);
        assert_eq!(config.default_material_color, [1.0, 0.0, 0.0, 1.0]);
    }
}
