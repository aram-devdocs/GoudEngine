//! # SDK Transform2D Component Operations
//!
//! Pure transform operations exposed via `#[goud_api]` proc-macro.
//! Factory functions create transforms by value; mutation functions
//! operate on `*mut FfiTransform2D` pointers with null-safety.

// The proc-macro wraps these in `unsafe extern "C"` FFI wrappers.
// The inner methods do their own null checks before any dereference.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod builder;
pub mod factory;
pub mod ptr_ops;
