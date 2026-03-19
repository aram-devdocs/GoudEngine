//! 3D Renderer Module
//!
//! Provides a complete 3D rendering system with:
//! - Primitive creation (cubes, spheres, planes, cylinders)
//! - Multiple light types (point, directional, spot)
//! - Camera control with position and rotation
//! - Grid rendering
//! - Skybox support
//!
//! All GPU operations go through the [`crate::libs::graphics::backend::RenderBackend`] trait,
//! enabling backend-agnostic rendering (OpenGL, wgpu, etc.).

mod core;
mod core_particles;
mod core_primitives;
mod debug_draw;
mod material;
mod mesh;
mod postprocess;
mod render;
mod render_pass;
mod shaders;
mod shadow;
mod skinned_mesh;
mod texture;
mod types;

#[cfg(test)]
mod tests;

// Public API re-exports — the external interface is unchanged.
pub use core::Renderer3D;
pub use texture::TextureManagerTrait;
pub use types::{
    AntiAliasingMode, BloomPass, Bone3D, Camera3D, ColorGradePass, FogConfig, GaussianBlurPass,
    GridConfig, GridRenderMode, InstanceTransform, Light, LightType, Material3D, MaterialType,
    ParticleEmitterConfig, PbrProperties, PostProcessPipeline, PrimitiveCreateInfo, PrimitiveType,
    RenderPass, Renderer3DStats, Skeleton3D, SkinnedMesh3D, SkyboxConfig, MAX_BONES,
    MAX_BONE_INFLUENCES, MAX_LIGHTS,
};
