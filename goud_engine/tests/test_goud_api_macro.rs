//! Integration test for the `#[goud_api]` proc-macro.
//!
//! This test verifies that the macro expands correctly and produces
//! compilable FFI wrappers. It uses a test-only struct to avoid
//! modifying production code.

/// A test helper struct that simulates a context with simple methods.
struct TestApi;

/// Verify that the macro compiles when applied to an impl block
/// with a simple no-self method returning a primitive.
#[goud_engine_macros::goud_api(module = "test_macro")]
impl TestApi {
    /// A simple static method returning a constant.
    pub fn get_version() -> u32 {
        42
    }

    /// A method with a parameter.
    pub fn add_numbers(a: u32, b: u32) -> u32 {
        a + b
    }
}

// The macro generates `#[no_mangle] pub extern "C"` functions inside
// a `__goud_generated_ffi` module. We reference them via `extern "C"` to
// exercise the full FFI pipeline.
extern "C" {
    fn goud_test_macro_get_version() -> u32;
    fn goud_test_macro_add_numbers(a: u32, b: u32) -> u32;
}

#[test]
fn test_macro_generates_static_ffi_function() {
    // SAFETY: calling our own generated no_mangle extern C function
    let result = unsafe { goud_test_macro_get_version() };
    assert_eq!(result, 42);
}

#[test]
fn test_macro_generates_ffi_with_params() {
    // SAFETY: calling our own generated no_mangle extern C function
    let result = unsafe { goud_test_macro_add_numbers(10, 32) };
    assert_eq!(result, 42);
}
