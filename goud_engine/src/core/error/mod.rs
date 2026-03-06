//! Error handling infrastructure for GoudEngine.
//!
//! This module re-exports from `crate::libs::error` which is the canonical location.
//! Import from here or from `crate::libs::error` -- both work identically.

pub use crate::libs::error::*;

#[cfg(test)]
mod tests {
    mod codes_tests;
    mod context_errors;
    mod entity_errors;
    mod ffi_tests;
    mod graphics_errors;
    mod internal_errors;
    mod resource_errors;
    mod system_errors;
    mod traits;
}
