//! Shared helpers for FFI safety tests.

use goud_engine::ffi::context::{goud_context_create, goud_context_destroy, goud_context_is_valid};

/// Sentinel for an invalid entity (matches `GOUD_INVALID_ENTITY_ID`).
pub const INVALID_ENTITY: u64 = u64::MAX;

/// Re-export context type for convenience.
pub use goud_engine::ffi::context::GoudContextId;

/// Re-export GoudEntityId for component operations.
pub use goud_engine::ffi::GoudEntityId;

/// Re-export the invalid context sentinel.
pub use goud_engine::ffi::GOUD_INVALID_CONTEXT_ID;

/// Creates a test context, panicking if creation fails.
pub fn create_test_context() -> GoudContextId {
    let id = goud_context_create();
    assert!(
        !id.is_invalid(),
        "goud_context_create() returned INVALID -- cannot proceed"
    );
    id
}

/// Destroys a context, asserting success.
pub fn cleanup_context(id: GoudContextId) {
    let ok = goud_context_destroy(id);
    assert!(ok, "goud_context_destroy({id}) failed unexpectedly");
}

/// Verifies a context is no longer valid.
#[allow(dead_code)]
pub fn assert_context_invalid(id: GoudContextId) {
    let valid = goud_context_is_valid(id);
    assert!(
        !valid,
        "expected context {id} to be invalid, but is_valid returned true"
    );
}
