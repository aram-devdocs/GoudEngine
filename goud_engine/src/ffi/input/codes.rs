//! FFI-compatible input code constants.
//!
//! Key codes and mouse button constants that map directly to GLFW values.

/// FFI-compatible key code.
///
/// These map directly to GLFW key codes for compatibility.
pub type GoudKeyCode = i32;

// Key code constants (matching GLFW key codes).
// These are self-documenting by name and intentionally have no docs.
#[allow(missing_docs)]
mod key_codes {
    use super::GoudKeyCode;
    pub const KEY_UNKNOWN: GoudKeyCode = -1;
    pub const KEY_SPACE: GoudKeyCode = 32;
    pub const KEY_APOSTROPHE: GoudKeyCode = 39;
    pub const KEY_COMMA: GoudKeyCode = 44;
    pub const KEY_MINUS: GoudKeyCode = 45;
    pub const KEY_PERIOD: GoudKeyCode = 46;
    pub const KEY_SLASH: GoudKeyCode = 47;
    pub const KEY_0: GoudKeyCode = 48;
    pub const KEY_1: GoudKeyCode = 49;
    pub const KEY_2: GoudKeyCode = 50;
    pub const KEY_3: GoudKeyCode = 51;
    pub const KEY_4: GoudKeyCode = 52;
    pub const KEY_5: GoudKeyCode = 53;
    pub const KEY_6: GoudKeyCode = 54;
    pub const KEY_7: GoudKeyCode = 55;
    pub const KEY_8: GoudKeyCode = 56;
    pub const KEY_9: GoudKeyCode = 57;
    pub const KEY_A: GoudKeyCode = 65;
    pub const KEY_B: GoudKeyCode = 66;
    pub const KEY_C: GoudKeyCode = 67;
    pub const KEY_D: GoudKeyCode = 68;
    pub const KEY_E: GoudKeyCode = 69;
    pub const KEY_F: GoudKeyCode = 70;
    pub const KEY_G: GoudKeyCode = 71;
    pub const KEY_H: GoudKeyCode = 72;
    pub const KEY_I: GoudKeyCode = 73;
    pub const KEY_J: GoudKeyCode = 74;
    pub const KEY_K: GoudKeyCode = 75;
    pub const KEY_L: GoudKeyCode = 76;
    pub const KEY_M: GoudKeyCode = 77;
    pub const KEY_N: GoudKeyCode = 78;
    pub const KEY_O: GoudKeyCode = 79;
    pub const KEY_P: GoudKeyCode = 80;
    pub const KEY_Q: GoudKeyCode = 81;
    pub const KEY_R: GoudKeyCode = 82;
    pub const KEY_S: GoudKeyCode = 83;
    pub const KEY_T: GoudKeyCode = 84;
    pub const KEY_U: GoudKeyCode = 85;
    pub const KEY_V: GoudKeyCode = 86;
    pub const KEY_W: GoudKeyCode = 87;
    pub const KEY_X: GoudKeyCode = 88;
    pub const KEY_Y: GoudKeyCode = 89;
    pub const KEY_Z: GoudKeyCode = 90;
    pub const KEY_ESCAPE: GoudKeyCode = 256;
    pub const KEY_ENTER: GoudKeyCode = 257;
    pub const KEY_TAB: GoudKeyCode = 258;
    pub const KEY_BACKSPACE: GoudKeyCode = 259;
    pub const KEY_INSERT: GoudKeyCode = 260;
    pub const KEY_DELETE: GoudKeyCode = 261;
    pub const KEY_RIGHT: GoudKeyCode = 262;
    pub const KEY_LEFT: GoudKeyCode = 263;
    pub const KEY_DOWN: GoudKeyCode = 264;
    pub const KEY_UP: GoudKeyCode = 265;
    pub const KEY_PAGE_UP: GoudKeyCode = 266;
    pub const KEY_PAGE_DOWN: GoudKeyCode = 267;
    pub const KEY_HOME: GoudKeyCode = 268;
    pub const KEY_END: GoudKeyCode = 269;
    pub const KEY_F1: GoudKeyCode = 290;
    pub const KEY_F2: GoudKeyCode = 291;
    pub const KEY_F3: GoudKeyCode = 292;
    pub const KEY_F4: GoudKeyCode = 293;
    pub const KEY_F5: GoudKeyCode = 294;
    pub const KEY_F6: GoudKeyCode = 295;
    pub const KEY_F7: GoudKeyCode = 296;
    pub const KEY_F8: GoudKeyCode = 297;
    pub const KEY_F9: GoudKeyCode = 298;
    pub const KEY_F10: GoudKeyCode = 299;
    pub const KEY_F11: GoudKeyCode = 300;
    pub const KEY_F12: GoudKeyCode = 301;
    pub const KEY_LEFT_SHIFT: GoudKeyCode = 340;
    pub const KEY_LEFT_CONTROL: GoudKeyCode = 341;
    pub const KEY_LEFT_ALT: GoudKeyCode = 342;
    pub const KEY_LEFT_SUPER: GoudKeyCode = 343;
    pub const KEY_RIGHT_SHIFT: GoudKeyCode = 344;
    pub const KEY_RIGHT_CONTROL: GoudKeyCode = 345;
    pub const KEY_RIGHT_ALT: GoudKeyCode = 346;
    pub const KEY_RIGHT_SUPER: GoudKeyCode = 347;
}
pub use key_codes::*;

