//! Material asset loader.
//!
//! This module provides asset types and loaders for material descriptor
//! files in JSON format (`.mat.json`). Materials reference a shader and
//! define uniform values and texture slot bindings.
//!
//! # Example
//!
//! ```no_run
//! use goud_engine::assets::{AssetServer, loaders::material::{MaterialLoader, MaterialAsset}};
//!
//! let mut server = AssetServer::new();
//! server.register_loader(MaterialLoader::default());
//!
//! let handle = server.load::<MaterialAsset>("materials/brick.mat.json");
//! ```
mod asset;
mod loader;
mod uniform;

#[cfg(test)]
mod tests;

pub use asset::MaterialAsset;
pub use loader::MaterialLoader;
pub use uniform::UniformValue;
