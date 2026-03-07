//! Test helpers for GPU-free testing.
//!
//! This module provides utilities for running tests without requiring a real GPU or OpenGL context.
//! Tests using these helpers can run in headless CI environments where graphics acceleration is unavailable.
//!
//! # Running Headless Tests
//!
//! To run the entire test suite with the headless backend:
//!
//! ```bash
//! cargo test --features headless
//! ```
//!
//! # Using init_test_context()
//!
//! The [`init_test_context()`] function creates a `NullBackend` — a no-op render backend that
//! satisfies the `RenderBackend` trait without allocating GPU resources. Use this in tests that
//! need a render backend but don't care about actual rendering:
//!
//! # Example
//!
//! ```ignore
//! #[test]
//! fn test_renderer_initialization() {
//!     let backend = init_test_context();
//!     // backend is now a NullBackend that can be passed to code expecting a RenderBackend
//!     // No GPU operations will occur; the backend silently succeeds all calls
//! }
//! ```

use crate::libs::graphics::backend::null::NullBackend;

/// Creates a headless render backend for tests that need a `RenderBackend`.
pub fn init_test_context() -> NullBackend {
    NullBackend::new()
}
