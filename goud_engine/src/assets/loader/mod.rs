//! Asset loading infrastructure.
//!
//! This module provides traits and types for implementing custom asset loaders.
//! Asset loaders are responsible for converting raw data (e.g., file bytes) into
//! typed asset instances.
//!
//! # Architecture
//!
//! The loading system is designed for asynchronous operation:
//!
//! ```text
//! ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
//! │  Raw Bytes   │────▶│    Loader    │────▶│    Asset     │
//! │  (from I/O)  │     │  (Parser)    │     │  (Typed)     │
//! └──────────────┘     └──────────────┘     └──────────────┘
//! ```
//!
//!
//! ```ignore
//! use goud_engine::assets::{Asset, AssetLoader, AssetLoadError, LoadContext};
//!
//! #[derive(Clone)]
//! struct MyAsset {
//!     data: String,
//! }
//!
//! impl Asset for MyAsset {}
//!
//! #[derive(Clone)]
//! struct MyAssetLoader;
//!
//! impl AssetLoader for MyAssetLoader {
//!     type Asset = MyAsset;
//!     type Settings = ();
//!
//!     fn extensions(&self) -> &[&str] {
//!         &["myasset"]
//!     }
//!
//!     fn load<'a>(
//!         &'a self,
//!         bytes: &'a [u8],
//!         _settings: &'a Self::Settings,
//!         _context: &'a mut LoadContext,
//!     ) -> Result<Self::Asset, AssetLoadError> {
//!         let data = String::from_utf8(bytes.to_vec())
//!             .map_err(|e| AssetLoadError::decode_failed(e.to_string()))?;
//!         Ok(MyAsset { data })
//!     }
//! }
//! ```
mod context;
mod error;
pub mod test_fixtures;
mod traits;

#[cfg(test)]
mod tests;

pub use context::LoadContext;
pub use error::AssetLoadError;
pub use traits::{AssetLoader, ErasedAssetLoader, TypedAssetLoader};
