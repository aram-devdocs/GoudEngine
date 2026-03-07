//! Shared setup utilities for integration tests.
//!
//! Provides helper functions for creating and cleaning up FFI contexts
//! so that each test is self-contained.

use goud_engine::context_registry::GOUD_INVALID_CONTEXT_ID;
use goud_engine::ffi::context::{goud_context_create, goud_context_destroy, GoudContextId};

/// Creates a new FFI context, panicking if creation fails.
///
/// The returned context ID must be cleaned up via [`cleanup_context`].
pub fn create_test_context() -> GoudContextId {
    let ctx = goud_context_create();
    assert_ne!(
        ctx, GOUD_INVALID_CONTEXT_ID,
        "Failed to create test context"
    );
    ctx
}

/// Destroys a previously created FFI context.
///
/// Panics if the context was already destroyed or is invalid.
pub fn cleanup_context(ctx: GoudContextId) {
    let destroyed = goud_context_destroy(ctx);
    assert!(
        destroyed,
        "Failed to destroy test context {:?} - was it already destroyed?",
        ctx
    );
}
