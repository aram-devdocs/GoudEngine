//! Type-safe, generation-counted handles for engine objects.
//!
//! Handles are the primary mechanism for referencing engine objects across the FFI
//! boundary. They provide:
//!
//! - **Type safety**: Handles are generic over the resource type, preventing
//!   accidental use of a texture handle where a shader handle is expected.
//! - **Generation counting**: Each handle includes a generation counter that
//!   increments when a slot is reused, preventing use-after-free bugs.
//! - **FFI compatibility**: The `#[repr(C)]` layout ensures consistent memory
//!   representation across language boundaries.
//!
//! # Design Pattern: Generational Indices
//!
//! Generational indices solve the ABA problem in resource management:
//!
//! 1. Allocate slot 5, generation 1 -> Handle(5, 1)
//! 2. Deallocate slot 5, generation becomes 2
//! 3. Allocate slot 5 again, generation 2 -> Handle(5, 2)
//! 4. Old Handle(5, 1) is now invalid (generation mismatch)
//!
//! # Example
//!
//! ```
//! use goud_engine::core::handle::Handle;
//!
//! // Marker type for textures
//! struct Texture;
//!
//! // Create a handle (normally done by HandleAllocator)
//! let handle: Handle<Texture> = Handle::new(0, 1);
//!
//! assert_eq!(handle.index(), 0);
//! assert_eq!(handle.generation(), 1);
//! assert!(handle.is_valid());
//!
//! // Invalid handle for representing "no resource"
//! let invalid: Handle<Texture> = Handle::INVALID;
//! assert!(!invalid.is_valid());
//! ```

mod allocator;
mod handle_type;
pub(crate) mod iterators;
mod map;

pub use allocator::HandleAllocator;
pub use handle_type::Handle;
pub use iterators::{
    HandleMapHandles, HandleMapIter, HandleMapIterMut, HandleMapValues, HandleMapValuesMut,
};
pub use map::HandleMap;

#[cfg(test)]
mod tests;
