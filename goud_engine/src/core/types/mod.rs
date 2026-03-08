//! # Core FFI-Safe Type Definitions
//!
//! This module defines FFI-safe types used throughout the engine for
//! cross-language interoperability. All types use `#[repr(C)]` for
//! predictable memory layout and primitive types for ABI stability.
//!
//! These types are the canonical definitions. The `ffi/` layer re-exports
//! them to preserve backward compatibility with generated bindings.

mod animation;
mod entity;
mod ffi_text;
mod math_types;
mod result;
mod sprite;
mod transform;

#[cfg(test)]
mod tests;

pub use animation::{FfiAnimationClipBuilder, FfiPlaybackMode, FfiSpriteAnimator};
pub use entity::GoudEntityId;
pub use ffi_text::FfiText;
pub use math_types::{FfiColor, FfiRect, FfiVec2};
pub use result::GoudResult;
pub use sprite::{FfiSprite, FfiSpriteBuilder, GoudContact};
pub use transform::{FfiMat3x3, FfiTransform2D, FfiTransform2DBuilder};
