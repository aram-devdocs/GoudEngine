//! Types for the 3D renderer: enums, config structs, camera, and scene objects.

use crate::libs::graphics::backend::BufferHandle;
pub use crate::libs::graphics::AntiAliasingMode;
use cgmath::{Deg, Matrix4, Rad, Vector3, Vector4};

// ============================================================================
// Constants
// ============================================================================

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

/// Last-frame renderer statistics exposed for tests and debugging.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Renderer3DStats {
    /// Total draw calls recorded by the renderer this frame.
    pub draw_calls: u32,
    /// Instanced draw calls recorded this frame.
    pub instanced_draw_calls: u32,
    /// Particle instanced draw calls recorded this frame.
    pub particle_draw_calls: u32,
    /// Number of instance records submitted this frame.
    pub active_instances: u32,
    /// Number of live particles submitted this frame.
    pub active_particles: u32,
}

/// A 3D object in the scene
#[derive(Debug)]
pub(super) struct Object3D {
    pub(super) buffer: BufferHandle,
    pub(super) vertex_count: i32,
    pub(super) vertices: Vec<f32>,
    pub(super) position: Vector3<f32>,
    pub(super) rotation: Vector3<f32>,
    pub(super) scale: Vector3<f32>,
    pub(super) texture_id: u32,
}

#[derive(Debug)]
pub(super) struct InstancedMesh {
    pub(super) mesh_buffer: BufferHandle,
    pub(super) vertex_count: u32,
    pub(super) instance_buffer: BufferHandle,
    pub(super) instances: Vec<InstanceTransform>,
    pub(super) texture_id: u32,
}

#[derive(Debug, Clone)]
pub(super) struct Particle {
    pub(super) position: Vector3<f32>,
    pub(super) velocity: Vector3<f32>,
    pub(super) age: f32,
    pub(super) lifetime: f32,
}

#[derive(Debug)]
pub(super) struct ParticleEmitter {
    pub(super) position: Vector3<f32>,
    pub(super) config: ParticleEmitterConfig,
    pub(super) particles: Vec<Particle>,
    pub(super) instance_buffer: BufferHandle,
    pub(super) spawn_accumulator: f32,
    pub(super) spawn_counter: u32,
}

/// A light in the scene
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
        }
    }
}

/// Skybox configuration
#[derive(Debug, Clone)]
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

/// Fog configuration
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct FogConfig {
    /// Whether fog is enabled
    pub enabled: bool,
    /// Fog color (RGB)
    pub color: Vector3<f32>,
    /// Fog density
    pub density: f32,
}

impl Default for FogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Vector3::new(0.05, 0.05, 0.1),
            density: 0.02,
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

// ============================================================================
// Material System
// ============================================================================

/// The type of material shading model.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialType {
    /// Phong shading model.
    Phong = 0,
    /// Physically based rendering model.
    Pbr = 1,
    /// Unlit / flat shading.
    Unlit = 2,
}

/// PBR (physically based rendering) material properties.
#[derive(Debug, Clone)]
pub struct PbrProperties {
    /// Base reflectivity of the surface (0.0 = dielectric, 1.0 = metal).
    pub metallic: f32,
    /// Surface roughness (0.0 = mirror, 1.0 = fully rough).
    pub roughness: f32,
    /// Ambient occlusion factor.
    pub ao: f32,
    /// Albedo texture ID (0 = none).
    pub albedo_map: u32,
    /// Normal map texture ID (0 = none).
    pub normal_map: u32,
    /// Metallic/roughness map texture ID (0 = none).
    pub metallic_roughness_map: u32,
}

impl Default for PbrProperties {
    fn default() -> Self {
        Self {
            metallic: 0.0,
            roughness: 0.5,
            ao: 1.0,
            albedo_map: 0,
            normal_map: 0,
            metallic_roughness_map: 0,
        }
    }
}

/// A 3D material describing how a surface is shaded.
#[derive(Debug, Clone)]
pub struct Material3D {
    /// Material shading model.
    pub material_type: MaterialType,
    /// Diffuse / albedo color (RGBA).
    pub color: cgmath::Vector4<f32>,
    /// Specular highlight intensity (Phong).
    pub shininess: f32,
    /// Optional PBR properties (only used when `material_type == Pbr`).
    pub pbr: PbrProperties,
}

