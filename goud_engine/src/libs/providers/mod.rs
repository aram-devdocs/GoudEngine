//! Provider traits -- re-exported from `core/providers/` for convenience.
//!
//! Trait definitions and shared types are canonical in `crate::core::providers`.
//! This module re-exports them so existing `crate::libs::providers::` import
//! paths continue to work, and hosts the concrete implementations in `impls/`.

pub use crate::core::providers::audio;
pub use crate::core::providers::input;
pub use crate::core::providers::input_types;
pub use crate::core::providers::physics;
pub use crate::core::providers::render;
pub use crate::core::providers::types;
pub use crate::core::providers::window;
pub use crate::core::providers::{Provider, ProviderLifecycle};

pub mod impls;
