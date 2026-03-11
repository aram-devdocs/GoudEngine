//! Texture asset loader.
//!
//! This module provides asset types and loaders for image-based textures.
//! Supports common image formats like PNG, JPG, BMP, TGA, and more.
//!
//! # Example
//!
//! ```no_run
//! use goud_engine::assets::{
//!     AssetServer,
//!     loaders::texture::{TextureAsset, TextureLoader},
//! };
//!
//! let mut server = AssetServer::new();
//! server.register_loader(TextureLoader::default());
//!
//! let handle = server.load::<TextureAsset>("textures/player.png");
//! ```
mod asset;
pub mod compressed;
pub mod dds;
mod format;
mod loader;
mod settings;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_integration;

pub use asset::TextureAsset;
pub use compressed::{CompressedTextureAsset, CompressedTextureLoader};
pub use dds::CompressedFormat;
pub use format::TextureFormat;
pub use loader::TextureLoader;
pub use settings::{TextureColorSpace, TextureSettings, TextureWrapMode};
