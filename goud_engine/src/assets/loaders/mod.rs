//! Built-in asset loaders.
//!
//! This module contains loaders for common asset types like textures,
//! shaders, and audio files.

use crate::assets::AssetServer;

pub mod animation;
pub mod audio;
pub mod bitmap_font;
pub mod config;
pub mod font;
pub(crate) mod gltf_utils;
pub mod material;
pub mod mesh;
#[cfg(feature = "desktop-native")]
mod rodio_integration;
#[cfg(feature = "lua")]
pub mod script;
pub mod shader;
pub mod sprite_sheet;
pub mod texture;
pub mod tiled_map;

#[cfg(feature = "lua")]
pub use script::{ScriptAsset, ScriptLoader};

pub use texture::{
    TextureAsset, TextureColorSpace, TextureFormat, TextureLoader, TextureSettings, TextureWrapMode,
};

pub use shader::{
    ShaderAsset, ShaderFormat, ShaderLoader, ShaderSettings, ShaderSource, ShaderStage,
};
pub use sprite_sheet::{SpriteRegion, SpriteSheetAsset, SpriteSheetLoader};

pub use audio::{AudioAsset, AudioFormat, AudioLoader, AudioSettings};

pub use config::{ConfigAsset, ConfigFormat, ConfigLoader};

pub use font::{FontAsset, FontFormat, FontLoader, FontSettings, FontStyle};

pub use bitmap_font::{BitmapCharInfo, BitmapFontAsset, BitmapFontLoader};

pub use material::{MaterialAsset, MaterialLoader, UniformValue};

pub use animation::{AnimationLoader, KeyframeAnimation};

#[cfg(feature = "native")]
pub use mesh::{default_registry, FbxProvider, GltfProvider, ObjProvider};
pub use mesh::{
    BoneData, MeshAsset, MeshLoader, MeshVertex, ModelData, ModelProvider, ModelProviderRegistry,
    SkeletonData, SubMesh,
};

pub use tiled_map::{
    visible_tile_range, MapObject, ObjectLayer, TileLayer, TiledMapAsset, TiledMapLoader,
};

pub(crate) fn ensure_3d_asset_loaders(asset_server: &mut AssetServer) {
    if !asset_server.has_loader_for_type::<TextureAsset>() {
        asset_server.register_loader(TextureLoader);
    }
    if !asset_server.has_loader_for_type::<ShaderAsset>() {
        asset_server.register_loader(ShaderLoader::default());
    }
    if !asset_server.has_loader_for_type::<MaterialAsset>() {
        asset_server.register_loader(MaterialLoader);
    }
    if !asset_server.has_loader_for_type::<MeshAsset>() {
        asset_server.register_loader(MeshLoader);
    }
}
