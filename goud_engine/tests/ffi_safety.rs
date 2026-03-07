//! FFI safety integration tests.
//!
//! Exercises null pointers, invalid handles, double-free, and
//! use-after-free scenarios against the public C API surface to
//! verify that every function returns a well-defined error rather
//! than invoking undefined behavior.

mod ffi_safety {
    pub mod helpers;
    pub mod invalid_handle_tests;
    pub mod lifecycle_safety_tests;
    pub mod null_pointer_tests;
}
