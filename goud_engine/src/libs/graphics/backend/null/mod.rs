//! Null (headless) render backend for GPU-free testing.

mod backend;
#[cfg(test)]
mod tests;
mod trait_impl;

pub use backend::NullBackend;
