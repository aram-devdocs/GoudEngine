//! Asset management system for the engine.
//!
//! This module provides the infrastructure for loading, caching, and managing
//! game assets such as textures, audio, meshes, and shaders. The asset system
//! is designed with these goals:
//!
//! - **Type Safety**: Assets are accessed through typed handles, preventing
//!   accidental misuse of asset references.
//! - **Async Loading**: Assets can be loaded asynchronously without blocking
//!   the game loop.
//! - **Reference Counting**: Automatic cleanup when assets are no longer referenced.
//! - **Hot Reloading**: Support for reloading assets during development.
//! - **FFI Compatibility**: All types are designed for cross-language use.
//!
//! # Core Concepts
//!
//! ## Assets
//!
//! An asset is any piece of content loaded from external sources (files, network, etc.)
//! that the game uses at runtime. The [`Asset`] trait marks types that can be managed
//! by the asset system.
//!
//! ## Asset Handles
//!
//! Assets are accessed through handles, never raw references. Handles provide:
//! - Generation counting to detect stale references
//! - Type safety preventing handle misuse
//! - FFI-safe representation
//!
//! The primary handle types are:
//! - [`AssetHandle<A>`]: Typed handle for a specific asset type
//! - [`UntypedAssetHandle`]: Type-erased handle for dynamic collections
//! - [`WeakAssetHandle<A>`]: Non-owning reference to an asset
//!
//! ## Asset Pipeline
//!
//! ```text
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//! │  Raw Asset  │───▶│   Loader    │───▶│   Cache     │───▶│   Handle    │
//! │   (File)    │    │  (Async)    │    │  (Memory)   │    │  (Returned) │
//! └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
//! ```
//!
//! # Example
//!
//! ```
//! use goud_engine::assets::{Asset, AssetHandle, AssetPath};
//!
//! // Define a custom asset type
//! struct MyTexture {
//!     width: u32,
//!     height: u32,
//!     data: Vec<u8>,
//! }
//!
//! // Implement the Asset trait
//! impl Asset for MyTexture {
//!     fn asset_type_name() -> &'static str {
//!         "MyTexture"
//!     }
//! }
//!
//! // Create a handle (normally done by AssetServer)
//! let handle: AssetHandle<MyTexture> = AssetHandle::new(0, 1);
//! assert!(handle.is_valid());
//!
//! // Work with asset paths
//! let path = AssetPath::new("textures/player.png");
//! assert_eq!(path.extension(), Some("png"));
//! ```

mod asset;
mod audio_manager;
mod handle;
mod hot_reload;
mod loader;
mod server;
mod storage;

// Built-in asset loaders
pub mod loaders;

// Re-export core asset traits and types
pub use asset::{Asset, AssetId, AssetInfo, AssetState, AssetType};

// Re-export asset handle types
pub use handle::{
    AssetHandle, AssetHandleAllocator, AssetPath, HandleLoadState, UntypedAssetHandle,
    WeakAssetHandle,
};

// Re-export asset loader types
pub use loader::{
    AssetLoadError, AssetLoader, ErasedAssetLoader, LoadContext, TypedAssetLoader,
};

// Re-export asset server
pub use server::AssetServer;

// Re-export asset storage types
pub use storage::{AnyAssetStorage, AssetEntry, AssetStorage, TypedAssetStorage};

// Re-export hot reload types
pub use hot_reload::{AssetChangeEvent, HotReloadConfig, HotReloadWatcher};

// Re-export audio manager
pub use audio_manager::AudioManager;
