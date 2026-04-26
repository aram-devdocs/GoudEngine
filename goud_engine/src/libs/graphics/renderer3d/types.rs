//! Types for the 3D renderer: enums, config structs, camera, and scene objects.

use crate::libs::graphics::backend::BufferHandle;
pub use crate::libs::graphics::AntiAliasingMode;
use cgmath::{Deg, Matrix4, Rad, Vector3, Vector4};

// Re-export types that were split into sibling modules.
pub use super::material::{Material3D, MaterialType, PbrProperties};
pub use super::render_pass::{
    BloomPass, ColorGradePass, GaussianBlurPass, PostProcessPipeline, RenderPass,
};
pub use super::skinned_mesh::{Bone3D, Skeleton3D, SkinnedMesh3D, MAX_BONES, MAX_BONE_INFLUENCES};

/// Maximum number of lights supported
pub const MAX_LIGHTS: usize = 8;

// ============================================================================
// Enums
// ============================================================================

/// Type of 3D primitive
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum PrimitiveType {
    /// A cube primitive
    Cube = 0,
    /// A sphere primitive
    Sphere = 1,
    /// A plane primitive
    Plane = 2,
    /// A cylinder primitive
    Cylinder = 3,
}

/// Type of light source
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum LightType {
    /// Point light (emits in all directions)
    Point = 0,
    /// Directional light (parallel rays)
    Directional = 1,
    /// Spot light (cone of light)
    Spot = 2,
}

/// Grid render mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum GridRenderMode {
    /// Blend grid with scene
    Blend = 0,
    /// Overlap grid on scene
    Overlap = 1,
}

// ============================================================================
// Data structs
// ============================================================================

/// Primitive creation info
#[repr(C)]
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct PrimitiveCreateInfo {
    /// Type of primitive to create
    pub primitive_type: PrimitiveType,
    /// Width of the primitive
    pub width: f32,
    /// Height of the primitive
    pub height: f32,
    /// Depth of the primitive
    pub depth: f32,
    /// Number of segments (for spheres/cylinders)
    pub segments: u32,
    /// Texture ID to apply
    pub texture_id: u32,
}

/// Per-instance transform and color for instanced drawing.
#[derive(Debug, Clone)]
pub struct InstanceTransform {
    /// Instance position in world space.
    pub position: Vector3<f32>,
    /// Instance rotation (pitch, yaw, roll) in degrees.
    pub rotation: Vector3<f32>,
    /// Instance scale.
    pub scale: Vector3<f32>,
    /// Instance tint color.
    pub color: Vector4<f32>,
}

impl Default for InstanceTransform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            color: Vector4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

/// CPU-driven particle emitter configuration.
#[derive(Debug, Clone)]
pub struct ParticleEmitterConfig {
    /// Particles emitted per second.
    pub emission_rate: f32,
    /// Maximum live particles tracked by the emitter.
    pub max_particles: usize,
    /// Lifetime of each particle in seconds.
    pub lifetime: f32,
    /// Minimum launch velocity.
    pub velocity_min: Vector3<f32>,
    /// Maximum launch velocity.
    pub velocity_max: Vector3<f32>,
    /// Starting particle color.
    pub start_color: Vector4<f32>,
    /// Ending particle color.
    pub end_color: Vector4<f32>,
    /// Starting particle size.
    pub start_size: f32,
    /// Ending particle size.
    pub end_size: f32,
    /// Optional particle texture.
    pub texture_id: u32,
}

impl Default for ParticleEmitterConfig {
    fn default() -> Self {
        Self {
            emission_rate: 16.0,
            max_particles: 256,
            lifetime: 1.0,
            velocity_min: Vector3::new(-0.25, 1.0, -0.25),
            velocity_max: Vector3::new(0.25, 2.0, 0.25),
            start_color: Vector4::new(1.0, 0.6, 0.2, 1.0),
            end_color: Vector4::new(0.8, 0.1, 0.0, 0.0),
            start_size: 0.35,
            end_size: 0.05,
            texture_id: 0,
        }
    }
}

// `Renderer3DStats` lives in `super::stats` so that file stays under the
// repo's per-file line limit; re-exported here for backward compatibility.
pub use super::stats::Renderer3DStats;

/// Local-space bounding sphere for frustum culling.
#[derive(Debug, Clone, Copy)]
pub(in crate::libs::graphics::renderer3d) struct BoundingSphere {
    /// Center in object-local space (typically the AABB center).
    pub(in crate::libs::graphics::renderer3d) center: Vector3<f32>,
    /// Radius from center to the farthest vertex.
    pub(in crate::libs::graphics::renderer3d) radius: f32,
}

