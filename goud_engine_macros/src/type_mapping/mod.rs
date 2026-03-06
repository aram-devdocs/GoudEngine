//! Type mapping from SDK/Rust types to FFI-compatible types.
//!
//! This module defines how Rust types are converted to C-ABI-safe types
//! for generated FFI wrapper functions.

mod helpers;
mod mapping;
#[cfg(test)]
mod tests;
mod types;

pub use mapping::{map_return_type, map_type};
pub use types::{FfiParam, FfiReturn, FfiTypeInfo};
