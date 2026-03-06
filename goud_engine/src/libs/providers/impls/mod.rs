//! Concrete provider implementations.
//!
//! Provides null (no-op) implementations for headless testing and
//! native (GLFW/OpenGL/rodio) implementations for desktop platforms.

pub mod null_audio;
pub mod null_input;
pub mod null_physics;
pub mod null_render;
pub mod null_window;

#[cfg(feature = "native")]
pub mod glfw_input;
#[cfg(feature = "native")]
pub mod glfw_window;
#[cfg(feature = "native")]
pub mod opengl_render;
#[cfg(feature = "native")]
pub mod rodio_audio;

pub use null_audio::NullAudioProvider;
pub use null_input::NullInputProvider;
pub use null_physics::NullPhysicsProvider;
pub use null_render::NullRenderProvider;
pub use null_window::NullWindowProvider;

#[cfg(feature = "native")]
pub use glfw_input::GlfwInputProvider;
#[cfg(feature = "native")]
pub use glfw_window::GlfwWindowProvider;
#[cfg(feature = "native")]
pub use opengl_render::OpenGLRenderProvider;
#[cfg(feature = "native")]
pub use rodio_audio::RodioAudioProvider;