impl Default for BoundingSphere {
    fn default() -> Self {
        Self {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius: 0.0,
        }
    }
}

/// Compute a bounding sphere from a flat vertex buffer with the given stride.
pub(in crate::libs::graphics::renderer3d) fn compute_bounding_sphere(
    vertices: &[f32],
    floats_per_vertex: usize,
) -> BoundingSphere {
    let fpv = floats_per_vertex;
    let vert_count = vertices.len() / fpv;
    if vert_count == 0 {
        return BoundingSphere::default();
    }

    // Compute AABB center.
    let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
    for i in 0..vert_count {
        let base = i * fpv;
        let p = Vector3::new(vertices[base], vertices[base + 1], vertices[base + 2]);
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        min.z = min.z.min(p.z);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
        max.z = max.z.max(p.z);
    }
    let center = (min + max) * 0.5;

    // Compute radius as max distance from center.
    let mut radius_sq = 0.0f32;
    for i in 0..vert_count {
        let base = i * fpv;
        let p = Vector3::new(vertices[base], vertices[base + 1], vertices[base + 2]);
        let d = p - center;
        radius_sq = radius_sq.max(d.x * d.x + d.y * d.y + d.z * d.z);
    }

    BoundingSphere {
        center,
        radius: radius_sq.sqrt(),
    }
}

/// A 3D object in the scene
#[derive(Debug)]
pub(in crate::libs::graphics::renderer3d) struct Object3D {
    pub(in crate::libs::graphics::renderer3d) buffer: BufferHandle,
    pub(in crate::libs::graphics::renderer3d) vertex_count: i32,
    pub(in crate::libs::graphics::renderer3d) vertices: Vec<f32>,
    pub(in crate::libs::graphics::renderer3d) position: Vector3<f32>,
    pub(in crate::libs::graphics::renderer3d) rotation: Vector3<f32>,
    pub(in crate::libs::graphics::renderer3d) scale: Vector3<f32>,
    pub(in crate::libs::graphics::renderer3d) texture_id: u32,
    /// Local-space bounding sphere for frustum culling.
    pub(in crate::libs::graphics::renderer3d) bounds: BoundingSphere,
    /// Whether this object is static (transform never changes). Used for future
    /// static batching optimizations; material sorting already groups these.
    pub(in crate::libs::graphics::renderer3d) is_static: bool,
}

#[derive(Debug)]
pub(in crate::libs::graphics::renderer3d) struct InstancedMesh {
    pub(in crate::libs::graphics::renderer3d) mesh_buffer: BufferHandle,
    pub(in crate::libs::graphics::renderer3d) vertex_count: u32,
    pub(in crate::libs::graphics::renderer3d) instance_buffer: BufferHandle,
    pub(in crate::libs::graphics::renderer3d) instances: Vec<InstanceTransform>,
    pub(in crate::libs::graphics::renderer3d) texture_id: u32,
}

#[derive(Debug, Clone)]
pub(in crate::libs::graphics::renderer3d) struct Particle {
    pub(in crate::libs::graphics::renderer3d) position: Vector3<f32>,
    pub(in crate::libs::graphics::renderer3d) velocity: Vector3<f32>,
    pub(in crate::libs::graphics::renderer3d) age: f32,
    pub(in crate::libs::graphics::renderer3d) lifetime: f32,
}

#[derive(Debug)]
pub(in crate::libs::graphics::renderer3d) struct ParticleEmitter {
    pub(in crate::libs::graphics::renderer3d) position: Vector3<f32>,
    pub(in crate::libs::graphics::renderer3d) config: ParticleEmitterConfig,
    pub(in crate::libs::graphics::renderer3d) particles: Vec<Particle>,
    pub(in crate::libs::graphics::renderer3d) instance_buffer: BufferHandle,
    pub(in crate::libs::graphics::renderer3d) spawn_accumulator: f32,
    pub(in crate::libs::graphics::renderer3d) spawn_counter: u32,
}

/// A light in the scene
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct Light {
    /// Type of light
    pub light_type: LightType,
    /// Position in world space
    pub position: Vector3<f32>,
    /// Direction (for directional/spot lights)
    pub direction: Vector3<f32>,
    /// Light color (RGB)
    pub color: Vector3<f32>,
    /// Light intensity
    pub intensity: f32,
    /// Light range (for point/spot lights)
    pub range: f32,
    /// Spot light angle in degrees
    pub spot_angle: f32,
    /// Whether the light is enabled
    pub enabled: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Point,
            position: Vector3::new(0.0, 5.0, 0.0),
            direction: Vector3::new(0.0, -1.0, 0.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            range: 10.0,
            spot_angle: 45.0,
            enabled: true,
        }
    }
}

