//! # Context Registry
//!
//! This module implements the core context type and registry. A context
//! represents a single engine instance with its own World, asset storage,
//! and error state.
//!
//! ## Context Lifecycle
//!
//! 1. **Creation**: `get_context_registry().lock().create()` allocates a new context
//! 2. **Operations**: All operations accept `GoudContextId` as first parameter
//! 3. **Destruction**: `registry.destroy(id)` releases all resources
//!
//! ## Thread Safety
//!
//! - Contexts are stored in a global registry protected by `Mutex`
//! - Each context owns its World (not Send+Sync)
//! - Context operations must be called from the thread that created the context

pub mod context;
pub mod context_id;
pub mod registry;
pub mod scene;

#[cfg(test)]
mod tests;

// Re-export all public items so existing `use crate::context_registry::*` works.
pub use context::GoudContext;
pub use context_id::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
pub use registry::{get_context_registry, GoudContextHandle, GoudContextRegistry};
pub use scene::{
    EntityData, EntityRemap, SceneData, SceneId, SceneManager, SerializedEntity,
    DEFAULT_SCENE_NAME,
};
