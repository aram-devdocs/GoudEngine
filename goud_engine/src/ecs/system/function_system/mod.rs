//! Function system wrapper for converting functions into systems.
//!
//! This module provides the [`FunctionSystem`] type that wraps a Rust function
//! and its associated state to implement the [`System`] trait. This allows
//! ergonomic system definition using regular functions.
//!
//! # Architecture
//!
//! The function system architecture consists of:
//!
//! - [`FunctionSystem`]: Wraps a function and its cached parameter state
//! - [`SystemParamFunction`]: Trait for functions that can be systems
//! - [`IntoSystem`] implementations: Convert functions to boxed systems
//!
//! # Supported Function Signatures
//!
//! Functions can have 0 to 8 parameters, each implementing [`SystemParam`]:
//!
//! ```ignore
//! fn no_params() { }
//! fn one_param(query: Query<&Position>) { }
//! fn two_params(query: Query<&Position>, time: Res<Time>) { }
//! // ... up to 8 parameters
//! ```
//!
//! # Thread Safety
//!
//! Function systems are `Send` if all their parameters and states are `Send`.
//! This enables parallel scheduling in the future.

pub mod core;
pub mod impls;

#[cfg(test)]
mod tests;

// Core types re-exported at this level
pub use core::{FunctionSystem, SystemParamFunction};
