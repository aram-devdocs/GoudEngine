//! # FFI Window Management
//!
//! This module provides C-compatible functions for window creation, event handling,
//! and game loop management. It integrates the platform backend with the ECS
//! context system.
//!
//! ## Design
//!
//! Window operations are integrated into the context system. When you create a
//! windowed context, it includes:
//! - A [`GlfwPlatform`] backend (window + input polling)
//! - An InputManager (as an ECS resource)
//! - An OpenGL rendering backend
//!
//! The platform-specific code lives in [`GlfwPlatform`]; this module handles
//! FFI marshalling and context integration.
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // Create a windowed context
//! var contextId = goud_window_create(800, 600, "My Game");
//! if (contextId == GOUD_INVALID_CONTEXT_ID) {
//!     Console.WriteLine("Failed to create window");
//!     return;
//! }
//!
//! // Game loop
//! while (!goud_window_should_close(contextId)) {
//!     float deltaTime = goud_window_poll_events(contextId);
//!
//!     // Update game logic...
//!     // Rendering will be handled by renderer FFI module
//!
//!     goud_window_swap_buffers(contextId);
//! }
//!
//! goud_window_destroy(contextId);
//! ```
//!
//! ## Submodules
//!
//! - [`state`] — [`WindowState`] struct and thread-local storage
//! - [`lifecycle`] — window creation and destruction FFI functions
//! - [`properties`] — per-frame query and mutation FFI functions

mod lifecycle;
mod properties;
mod state;

// Re-export the public surface so callers using `ffi::window::*` see the
// same symbols as before the split.
pub use state::{with_window_state, WindowState};

// FFI functions are `#[no_mangle]` and therefore globally visible, but we
// also re-export them so that `pub mod window` keeps them reachable via the
// module path for any internal callers.
pub use lifecycle::{goud_window_create, goud_window_destroy};
pub use properties::{
    goud_window_clear, goud_window_get_delta_time, goud_window_get_size, goud_window_poll_events,
    goud_window_set_should_close, goud_window_should_close, goud_window_swap_buffers,
};
