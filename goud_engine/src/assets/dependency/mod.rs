//! Asset dependency tracking for cascade reloading.
//!
//! This module provides the [`DependencyGraph`] which tracks relationships
//! between assets so that when one asset changes, all assets that depend
//! on it can be reloaded in the correct order.

mod graph;

#[cfg(test)]
mod tests;

pub use graph::{CycleError, DependencyGraph};
