//! Built-in asset loaders.
//!
//! This module contains loaders for common asset types like textures,
//! shaders, and audio files.

pub mod animation;
pub mod audio;
pub mod bitmap_font;
pub mod config;
pub mod font;
pub(crate) mod gltf_utils;
pub mod material;
pub mod mesh;
#[cfg(feature = "native")]
mod rodio_integration;
pub mod shader;
pub mod texture;
pub mod tiled_map;

pub use texture::{
    TextureAsset, TextureColorSpace, TextureFormat, TextureLoader, TextureSettings, TextureWrapMode,
};

pub use shader::{
    ShaderAsset, ShaderFormat, ShaderLoader, ShaderSettings, ShaderSource, ShaderStage,
};

pub use audio::{AudioAsset, AudioFormat, AudioLoader, AudioSettings};

pub use config::{ConfigAsset, ConfigFormat, ConfigLoader};

pub use font::{FontAsset, FontFormat, FontLoader, FontSettings, FontStyle};

pub use bitmap_font::{BitmapCharInfo, BitmapFontAsset, BitmapFontLoader};

pub use material::{MaterialAsset, MaterialLoader, UniformValue};

pub use animation::{AnimationLoader, KeyframeAnimation};

pub use mesh::{MeshAsset, MeshLoader, MeshVertex, SubMesh};

pub use tiled_map::{
    visible_tile_range, MapObject, ObjectLayer, TileLayer, TiledMapAsset, TiledMapLoader,
};
