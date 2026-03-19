//! Material system types for the 3D renderer.

use cgmath::Vector4;

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
    pub color: Vector4<f32>,
    /// Specular highlight intensity (Phong).
    pub shininess: f32,
    /// Optional PBR properties (only used when `material_type == Pbr`).
    pub pbr: PbrProperties,
}

impl Default for Material3D {
    fn default() -> Self {
        Self {
            material_type: MaterialType::Phong,
            color: Vector4::new(0.8, 0.8, 0.8, 1.0),
            shininess: 32.0,
            pbr: PbrProperties::default(),
        }
    }
}
