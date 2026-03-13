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
mod debug_draw;
mod mesh;
mod render;
mod shaders;
mod texture;
mod types;

// Public API re-exports — the external interface is unchanged.
pub use core::Renderer3D;
pub use texture::TextureManagerTrait;
pub use types::{
    Camera3D, FogConfig, GridConfig, GridRenderMode, Light, LightType, PrimitiveCreateInfo,
    PrimitiveType, SkyboxConfig, MAX_LIGHTS,
};
