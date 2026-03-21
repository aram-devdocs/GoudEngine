//! Tests for FFI component operations.
//!
//! Tests are organized into submodules:
//! - `tests_registration` — type registration
//! - `tests_single` — single-entity add/remove/has/get operations
//! - `tests_batch` — batch add/remove/has operations

use crate::ecs::Component;
use crate::ffi::context::goud_context_create;
use crate::ffi::GoudContextId;

use super::access::{goud_component_get, goud_component_get_mut, goud_component_has};
use super::batch::{
    goud_component_add_batch, goud_component_has_batch, goud_component_remove_batch,
};
use super::ops::{goud_component_add, goud_component_register_type, goud_component_remove};
use super::query::{goud_component_count, goud_component_get_all, goud_component_get_entities};

// ---------------------------------------------------------------------------
// Shared test helpers
// ---------------------------------------------------------------------------

/// Minimal component used in all FFI component tests.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub(super) struct TestComponent {
    pub(super) x: f32,
    pub(super) y: f32,
}

impl Component for TestComponent {}

pub(super) const TEST_TYPE_ID: u64 = 12345;

pub(super) fn setup_test_context() -> GoudContextId {
    // SAFETY: goud_context_create has no safety preconditions; it always returns a valid context.
    unsafe { goud_context_create() }
}

/// Registers `TestComponent` under `type_id` (silently ignores duplicate registration).
pub(super) fn register_test_type(type_id: u64) {
    let name = b"TestComponent";
    // SAFETY: name is a valid non-null pointer to a UTF-8 byte slice; size and align match the TestComponent type layout.
    unsafe {
        goud_component_register_type(
            type_id,
            name.as_ptr(),
            name.len(),
            std::mem::size_of::<TestComponent>(),
            std::mem::align_of::<TestComponent>(),
        );
    }
}

// ---------------------------------------------------------------------------
// Sub-modules
// ---------------------------------------------------------------------------

mod tests_batch;
mod tests_query;
mod tests_registration;
mod tests_single;
