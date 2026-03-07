//! FFI-safe result type for engine operations.
//!
//! `GoudResult` is a type alias for [`GoudFFIResult`], the canonical
//! FFI-safe result type defined in `core::error::ffi_bridge`.

/// FFI-safe result type for operations that can fail.
///
/// This is an alias for [`crate::core::error::GoudFFIResult`] to avoid
/// duplicating the type definition. All methods (`ok()`, `err()`,
/// `is_ok()`, `is_err()`) are available through the canonical type.
pub type GoudResult = crate::core::error::GoudFFIResult;
