//! Runtime texture atlas packing.
//!
//! Packs multiple textures into a single GPU texture at load time to
//! minimize draw calls when used with the sprite batch renderer.

mod atlas;
mod packer;
mod stats;

pub use atlas::{AtlasUvRect, PackedTextureInfo, TextureAtlas, DEFAULT_MAX_ATLAS_SIZE};
pub use packer::{PackedRect, ShelfPacker};
pub use stats::AtlasStats;
