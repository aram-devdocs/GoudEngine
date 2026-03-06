//! Sparse set data structure for component storage.
//!
//! Re-exports all public types so that the public API is identical to the
//! former single-file `sparse_set.rs` module.

mod core;
mod iter;
mod ops;
mod tests;

pub use core::SparseSet;
pub use iter::{SparseSetIter, SparseSetIterMut};
