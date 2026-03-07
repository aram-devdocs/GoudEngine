//! Font asset loading for TTF and OTF files.
//!
//! This module provides types for loading and managing font assets using
//! the `fontdue` crate for parsing and rasterization.

pub mod asset;
pub mod format;
pub mod loader;
pub mod settings;

#[cfg(test)]
mod tests;

pub use asset::{FontAsset, FontStyle};
pub use format::FontFormat;
pub use loader::FontLoader;
pub use settings::FontSettings;
