//! # FFI Input Module
//!
//! This module provides C-compatible functions for querying input state.
//! It integrates with the ECS InputManager resource to expose keyboard,
//! mouse, and gamepad state to the C# SDK.
//!
//! ## Design
//!
//! The input FFI provides query functions that read from the InputManager
//! resource. The InputManager is updated by the window FFI during event polling.
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // In game loop after goud_window_poll_events:
//! if (goud_input_key_pressed(contextId, KeyCode.W)) {
//!     MoveForward(speed * deltaTime);
//! }
//!
//! if (goud_input_key_just_pressed(contextId, KeyCode.Space)) {
//!     Jump();
//! }
//!
//! float mouseX, mouseY;
//! goud_input_get_mouse_position(contextId, out mouseX, out mouseY);
//! ```

mod actions;
mod codes;
mod helpers;
mod keyboard;
mod mouse;

// Re-export type aliases and constants so callers see the same public API.
pub use codes::{
    GoudKeyCode, GoudMouseButton, KEY_0, KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8,
    KEY_9, KEY_A, KEY_APOSTROPHE, KEY_B, KEY_BACKSPACE, KEY_C, KEY_COMMA, KEY_D, KEY_DELETE,
    KEY_DOWN, KEY_E, KEY_END, KEY_ENTER, KEY_ESCAPE, KEY_F, KEY_F1, KEY_F10, KEY_F11, KEY_F12,
    KEY_F2, KEY_F3, KEY_F4, KEY_F5, KEY_F6, KEY_F7, KEY_F8, KEY_F9, KEY_G, KEY_H, KEY_HOME, KEY_I,
    KEY_INSERT, KEY_J, KEY_K, KEY_L, KEY_LEFT, KEY_LEFT_ALT, KEY_LEFT_CONTROL, KEY_LEFT_SHIFT,
    KEY_LEFT_SUPER, KEY_M, KEY_MINUS, KEY_N, KEY_O, KEY_P, KEY_PAGE_DOWN, KEY_PAGE_UP, KEY_PERIOD,
    KEY_Q, KEY_R, KEY_RIGHT, KEY_RIGHT_ALT, KEY_RIGHT_CONTROL, KEY_RIGHT_SHIFT, KEY_RIGHT_SUPER,
    KEY_S, KEY_SLASH, KEY_SPACE, KEY_T, KEY_TAB, KEY_U, KEY_UNKNOWN, KEY_UP, KEY_V, KEY_W, KEY_X,
    KEY_Y, KEY_Z, MOUSE_BUTTON_4, MOUSE_BUTTON_5, MOUSE_BUTTON_6, MOUSE_BUTTON_7, MOUSE_BUTTON_8,
    MOUSE_BUTTON_LEFT, MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT,
};

// Re-export all FFI functions.
pub use actions::{
    goud_input_action_just_pressed, goud_input_action_just_released, goud_input_action_pressed,
    goud_input_map_action_key,
};
pub use keyboard::{
    goud_input_key_just_pressed, goud_input_key_just_released, goud_input_key_pressed,
};
pub use mouse::{
    goud_input_get_mouse_delta, goud_input_get_mouse_position, goud_input_get_scroll_delta,
    goud_input_mouse_button_just_pressed, goud_input_mouse_button_just_released,
    goud_input_mouse_button_pressed,
};
