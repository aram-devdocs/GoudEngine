//! Shared helpers for FFI code generation.
//!
//! This module provides utilities for parameter extraction, receiver detection,
//! and return handling used by the `ffi_gen` module.

mod params;
mod receiver;
mod return_handling;

pub use params::{extract_param_name, generate_param_conversion, generate_vec2_reconstruction};
pub use receiver::{determine_receiver, ReceiverKind};
pub use return_handling::{generate_return_handling, manifest_return_info};
