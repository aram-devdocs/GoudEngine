//! Tiled map asset loader.
//!
//! This module provides asset types and a loader for Tiled map files
//! (.tmx and .tmj). The loader uses the `tiled` crate to parse map
//! data and produces a [`TiledMapAsset`] containing tile layers,
//! object layers, and tileset references.
//!
//! # Example
//!
//! ```no_run
//! use goud_engine::assets::{AssetServer, loaders::tiled_map::{TiledMapLoader, TiledMapAsset}};
//!
//! let mut server = AssetServer::new();
//! server.register_loader(TiledMapLoader::default());
//!
//! let handle = server.load::<TiledMapAsset>("maps/level1.tmx");
//! ```

pub mod asset;
pub mod layer;
pub mod loader;

pub use asset::{visible_tile_range, TiledMapAsset};
pub use layer::{MapObject, ObjectLayer, TileLayer};
pub use loader::TiledMapLoader;
