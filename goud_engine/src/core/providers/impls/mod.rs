//! Null (no-op) provider implementations for headless testing.
//!
//! These live in the Foundation layer so that `core/providers/registry.rs`
//! and `core/providers/builder.rs` can use them as defaults without
//! importing from the Libs layer.

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
