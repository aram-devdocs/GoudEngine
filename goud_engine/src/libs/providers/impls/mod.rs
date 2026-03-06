//! Concrete provider implementations.
//!
//! Currently provides null (no-op) implementations for headless testing
//! and as defaults before real backends are configured.

pub mod null_audio;
pub mod null_input;
pub mod null_physics;
pub mod null_render;
pub mod null_window;

pub use null_audio::NullAudioProvider;
pub use null_input::NullInputProvider;
pub use null_physics::NullPhysicsProvider;
pub use null_render::NullRenderProvider;
pub use null_window::NullWindowProvider;
