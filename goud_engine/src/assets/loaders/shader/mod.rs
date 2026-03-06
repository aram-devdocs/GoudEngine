//! Shader asset loader.
//!
//! This module provides asset types and loaders for GLSL shaders.
//! Supports vertex, fragment, geometry, and compute shaders.
//!
//! # Example
//!
//! ```no_run
//! use goud_engine::assets::{AssetServer, loaders::shader::ShaderLoader, loaders::shader::ShaderAsset};
//!
//! let mut server = AssetServer::new();
//! server.register_loader(ShaderLoader::default());
//!
//! // Load a complete shader program
//! let handle = server.load::<ShaderAsset>("shaders/basic.shader");
//! ```

mod asset;
mod loader;
mod stage;

pub use asset::{ShaderAsset, ShaderFormat, ShaderSource};
pub use loader::{ShaderLoader, ShaderSettings};
pub use stage::ShaderStage;

#[cfg(test)]
mod tests;
