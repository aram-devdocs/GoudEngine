//! Platform-independent input types for provider traits.
//!
//! These types are independent of GLFW and other platform-specific input
//! libraries, allowing input providers to work without a windowing dependency.

/// Platform-independent key code for keyboard input.
///
/// This enum is independent of GLFW's `Key` type, allowing input providers
/// to work without a windowing dependency. Concrete providers convert between
/// this type and their platform-specific key codes.
///
/// Values follow the GLFW key code convention for easy mapping.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum KeyCode {
    Unknown = 0,
    Space = 32,
    Apostrophe = 39,
    Comma = 44,
    Minus = 45,
    Period = 46,
    Slash = 47,
    Num0 = 48,
    Num1 = 49,
    Num2 = 50,
    Num3 = 51,
    Num4 = 52,
    Num5 = 53,
    Num6 = 54,
    Num7 = 55,
    Num8 = 56,
    Num9 = 57,
    Semicolon = 59,
    Equal = 61,
    A = 65,
    B = 66,
    C = 67,
    D = 68,
    E = 69,
    F = 70,
    G = 71,
    H = 72,
    I = 73,
    J = 74,
    K = 75,
    L = 76,
    M = 77,
    N = 78,
    O = 79,
    P = 80,
    Q = 81,
    R = 82,
    S = 83,
    T = 84,
    U = 85,
    V = 86,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    Escape = 256,
    Enter = 257,
    KpEnter = 335,
    Tab = 258,
    Backspace = 259,
    Insert = 260,
    Delete = 261,
    Right = 262,
    Left = 263,
    Down = 264,
    Up = 265,
    PageUp = 266,
    PageDown = 267,
    Home = 268,
    End = 269,
    F1 = 290,
    F2 = 291,
    F3 = 292,
    F4 = 293,
    F5 = 294,
    F6 = 295,
    F7 = 296,
    F8 = 297,
    F9 = 298,
    F10 = 299,
    F11 = 300,
    F12 = 301,
    LeftShift = 340,
    LeftControl = 341,
    LeftAlt = 342,
    LeftSuper = 343,
    RightShift = 344,
    RightControl = 345,
    RightAlt = 346,
    RightSuper = 347,
}

impl KeyCode {
    /// Converts a raw engine/FFI key value into a key code.
    pub fn from_u32(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Unknown,
            32 => Self::Space,
            39 => Self::Apostrophe,
            44 => Self::Comma,
            45 => Self::Minus,
            46 => Self::Period,
            47 => Self::Slash,
            48 => Self::Num0,
            49 => Self::Num1,
            50 => Self::Num2,
            51 => Self::Num3,
            52 => Self::Num4,
            53 => Self::Num5,
            54 => Self::Num6,
            55 => Self::Num7,
            56 => Self::Num8,
            57 => Self::Num9,
            59 => Self::Semicolon,
            61 => Self::Equal,
            65 => Self::A,
            66 => Self::B,
            67 => Self::C,
            68 => Self::D,
            69 => Self::E,
            70 => Self::F,
            71 => Self::G,
            72 => Self::H,
            73 => Self::I,
            74 => Self::J,
            75 => Self::K,
            76 => Self::L,
            77 => Self::M,
            78 => Self::N,
            79 => Self::O,
            80 => Self::P,
            81 => Self::Q,
            82 => Self::R,
            83 => Self::S,
            84 => Self::T,
            85 => Self::U,
            86 => Self::V,
            87 => Self::W,
            88 => Self::X,
            89 => Self::Y,
            90 => Self::Z,
            256 => Self::Escape,
            257 => Self::Enter,
            258 => Self::Tab,
            259 => Self::Backspace,
            260 => Self::Insert,
            261 => Self::Delete,
            262 => Self::Right,
            263 => Self::Left,
            264 => Self::Down,
            265 => Self::Up,
            266 => Self::PageUp,
            267 => Self::PageDown,
            268 => Self::Home,
            269 => Self::End,
            290 => Self::F1,
            291 => Self::F2,
            292 => Self::F3,
            293 => Self::F4,
            294 => Self::F5,
            295 => Self::F6,
            296 => Self::F7,
            297 => Self::F8,
            298 => Self::F9,
            299 => Self::F10,
            300 => Self::F11,
            301 => Self::F12,
            335 => Self::KpEnter,
            340 => Self::LeftShift,
            341 => Self::LeftControl,
            342 => Self::LeftAlt,
            343 => Self::LeftSuper,
            344 => Self::RightShift,
            345 => Self::RightControl,
            346 => Self::RightAlt,
            347 => Self::RightSuper,
            _ => return None,
        })
    }

    /// Legacy alias retained while engine-facing code migrates off GLFW symbols.
    pub const KP_ENTER: Self = Self::KpEnter;
}

