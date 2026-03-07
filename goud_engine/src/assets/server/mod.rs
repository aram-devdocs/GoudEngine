//! Asset server for managing asset loading and caching.
//!
//! The `AssetServer` is the central coordinator for all asset operations:
//! - Loading assets from disk (sync and async)
//! - Caching loaded assets
//! - Tracking asset loading states
//! - Managing asset loaders
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                      AssetServer                            │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
//! │  │   Loaders    │  │   Storage    │  │  IO Thread   │    │
//! │  │  Registry    │  │   (Cache)    │  │    Pool      │    │
//! │  └──────────────┘  └──────────────┘  └──────────────┘    │
//! └────────────────────────────────────────────────────────────┘
//!         │                    │                    │
//!         ▼                    ▼                    ▼
//!    Load Asset          Get Cached           Async Load
//! ```
//!
//! # Example
//!
//! ```
//! use goud_engine::assets::{Asset, AssetServer, AssetPath};
//!
//! struct Texture { width: u32, height: u32 }
//! impl Asset for Texture {}
//!
//! // Create asset server
//! let mut server = AssetServer::new();
//!
//! // Load an asset (returns handle immediately, loads in background)
//! let handle = server.load::<Texture>("textures/player.png");
//!
//! // Check if loaded
//! if server.is_loaded(&handle) {
//!     if let Some(texture) = server.get(&handle) {
//!         println!("Texture: {}x{}", texture.width, texture.height);
//!     }
//! }
//! ```

mod async_operations;
mod core;
mod loader_registry;
mod operations;
mod web_operations;

#[cfg(test)]
mod async_tests;
#[cfg(test)]
mod tests;

pub use core::AssetServer;
