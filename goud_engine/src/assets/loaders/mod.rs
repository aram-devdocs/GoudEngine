//! Built-in asset loaders.
//!
//! This module contains loaders for common asset types like textures,
//! shaders, and audio files.

pub mod audio;
mod rodio_integration;
pub mod shader;
pub mod texture;

pub use texture::{
    TextureAsset, TextureColorSpace, TextureFormat, TextureLoader, TextureSettings, TextureWrapMode,
};

pub use shader::{
    ShaderAsset, ShaderFormat, ShaderLoader, ShaderSettings, ShaderSource, ShaderStage,
};

pub use audio::{AudioAsset, AudioFormat, AudioLoader, AudioSettings};
