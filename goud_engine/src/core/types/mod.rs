//! # Core FFI-Safe Type Definitions
//!
//! This module defines FFI-safe types used throughout the engine for
//! cross-language interoperability. All types use `#[repr(C)]` for
//! predictable memory layout and primitive types for ABI stability.
//!
//! These types are the canonical definitions. The `ffi/` layer re-exports
//! them to preserve backward compatibility with generated bindings.

mod entity;
mod math_types;
mod result;
mod sprite;
mod transform;

#[cfg(test)]
mod tests;

pub use entity::GoudEntityId;
pub use math_types::{FfiColor, FfiRect, FfiVec2};
pub use result::GoudResult;
pub use sprite::{FfiSprite, FfiSpriteBuilder, GoudContact};
pub use transform::{FfiMat3x3, FfiTransform2D, FfiTransform2DBuilder};