/// FFI-compatible mouse button code.
pub type GoudMouseButton = i32;

// Mouse button constants (matching GLFW button codes).
#[allow(missing_docs)]
mod mouse_buttons {
    use super::GoudMouseButton;
    pub const MOUSE_BUTTON_LEFT: GoudMouseButton = 0;
    pub const MOUSE_BUTTON_RIGHT: GoudMouseButton = 1;
    pub const MOUSE_BUTTON_MIDDLE: GoudMouseButton = 2;
    pub const MOUSE_BUTTON_4: GoudMouseButton = 3;
    pub const MOUSE_BUTTON_5: GoudMouseButton = 4;
    pub const MOUSE_BUTTON_6: GoudMouseButton = 5;
    pub const MOUSE_BUTTON_7: GoudMouseButton = 6;
    pub const MOUSE_BUTTON_8: GoudMouseButton = 7;
}
pub use mouse_buttons::*;

/// FFI-compatible gamepad button code.
pub type GoudGamepadButton = u32;

/// FFI-compatible gamepad axis code.
pub type GoudGamepadAxis = u32;

// Gamepad button constants (matching GamepadButton repr(u32) discriminants).
#[allow(missing_docs)]
mod gamepad_buttons {
    use super::GoudGamepadButton;
    /// A / Cross
    pub const GAMEPAD_BUTTON_SOUTH: GoudGamepadButton = 0;
    /// B / Circle
    pub const GAMEPAD_BUTTON_EAST: GoudGamepadButton = 1;
    /// X / Square
    pub const GAMEPAD_BUTTON_WEST: GoudGamepadButton = 2;
    /// Y / Triangle
    pub const GAMEPAD_BUTTON_NORTH: GoudGamepadButton = 3;
    pub const GAMEPAD_BUTTON_LEFT_BUMPER: GoudGamepadButton = 4;
    pub const GAMEPAD_BUTTON_RIGHT_BUMPER: GoudGamepadButton = 5;
    pub const GAMEPAD_BUTTON_BACK: GoudGamepadButton = 6;
    pub const GAMEPAD_BUTTON_START: GoudGamepadButton = 7;
    pub const GAMEPAD_BUTTON_GUIDE: GoudGamepadButton = 8;
    pub const GAMEPAD_BUTTON_LEFT_STICK: GoudGamepadButton = 9;
    pub const GAMEPAD_BUTTON_RIGHT_STICK: GoudGamepadButton = 10;
    pub const GAMEPAD_BUTTON_DPAD_UP: GoudGamepadButton = 11;
    pub const GAMEPAD_BUTTON_DPAD_RIGHT: GoudGamepadButton = 12;
    pub const GAMEPAD_BUTTON_DPAD_DOWN: GoudGamepadButton = 13;
    pub const GAMEPAD_BUTTON_DPAD_LEFT: GoudGamepadButton = 14;
}
pub use gamepad_buttons::*;

// Gamepad axis constants (matching GamepadAxis repr(u32) discriminants).
#[allow(missing_docs)]
mod gamepad_axes {
    use super::GoudGamepadAxis;
    pub const GAMEPAD_AXIS_LEFT_X: GoudGamepadAxis = 0;
    pub const GAMEPAD_AXIS_LEFT_Y: GoudGamepadAxis = 1;
    pub const GAMEPAD_AXIS_RIGHT_X: GoudGamepadAxis = 2;
    pub const GAMEPAD_AXIS_RIGHT_Y: GoudGamepadAxis = 3;
    pub const GAMEPAD_AXIS_LEFT_TRIGGER: GoudGamepadAxis = 4;
    pub const GAMEPAD_AXIS_RIGHT_TRIGGER: GoudGamepadAxis = 5;
}
pub use gamepad_axes::*;