impl Default for Material3D {
    fn default() -> Self {
        Self {
            material_type: MaterialType::Phong,
            color: cgmath::Vector4::new(0.8, 0.8, 0.8, 1.0),
            shininess: 32.0,
            pbr: PbrProperties::default(),
        }
    }
}

// ============================================================================
// Skeletal Mesh
// ============================================================================

/// Maximum bones per skeleton for GPU skinning.
pub const MAX_BONES: usize = 128;

/// Maximum bone influences per vertex.
pub const MAX_BONE_INFLUENCES: usize = 4;

/// A single bone in a skeleton hierarchy.
#[derive(Debug, Clone)]
pub struct Bone3D {
    /// Bone name for lookup.
    pub name: String,
    /// Index of the parent bone (-1 for root).
    pub parent_index: i32,
    /// Inverse bind-pose matrix (column-major 4x4).
    pub inverse_bind_matrix: [f32; 16],
}

/// A complete skeleton definition.
#[derive(Debug, Clone)]
pub struct Skeleton3D {
    /// Ordered list of bones.
    pub bones: Vec<Bone3D>,
}

impl Skeleton3D {
    /// Create a new empty skeleton.
    pub fn new() -> Self {
        Self { bones: Vec::new() }
    }

    /// Return the number of bones in this skeleton.
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }
}

impl Default for Skeleton3D {
    fn default() -> Self {
        Self::new()
    }
}

/// A mesh with per-vertex bone weights for GPU skinning.
#[derive(Debug)]
#[allow(dead_code)]
pub struct SkinnedMesh3D {
    /// Vertex data (position + normal + texcoord + bone indices + bone weights).
    pub vertices: Vec<f32>,
    /// The skeleton driving this mesh.
    pub skeleton: Skeleton3D,
    /// Current bone matrices (column-major, one per bone).
    pub bone_matrices: Vec<[f32; 16]>,
    /// Associated GPU buffer handle.
    pub(super) buffer: crate::libs::graphics::backend::BufferHandle,
    /// Number of vertices.
    pub(super) vertex_count: i32,
    /// Position in world space.
    pub(super) position: Vector3<f32>,
    /// Rotation (pitch, yaw, roll) in degrees.
    pub(super) rotation: Vector3<f32>,
    /// Scale.
    pub(super) scale: Vector3<f32>,
}

// ============================================================================
// Post-Processing Pipeline
// ============================================================================

/// A single render pass in the post-processing pipeline.
pub trait RenderPass: std::fmt::Debug + Send {
    /// The display name of this pass.
    fn name(&self) -> &str;

    /// Whether the pass is currently active.
    fn enabled(&self) -> bool;

    /// Process an RGBA8 image buffer in-place.
    fn process(&self, width: u32, height: u32, data: &mut [u8]);
}

/// A pipeline of chained post-processing passes.
#[derive(Debug)]
pub struct PostProcessPipeline {
    passes: Vec<Box<dyn RenderPass>>,
}

impl Default for PostProcessPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl PostProcessPipeline {
    /// Create a new empty post-processing pipeline.
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Add a render pass to the end of the pipeline.
    pub fn add_pass(&mut self, pass: Box<dyn RenderPass>) {
        self.passes.push(pass);
    }

    /// Remove a pass by index. Returns `true` if the index was valid.
    pub fn remove_pass(&mut self, index: usize) -> bool {
        if index < self.passes.len() {
            self.passes.remove(index);
            true
        } else {
            false
        }
    }

    /// Process an image through all enabled passes in order.
    pub fn run(&self, width: u32, height: u32, data: &mut [u8]) {
        for pass in &self.passes {
            if pass.enabled() {
                pass.process(width, height, data);
            }
        }
    }

    /// Return the number of passes in the pipeline.
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }
}

/// Bloom post-processing pass.
#[derive(Debug, Clone)]
pub struct BloomPass {
    /// Brightness threshold for bloom extraction.
    pub threshold: f32,
    /// Bloom intensity multiplier.
    pub intensity: f32,
    /// Whether this pass is enabled.
    pub enabled: bool,
}

