//! # FFI (Foreign Function Interface) Layer
//!
//! This module provides a professional, type-safe FFI layer for cross-language
//! interoperability with the GoudEngine. It is designed to support multiple
//! language bindings (C#, Python, TypeScript, Lua, Go, Rust native) through a
//! consistent context-based API.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  Language Bindings Layer                     │
//! │  (C#, Python, TypeScript, Lua, Go, Rust native)             │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    FFI Context Layer                         │
//! │  - Context Registry (thread-safe handle storage)            │
//! │  - Error Handling (thread-local error codes/messages)       │
//! │  - Type-safe operations (entity, component, resource)       │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  Rust Engine Core (World)                   │
//! │  - ECS (Entity, Component, System)                          │
//! │  - Assets (Loader, Storage, Hot Reload)                     │
//! │  - Graphics (Renderer, Texture, Shader)                     │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Design Principles
//!
//! 1. **Context-Based**: All FFI operations go through opaque context handles,
//!    preventing direct pointer manipulation and enabling proper lifecycle management.
//!
//! 2. **Thread-Safe**: Context registry uses interior mutability with Mutex for
//!    concurrent access. Error storage is thread-local to avoid races.
//!
//! 3. **Error Handling**: Comprehensive error codes with FFI-safe integer ranges,
//!    thread-local error storage, and human-readable error messages.
//!
//! 4. **Type Safety**: Strong typing with generational handles prevents use-after-free
//!    and type confusion across FFI boundary.
//!
//! 5. **ABI Stability**: All FFI types use `#[repr(C)]` and primitive types for
//!    stable binary interface across compiler versions.
//!
//! 6. **Non-Nullable**: FFI functions never return null pointers; they use error
//!    codes and sentinel values for failure cases.
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // Create context
//! var contextId = goud_context_create();
//! if (contextId == GOUD_INVALID_CONTEXT_ID) {
//!     var error = goud_get_last_error_message();
//!     throw new Exception(error);
//! }
//!
//! // Initialize world
//! var result = goud_world_initialize(contextId);
//! if (result != GOUD_SUCCESS) {
//!     // Handle error...
//! }
//!
//! // Spawn entity
//! var entity = goud_entity_spawn(contextId);
//!
//! // Destroy context
//! goud_context_destroy(contextId);
//! ```
//!
//! ## Modules
//!
//! - `context` - Context type and registry for managing engine instances
//! - `error` - Error handling types and thread-local error storage
//! - `entity` - Entity spawn/despawn operations
//! - `component` - Component add/remove/query operations
//! - `resource` - Resource insert/remove/access operations
//! - `asset` - Asset loading and management operations
//!
//! ## Safety
//!
//! All FFI functions are `unsafe` from Rust's perspective because they:
//! - Accept raw pointers from foreign code
//! - Cannot verify lifetime or ownership of data
//! - Must validate all inputs for safety
//!
//! However, the API is designed to be safe from the caller's perspective:
//! - No null pointer dereferencing (validated immediately)
//! - No use-after-free (generational handles detect stale references)
//! - No type confusion (handles are strongly typed)
//! - No data races (contexts are Send+Sync, errors are thread-local)

pub mod collision;
pub mod component;
pub mod component_sprite;
pub mod component_transform2d;
pub mod context;
pub mod entity;
pub mod input;
pub mod renderer;
pub mod renderer3d;
pub mod types;
pub mod window;

// Re-export core types for convenience
pub use context::{
    GoudContext, GoudContextHandle, GoudContextId, GoudContextRegistry, GOUD_INVALID_CONTEXT_ID,
};
pub use entity::GOUD_INVALID_ENTITY_ID;
pub use types::{FfiVec2, GoudEntityId, GoudResult};

// Re-export error types from core module
pub use crate::core::error::{GoudError, GoudErrorCode, GoudResult as CoreResult};

// Re-export component FFI types
pub use component_sprite::{FfiColor, FfiRect, FfiSprite, FfiSpriteBuilder};
pub use component_transform2d::{FfiMat3x3, FfiTransform2D, FfiTransform2DBuilder};
