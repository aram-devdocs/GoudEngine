//! Test helpers for GPU-free testing.

use crate::libs::graphics::backend::null::NullBackend;

/// Creates a headless render backend for tests that need a `RenderBackend`.
pub fn init_test_context() -> NullBackend {
    NullBackend::new()
}
