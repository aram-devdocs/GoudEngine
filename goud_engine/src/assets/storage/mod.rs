//! Asset storage system for caching and managing loaded assets.
//!
//! This module provides the infrastructure for storing loaded assets:
//!
//! - [`TypedAssetStorage<A>`]: Storage for a single asset type
//! - [`AssetStorage`]: Type-erased container for all asset types
//! - [`AssetEntry<A>`]: Individual asset entry with metadata
//!
//! # Design Philosophy
//!
//! The asset storage system is designed with these goals:
//!
//! 1. **Type Safety**: Typed access through handles prevents misuse
//! 2. **Efficient Lookup**: O(1) access by handle, O(1) amortized by path
//! 3. **Memory Efficient**: Slot reuse through generational indices
//! 4. **Thread Safe**: Storage is `Send + Sync` for parallel access
//!
//! # Storage Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        AssetStorage                              │
//! │  ┌───────────────────────────────────────────────────────────┐  │
//! │  │  HashMap<AssetId, Box<dyn AnyAssetStorage>>               │  │
//! │  │  ┌─────────────────────┐  ┌─────────────────────┐        │  │
//! │  │  │TypedAssetStorage<T1>│  │TypedAssetStorage<T2>│  ...   │  │
//! │  │  │  - allocator        │  │  - allocator        │        │  │
//! │  │  │  - entries[]        │  │  - entries[]        │        │  │
//! │  │  │  - path_index       │  │  - path_index       │        │  │
//! │  │  └─────────────────────┘  └─────────────────────┘        │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```
//! use goud_engine::assets::{Asset, AssetStorage, AssetState, AssetPath};
//!
//! // Define asset types
//! struct Texture { width: u32, height: u32 }
//! impl Asset for Texture {}
//!
//! struct Audio { duration: f32 }
//! impl Asset for Audio {}
//!
//! // Create storage
//! let mut storage = AssetStorage::new();
//!
//! // Insert assets
//! let tex_handle = storage.insert(Texture { width: 256, height: 256 });
//! let audio_handle = storage.insert(Audio { duration: 2.5 });
//!
//! // Access assets
//! let tex = storage.get::<Texture>(&tex_handle);
//! assert!(tex.is_some());
//!
//! // Path-based lookup (after associating path)
//! storage.set_path(&tex_handle, AssetPath::new("textures/player.png"));
//! let found = storage.get_handle_by_path::<Texture>("textures/player.png");
//! assert_eq!(found, Some(tex_handle));
//! ```

mod any_storage;
mod container;
mod entry;
mod typed;

#[cfg(test)]
mod tests;

pub use any_storage::AnyAssetStorage;
pub use container::AssetStorage;
pub use entry::AssetEntry;
pub use typed::TypedAssetStorage;
