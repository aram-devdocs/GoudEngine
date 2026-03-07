//! Concrete provider implementations.
//!
//! Null (no-op) implementations are canonical in `crate::core::providers::impls`
//! and re-exported here for backward compatibility. Native (GLFW/OpenGL/rodio)
//! implementations live here because they depend on Libs-layer crates.

// Re-export null providers from core (Foundation layer)
pub use crate::core::providers::impls::null_audio;
pub use crate::core::providers::impls::null_input;
pub use crate::core::providers::impls::null_physics;
pub use crate::core::providers::impls::null_render;
pub use crate::core::providers::impls::null_window;

pub use crate::core::providers::impls::NullAudioProvider;
pub use crate::core::providers::impls::NullInputProvider;
pub use crate::core::providers::impls::NullPhysicsProvider;
pub use crate::core::providers::impls::NullRenderProvider;
pub use crate::core::providers::impls::NullWindowProvider;

#[cfg(feature = "native")]
pub mod glfw_input;
#[cfg(feature = "native")]
pub mod glfw_window;
#[cfg(feature = "native")]
pub mod opengl_render;
#[cfg(feature = "native")]
pub mod rodio_audio;

#[cfg(feature = "native")]
pub use glfw_input::GlfwInputProvider;
#[cfg(feature = "native")]
pub use glfw_window::GlfwWindowProvider;
#[cfg(feature = "native")]
pub use opengl_render::OpenGLRenderProvider;
#[cfg(feature = "native")]
pub use rodio_audio::RodioAudioProvider;
