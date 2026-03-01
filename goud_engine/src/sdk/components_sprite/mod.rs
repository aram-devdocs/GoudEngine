//! # SDK Sprite Component Operations
//!
//! Pure sprite operations exposed via `#[goud_api]` proc-macro.
//! Factory functions create sprites by value; mutation functions
//! operate on `*mut FfiSprite` pointers with null-safety.
//! Value-builder functions take and return `FfiSprite` by value.

// The proc-macro wraps these in `unsafe extern "C"` FFI wrappers.
// The inner methods do their own null checks before any dereference.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod builder;
pub mod factory;
pub mod ptr_ops;
