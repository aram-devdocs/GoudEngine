//! Asset handles for type-safe, reference-counted asset access.
//!
//! This module provides specialized handle types for the asset system:
//!
//! - [`AssetHandle<A>`]: A typed handle to a specific asset type
//! - [`UntypedAssetHandle`]: A type-erased handle for dynamic asset access
//! - [`AssetPath`]: A path-based identifier for assets
//!
//! # Design Philosophy
//!
//! Asset handles differ from regular [`Handle<T>`](crate::core::handle::Handle) in several ways:
//!
//! 1. **Load State Tracking**: Asset handles know if their asset is loading, loaded, or failed
//! 2. **Path Association**: Assets can be loaded by path, and handles preserve this association
//! 3. **Type Erasure**: Untyped handles allow heterogeneous asset collections
//! 4. **Reference Semantics**: Multiple handles can reference the same asset
//!
//! # FFI Safety
//!
//! Both `AssetHandle<A>` and `UntypedAssetHandle` are FFI-compatible:
//! - Fixed size (16 bytes for typed, 24 bytes for untyped)
//! - `#[repr(C)]` layout
//! - Can be converted to/from integer representations
//!
//! # Example
//!
//! ```
//! use goud_engine::assets::{Asset, AssetHandle, AssetPath, HandleLoadState};
//!
//! // Define a texture asset type
//! struct Texture { width: u32, height: u32 }
//! impl Asset for Texture {}
//!
//! // Create a handle (normally done by AssetServer)
//! let handle: AssetHandle<Texture> = AssetHandle::new(0, 1);
//!
//! // Check handle state
//! assert!(handle.is_valid());
//!
//! // Asset handles can be associated with paths
//! let path = AssetPath::new("textures/player.png");
//! assert_eq!(path.extension(), Some("png"));
//! ```

pub mod allocator;
pub mod load_state;
pub mod path;
pub mod typed;
pub mod untyped;

#[cfg(test)]
mod tests;

// Re-export all public types so the parent module sees an unchanged API.
pub use allocator::AssetHandleAllocator;
pub use load_state::HandleLoadState;
pub use path::AssetPath;
pub use typed::{AssetHandle, WeakAssetHandle};
pub use untyped::UntypedAssetHandle;
