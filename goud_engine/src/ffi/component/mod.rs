//! # FFI Component Operations
//!
//! This module provides C-compatible functions for adding, removing, and querying
//! components on entities. Since components are generic types in Rust, the FFI
//! layer uses raw byte pointers and type IDs for component data.
//!
//! ## Design
//!
//! Component operations in FFI require:
//! - **Type Registration**: Components must be registered with the engine
//! - **Raw Pointers**: Component data passed as `*const u8` / `*mut u8`
//! - **Size/Alignment**: Caller must provide correct size and alignment
//! - **Type IDs**: Components identified by 64-bit type hash
//!
//! ## Safety
//!
//! The FFI layer performs extensive validation:
//! - Context ID validation
//! - Entity liveness checks
//! - Pointer null checks
//! - Size/alignment validation
//! - Type ID verification
//!
//! However, the caller MUST ensure:
//! - Pointers point to valid component data
//! - Size/alignment match the registered type
//! - Memory remains valid for the duration of the call
//!
//! ## Thread Safety
//!
//! Component operations must be called from the thread that owns the context.
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // Register component type (once at startup)
//! var positionTypeId = goud_component_register_type(
//!     contextId,
//!     "Position",
//!     sizeof(Position),
//!     alignof(Position)
//! );
//!
//! // Create entity
//! var entity = goud_entity_spawn_empty(contextId);
//!
//! // Add component
//! var position = new Position { x = 10.0f, y = 20.0f };
//! fixed (Position* ptr = &position) {
//!     var result = goud_component_add(
//!         contextId,
//!         entity,
//!         positionTypeId,
//!         ptr,
//!         sizeof(Position)
//!     );
//!     if (!result.success) {
//!         // Handle error...
//!     }
//! }
//!
//! // Check if entity has component
//! if (goud_component_has(contextId, entity, positionTypeId)) {
//!     // Entity has Position component
//! }
//!
//! // Get component (read-only)
//! var posPtr = goud_component_get(contextId, entity, positionTypeId);
//! if (posPtr != null) {
//!     var pos = Marshal.PtrToStructure<Position>(posPtr);
//!     Console.WriteLine($"Position: ({pos.x}, {pos.y})");
//! }
//!
//! // Get component (mutable)
//! var posMutPtr = goud_component_get_mut(contextId, entity, positionTypeId);
//! if (posMutPtr != null) {
//!     var pos = Marshal.PtrToStructure<Position>(posMutPtr);
//!     pos.x += 1.0f;
//!     Marshal.StructureToPtr(pos, posMutPtr, false);
//! }
//!
//! // Remove component
//! var removeResult = goud_component_remove(contextId, entity, positionTypeId);
//! if (removeResult.success) {
//!     // Component removed successfully
//! }
//! ```

mod access;
mod batch;
mod ops;
mod registry;
mod storage;

// Re-export all public FFI functions so callers using `crate::ffi::component::*`
// or the parent `pub mod component` declaration continue to work unchanged.
pub use access::{goud_component_get, goud_component_get_mut, goud_component_has};
pub use batch::{goud_component_add_batch, goud_component_has_batch, goud_component_remove_batch};
pub use ops::{goud_component_add, goud_component_register_type, goud_component_remove};

#[cfg(test)]
mod ffi_tests;
