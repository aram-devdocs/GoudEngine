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

mod animation;
#[allow(dead_code)]
mod animation_math;
mod animation_sampling;
#[cfg(test)]
mod animation_tests;
pub mod config;
mod core;
mod core_config;
mod core_materials;
mod core_model_animation;
mod core_model_instances;
mod core_models;
mod core_particles;
mod core_primitives;
mod core_scenes;
mod core_skinned;
mod core_static_batch;
mod debug_draw;
mod frustum;
mod material;
mod mesh;
mod model;
mod postprocess;
mod render;
mod render_helpers;
mod render_instanced;
mod render_instanced_skinned;
mod render_pass;
pub mod scene;
mod shaders;
mod shadow;
mod skinned_mesh;
mod texture;
mod types;

#[cfg(test)]
mod tests;

// Public API re-exports — the external interface is unchanged.
pub use animation::{AnimationPlayer, AnimationState, AnimationTransition};
pub use config::Render3DConfig;
pub use core::Renderer3D;
pub use scene::Scene3D;
pub use texture::TextureManagerTrait;
pub use types::{
    AntiAliasingMode, BloomPass, Bone3D, Camera3D, ColorGradePass, FogConfig, FogMode,
    GaussianBlurPass, GridConfig, GridRenderMode, InstanceTransform, Light, LightType, Material3D,
    MaterialType, ParticleEmitterConfig, PbrProperties, PostProcessPipeline, PrimitiveCreateInfo,
    PrimitiveType, RenderPass, Renderer3DStats, Skeleton3D, SkinnedMesh3D, SkyboxConfig, MAX_BONES,
    MAX_BONE_INFLUENCES, MAX_LIGHTS,
};