impl Default for BloomPass {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            intensity: 1.0,
            enabled: true,
        }
    }
}

impl RenderPass for BloomPass {
    fn name(&self) -> &str {
        "Bloom"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn process(&self, width: u32, height: u32, data: &mut [u8]) {
        let pixel_count = (width * height) as usize;
        if data.len() < pixel_count * 4 {
            return;
        }
        // Extract bright pixels and additively blend them back.
        let mut bright = vec![0u8; data.len()];
        for i in 0..pixel_count {
            let idx = i * 4;
            let luminance = 0.2126 * (data[idx] as f32)
                + 0.7152 * (data[idx + 1] as f32)
                + 0.0722 * (data[idx + 2] as f32);
            if luminance / 255.0 > self.threshold {
                bright[idx] = data[idx];
                bright[idx + 1] = data[idx + 1];
                bright[idx + 2] = data[idx + 2];
            }
        }
        // Simple box blur on the bright pixels (3x3).
        let blurred = box_blur_rgba(&bright, width, height);
        for i in 0..pixel_count {
            let idx = i * 4;
            for c in 0..3 {
                let combined = data[idx + c] as f32 + blurred[idx + c] as f32 * self.intensity;
                data[idx + c] = (combined.min(255.0)) as u8;
            }
        }
    }
}

/// Gaussian blur post-processing pass.
#[derive(Debug, Clone)]
pub struct GaussianBlurPass {
    /// Blur radius in pixels.
    pub radius: u32,
    /// Whether this pass is enabled.
    pub enabled: bool,
}

impl Default for GaussianBlurPass {
    fn default() -> Self {
        Self {
            radius: 2,
            enabled: true,
        }
    }
}

impl RenderPass for GaussianBlurPass {
    fn name(&self) -> &str {
        "GaussianBlur"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn process(&self, width: u32, height: u32, data: &mut [u8]) {
        let result = box_blur_rgba(data, width, height);
        let len = (width * height * 4) as usize;
        if result.len() >= len && data.len() >= len {
            data[..len].copy_from_slice(&result[..len]);
        }
    }
}

/// Color grading / tone mapping post-processing pass.
#[derive(Debug, Clone)]
pub struct ColorGradePass {
    /// Exposure adjustment.
    pub exposure: f32,
    /// Contrast adjustment (1.0 = no change).
    pub contrast: f32,
    /// Saturation adjustment (1.0 = no change).
    pub saturation: f32,
    /// Whether this pass is enabled.
    pub enabled: bool,
}

impl Default for ColorGradePass {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            enabled: true,
        }
    }
}

impl RenderPass for ColorGradePass {
    fn name(&self) -> &str {
        "ColorGrade"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn process(&self, width: u32, height: u32, data: &mut [u8]) {
        let pixel_count = (width * height) as usize;
        if data.len() < pixel_count * 4 {
            return;
        }
        for i in 0..pixel_count {
            let idx = i * 4;
            for c in 0..3 {
                let mut v = data[idx + c] as f32 / 255.0;
                // Exposure
                v *= self.exposure;
                // Contrast
                v = ((v - 0.5) * self.contrast) + 0.5;
                // Saturation (simple luminance-based)
                let lum = 0.2126 * (data[idx] as f32 / 255.0)
                    + 0.7152 * (data[idx + 1] as f32 / 255.0)
                    + 0.0722 * (data[idx + 2] as f32 / 255.0);
                v = lum + (v - lum) * self.saturation;
                data[idx + c] = (v.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
    }
}

/// Simple 3x3 box blur for RGBA8 image data.
fn box_blur_rgba(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let mut output = data.to_vec();
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            for c in 0..3 {
                let mut sum: u32 = 0;
                for dy in 0..3u32 {
                    for dx in 0..3u32 {
                        let sx = x - 1 + dx as usize;
                        let sy = y - 1 + dy as usize;
                        sum += data[(sy * w + sx) * 4 + c] as u32;
                    }
                }
                output[(y * w + x) * 4 + c] = (sum / 9) as u8;
            }
        }
    }
    output
}
