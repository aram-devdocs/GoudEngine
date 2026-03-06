//! Core asset trait and type identification.
//!
//! This module defines the [`Asset`] trait that all loadable assets must implement,
//! along with supporting types for asset identification and metadata.
//!
//! # Design Philosophy
//!
//! Assets in GoudEngine are designed to be:
//!
//! 1. **Type-Safe**: Each asset type has a unique [`AssetId`] for runtime type checking
//! 2. **Thread-Safe**: Assets must be `Send + Sync` for parallel loading
//! 3. **Self-Describing**: Assets can report their type name and metadata
//! 4. **FFI-Compatible**: All IDs and enums are designed for cross-language use
//!
//! # Example
//!
//! ```
//! use goud_engine::assets::{Asset, AssetId, AssetType};
//!
//! // Custom texture asset
//! struct Texture {
//!     width: u32,
//!     height: u32,
//!     gpu_id: u32,
//! }
//!
//! impl Asset for Texture {
//!     fn asset_type_name() -> &'static str {
//!         "Texture"
//!     }
//!
//!     fn asset_type() -> AssetType {
//!         AssetType::Texture
//!     }
//! }
//!
//! // Get runtime type info
//! let id = AssetId::of::<Texture>();
//! println!("Texture type ID: {:?}", id);
//! ```

mod asset_id;
mod asset_info;
mod asset_state;
mod asset_type;
mod trait_def;

pub use asset_id::AssetId;
pub use asset_info::AssetInfo;
pub use asset_state::AssetState;
pub use asset_type::AssetType;
pub use trait_def::Asset;

#[cfg(test)]
mod tests;