/// Platform-independent mouse button identifier.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MouseButton {
    Left = 0,
    Right = 1,
    Middle = 2,
    Button4 = 3,
    Button5 = 4,
}

impl MouseButton {
    #[allow(non_upper_case_globals)]
    /// Converts a raw engine/FFI mouse button value into a mouse button.
    pub fn from_u32(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Middle,
            3 => Self::Button4,
            4 => Self::Button5,
            _ => return None,
        })
    }

    #[allow(non_upper_case_globals)]
    /// Legacy GLFW-style aliases retained for compatibility during migration.
    pub const Button1: Self = Self::Left;
    #[allow(non_upper_case_globals)]
    /// Legacy GLFW-style aliases retained for compatibility during migration.
    pub const Button2: Self = Self::Right;
    #[allow(non_upper_case_globals)]
    /// Legacy GLFW-style aliases retained for compatibility during migration.
    pub const Button3: Self = Self::Middle;
}

/// Gamepad identifier (0-indexed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GamepadId(pub u32);

/// Platform-independent gamepad axis identifier.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GamepadAxis {
    LeftStickX = 0,
    LeftStickY = 1,
    RightStickX = 2,
    RightStickY = 3,
    LeftTrigger = 4,
    RightTrigger = 5,
}

impl GamepadAxis {
    #[allow(non_upper_case_globals)]
    /// Legacy GLFW-style aliases retained for compatibility during migration.
    pub const AxisLeftX: Self = Self::LeftStickX;
    #[allow(non_upper_case_globals)]
    /// Legacy GLFW-style aliases retained for compatibility during migration.
    pub const AxisLeftY: Self = Self::LeftStickY;
    #[allow(non_upper_case_globals)]
    /// Legacy GLFW-style aliases retained for compatibility during migration.
    pub const AxisRightX: Self = Self::RightStickX;
    #[allow(non_upper_case_globals)]
    /// Legacy GLFW-style aliases retained for compatibility during migration.
    pub const AxisRightY: Self = Self::RightStickY;
    #[allow(non_upper_case_globals)]
    /// Legacy GLFW-style aliases retained for compatibility during migration.
    pub const AxisLeftTrigger: Self = Self::LeftTrigger;
    #[allow(non_upper_case_globals)]
    /// Legacy GLFW-style aliases retained for compatibility during migration.
    pub const AxisRightTrigger: Self = Self::RightTrigger;
}

/// Platform-independent gamepad button identifier.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GamepadButton {
    South = 0,
    East = 1,
    West = 2,
    North = 3,
    LeftBumper = 4,
    RightBumper = 5,
    Back = 6,
    Start = 7,
    Guide = 8,
    LeftStick = 9,
    RightStick = 10,
    DPadUp = 11,
    DPadRight = 12,
    DPadDown = 13,
    DPadLeft = 14,
}

/// Capabilities reported by an input provider.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct InputCapabilities {
    /// Whether gamepad input is supported.
    pub supports_gamepad: bool,
    /// Whether touch input is supported.
    pub supports_touch: bool,
    /// Maximum number of simultaneous gamepads.
    pub max_gamepads: u32,
}
