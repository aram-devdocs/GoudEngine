//! Type registration tests for FFI component operations.

use super::{goud_component_register_type, TestComponent, TEST_TYPE_ID};

// ============================================================================
// Type Registration Tests
// ============================================================================

#[test]
fn test_register_type_basic() {
    // Use a unique type ID to avoid conflicts with other tests
    const UNIQUE_TYPE_ID: u64 = TEST_TYPE_ID + 1000;
    let name = b"TestComponent";
    // SAFETY: name is a valid non-null pointer to a byte slice; size and align match TestComponent.
    let _result = unsafe {
        goud_component_register_type(
            UNIQUE_TYPE_ID,
            name.as_ptr(),
            name.len(),
            std::mem::size_of::<TestComponent>(),
            std::mem::align_of::<TestComponent>(),
        )
    };
    // First registration should succeed (or may be false if already registered in other tests)
    // This is fine - the registry is global across all tests

    // Second registration should return false
    // SAFETY: name is a valid non-null pointer to a byte slice; size and align match TestComponent.
    let result2 = unsafe {
        goud_component_register_type(
            UNIQUE_TYPE_ID,
            name.as_ptr(),
            name.len(),
            std::mem::size_of::<TestComponent>(),
            std::mem::align_of::<TestComponent>(),
        )
    };
    assert!(!result2, "Second registration should return false");
}

#[test]
fn test_register_type_null_name() {
    // SAFETY: Passing null for name is explicitly testing that the function handles null gracefully.
    let result = unsafe {
        goud_component_register_type(
            TEST_TYPE_ID + 1,
            std::ptr::null(),
            0,
            std::mem::size_of::<TestComponent>(),
            std::mem::align_of::<TestComponent>(),
        )
    };
    assert!(!result, "Registration with null name should fail");
}
