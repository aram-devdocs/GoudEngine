//! # Core Component Operations
//!
//! This module provides the implementation logic for type-erased component
//! operations: register, add, remove, has, get, get_mut, and batch variants.
//!
//! These `_impl` functions contain the actual logic and are called by both
//! the FFI `#[no_mangle]` wrappers and the Rust SDK wrapper types.
//!
//! ## Design
//!
//! Component operations use raw byte pointers and type IDs because the FFI
//! layer does not know concrete Rust types at compile time. The raw component
//! storage uses a sparse set internally.
//!
//! ## Safety
//!
//! The caller MUST ensure:
//! - Pointers point to valid component data
//! - Size/alignment match the registered type
//! - Memory remains valid for the duration of the call

mod batch_ops;
mod helpers;
mod single_ops;
mod storage;

// Re-export all public _impl functions so external callers can use
// `crate::component_ops::function_name` unchanged.
pub use batch_ops::{
    component_add_batch_impl, component_has_batch_impl, component_remove_batch_impl,
};
pub use single_ops::{
    component_add_impl, component_get_impl, component_get_mut_impl, component_has_impl,
    component_register_type_impl, component_remove_impl,
};