/// Grid configuration
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct GridConfig {
    /// Whether grid is enabled
    pub enabled: bool,
    /// Size of the grid
    pub size: f32,
    /// Number of divisions
    pub divisions: u32,
    /// Show XZ plane
    pub show_xz_plane: bool,
    /// Show XY plane
    pub show_xy_plane: bool,
    /// Show YZ plane
    pub show_yz_plane: bool,
    /// Render mode
    pub render_mode: GridRenderMode,
    /// Line color
    pub line_color: Vector3<f32>,
    /// Grid line opacity (default: `0.4`).
    pub alpha: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 20.0,
            divisions: 20,
            show_xz_plane: true,
            show_xy_plane: false,
            show_yz_plane: false,
            render_mode: GridRenderMode::Blend,
            line_color: Vector3::new(0.5, 0.5, 0.5),
            alpha: 0.4,
        }
    }
}

/// Skybox configuration
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct SkyboxConfig {
    /// Whether skybox is enabled
    pub enabled: bool,
    /// Skybox color (RGBA)
    pub color: Vector4<f32>,
}

impl Default for SkyboxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            color: Vector4::new(0.1, 0.1, 0.2, 1.0),
        }
    }
}

/// Fog calculation mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FogMode {
    /// Classic exponential fog. Visibility = exp(-(density * distance)^2).
    Exponential {
        /// Fog density factor.
        density: f32,
    },
    /// Linear fog with explicit start/end distances. Clear before start, fully fogged after end.
    Linear {
        /// Distance at which fog begins.
        start: f32,
        /// Distance at which fog is fully opaque.
        end: f32,
    },
}

/// Fog configuration
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct FogConfig {
    /// Whether fog is enabled
    pub enabled: bool,
    /// Fog color (RGB)
    pub color: Vector3<f32>,
    /// Fog mode (exponential or linear)
    pub mode: FogMode,
}

impl FogConfig {
    /// Returns 0 for exponential, 1 for linear (used as shader uniform).
    pub fn mode_int(&self) -> i32 {
        match self.mode {
            FogMode::Exponential { .. } => 0,
            FogMode::Linear { .. } => 1,
        }
    }
    /// Returns density (0.0 for linear mode).
    pub fn density(&self) -> f32 {
        match self.mode {
            FogMode::Exponential { density } => density,
            FogMode::Linear { .. } => 0.0,
        }
    }
    /// Returns fog start distance (0.0 for exponential mode).
    pub fn start(&self) -> f32 {
        match self.mode {
            FogMode::Exponential { .. } => 0.0,
            FogMode::Linear { start, .. } => start,
        }
    }
    /// Returns fog end distance (0.0 for exponential mode).
    pub fn end(&self) -> f32 {
        match self.mode {
            FogMode::Exponential { .. } => 0.0,
            FogMode::Linear { end, .. } => end,
        }
    }
}

impl Default for FogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Vector3::new(0.05, 0.05, 0.1),
            mode: FogMode::Exponential { density: 0.02 },
        }
    }
}

// ============================================================================
// Camera
// ============================================================================

/// 3D Camera
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct Camera3D {
    /// Camera position in world space
    pub position: Vector3<f32>,
    /// Camera rotation (pitch, yaw, roll) in degrees
    pub rotation: Vector3<f32>,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 10.0, -20.0),
            rotation: Vector3::new(-30.0, 0.0, 0.0),
        }
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;

impl Camera3D {
    /// Get the view matrix
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let pitch = Rad::from(Deg(self.rotation.x));
        let yaw = Rad::from(Deg(self.rotation.y));

        let cos_pitch = pitch.0.cos();
        let sin_pitch = pitch.0.sin();
        let cos_yaw = yaw.0.cos();
        let sin_yaw = yaw.0.sin();

        let forward = Vector3::new(sin_yaw * cos_pitch, sin_pitch, cos_yaw * cos_pitch);

        let target = self.position + forward;
        let up = Vector3::new(0.0, 1.0, 0.0);

        Matrix4::look_at_rh(
            cgmath::Point3::new(self.position.x, self.position.y, self.position.z),
            cgmath::Point3::new(target.x, target.y, target.z),
            up,
        )
    }
}
