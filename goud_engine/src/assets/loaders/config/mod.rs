//! Configuration asset loader.
//!
//! This module provides asset types and loaders for configuration files
//! in JSON and TOML formats. All config data is normalized to
//! [`serde_json::Value`] for uniform access.
//!
//! # Example
//!
//! ```no_run
//! use goud_engine::assets::{AssetServer, loaders::config::{ConfigLoader, ConfigAsset}};
//!
//! let mut server = AssetServer::new();
//! server.register_loader(ConfigLoader::default());
//!
//! let handle = server.load::<ConfigAsset>("config/settings.json");
//! ```
mod asset;
mod format;
mod loader;

#[cfg(test)]
mod tests;

pub use asset::ConfigAsset;
pub use format::ConfigFormat;
pub use loader::ConfigLoader;
