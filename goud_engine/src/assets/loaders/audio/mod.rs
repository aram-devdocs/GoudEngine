//! Audio asset loading (stub implementation).
//!
//! This module provides basic types for audio assets. Full audio decoding
//! and playback will be implemented in Phase 6 with the rodio integration.

pub mod asset;
pub mod format;
pub mod loader;
pub mod settings;

#[cfg(test)]
mod tests;

pub use asset::{AudioAsset, AudioData};
pub use format::AudioFormat;
pub use loader::AudioLoader;
pub use settings::AudioSettings;
