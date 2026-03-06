//! Types for the 3D renderer: enums, config structs, camera, and scene objects.

use crate::libs::graphics::backend::BufferHandle;
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

/// A 3D object in the scene
#[derive(Debug)]
pub(super) struct Object3D {
    pub(super) buffer: BufferHandle,
    pub(super) vertex_count: i32,
    pub(super) position: Vector3<f32>,
    pub(super) rotation: Vector3<f32>,
    pub(super) scale: Vector3<f32>,
    pub(super) texture_id: u32,
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
