use crate::game::{GameSdk, WindowBuilder};
use crate::types::{MousePosition, Rectangle};
use crate::types::{SpriteCreateDto, SpriteUpdateDto, UpdateResponseData};
use glfw::Key;
use std::ffi::{c_uint, CStr, CString};
use std::os::raw::{c_char, c_int};

/// Initializes a new game instance with the specified window settings and returns a raw pointer to the `GameSdk`.
///
/// # Arguments
/// * `width` - The width of the game window.
/// * `height` - The height of the game window.
/// * `title` - A pointer to the C-style string for the game window title.
/// * `target_fps` - Target frames per second for the game.
///
/// # Returns
/// * `*mut GameSdk` - A raw pointer to the newly created `GameSdk` instance.
#[no_mangle]
pub extern "C" fn game_create(
    width: u32,
    height: u32,
    title: *const c_char,
    target_fps: u32,
) -> *mut GameSdk {
    println!("Creating game instance");
    let title_str = unsafe { CStr::from_ptr(title).to_str().unwrap() };
    let title_cstring = CString::new(title_str).unwrap();
    let builder = WindowBuilder {
        width,
        height,
        title: title_cstring.as_ptr(),
        target_fps,
    };
    let game = GameSdk::new(builder);
    Box::into_raw(Box::new(game))
}

/// Initializes the game instance by setting up necessary resources.
#[no_mangle]
pub extern "C" fn game_initialize(game: *mut GameSdk) {
    let game = unsafe { &mut *game };
    game.init(|_| {});
}

/// Starts the game loop for the provided game instance.
#[no_mangle]
pub extern "C" fn game_start(game: *mut GameSdk) {
    let game = unsafe { &mut *game };
    game.start(|_| {});
}

/// Updates the game state and returns update response data, including delta time.
///
/// # Returns
/// * `UpdateResponseData` - Data related to the frame update, including delta time.
#[no_mangle]
pub extern "C" fn game_update(game: *mut GameSdk) -> UpdateResponseData {
    let game = unsafe { &mut *game };
    game.update(&|_| {});
    UpdateResponseData {
        delta_time: game.window.delta_time,
    }
}

/// Terminates the game instance, releasing all allocated resources.
#[no_mangle]
pub extern "C" fn game_terminate(game: *mut GameSdk) {
    if !game.is_null() {
        let game = unsafe { &mut *game };
        game.terminate();
        println!("Terminating game instance");
        unsafe {
            drop(Box::from_raw(game));
        }
    }
}

/// Adds a sprite to the game instance with specified properties.
///
/// # Arguments
/// * `data` - The data needed to create a sprite.
#[no_mangle]
pub extern "C" fn game_add_sprite(game: *mut GameSdk, data: SpriteCreateDto) -> u32 {
    let game = unsafe { &mut *game };
    let texture_clone = game.texture_manager.get_texture(data.texture_id).clone();

    let sprite = SpriteCreateDto::new(
        data.x,
        data.y,
        if data.z_layer == 0 { 0 } else { data.z_layer },
        if data.scale_x == 0.0 {
            1.0
        } else {
            data.scale_x
        },
        if data.scale_y == 0.0 {
            1.0
        } else {
            data.scale_y
        },
        if data.dimension_x == 0.0 {
            texture_clone.width() as f32
        } else {
            data.dimension_x
        },
        if data.dimension_y == 0.0 {
            texture_clone.height() as f32
        } else {
            data.dimension_y
        },
        data.rotation,
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        data.texture_id,
        data.debug,
    );

    let id = game.ecs.add_sprite(sprite);
    id
}

/// Loads a texture into the game and returns its ID.
///
/// # Arguments
/// * `texture_path` - Path to the texture file as a C string.
#[no_mangle]
pub extern "C" fn game_create_texture(game: *mut GameSdk, texture_path: *const c_char) -> c_uint {
    let game = unsafe { &mut *game };
    let texture_path_str = unsafe { CStr::from_ptr(texture_path).to_str().unwrap() };
    let texture_path_cstring = CString::new(texture_path_str).unwrap();
    game.texture_manager
        .create_texture(texture_path_cstring.as_ptr())
}

/// Updates an existing sprite with new properties.
#[no_mangle]
pub extern "C" fn game_update_sprite(game: *mut GameSdk, data: SpriteUpdateDto) {
    let game = unsafe { &mut *game };
    let sprite_ref = game.ecs.get_sprite(data.id).expect("Sprite not found");

    let sprite = SpriteUpdateDto::new(
        data.id,
        data.x,
        data.y,
        // TODO: We need to handle all of the == 0.0 cases as they can cause weird behavior. If I switch to 0, I will always be switched back to the initial value.
        if data.z_layer == 0 {
            sprite_ref.z_layer
        } else {
            data.z_layer
        },
        if data.scale_x == 0.0 {
            sprite_ref.scale_x
        } else {
            data.scale_x
        },
        if data.scale_y == 0.0 {
            sprite_ref.scale_y
        } else {
            data.scale_y
        },
        if data.dimension_x == 0.0 {
            sprite_ref.dimension_x
        } else {
            data.dimension_x
        },
        if data.dimension_y == 0.0 {
            sprite_ref.dimension_y
        } else {
            data.dimension_y
        },
        data.rotation,
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        if data.texture_id < 0 {
            sprite_ref.texture_id
        } else {
            data.texture_id
        },
        data.debug,
    );

    game.ecs
        .update_sprite(sprite)
        .expect("Failed to update sprite");
}

/// Removes a sprite from the game instance.
#[no_mangle]
pub extern "C" fn game_remove_sprite(game: *mut GameSdk, id: c_uint) {
    let game = unsafe { &mut *game };
    println!("Removing sprite with id: {}", id);
    game.ecs.remove_sprite(id).expect("Failed to remove sprite");
}

/// Checks if the specified key is currently pressed.
#[no_mangle]
pub extern "C" fn game_is_key_pressed(game: *mut GameSdk, key_code: c_int) -> bool {
    let game = unsafe { &*game };
    let key = from_glfw_key_code(key_code);
    game.window.is_key_pressed(key)
}

/// Checks if the specified mouse button is pressed.
#[no_mangle]
pub extern "C" fn game_is_mouse_button_pressed(game: *mut GameSdk, button: c_int) -> bool {
    let game = unsafe { &*game };
    let button = from_glfw_mouse_button(button);
    game.window.is_mouse_button_pressed(button)
}

/// Retrieves the current mouse position in the game window.
#[no_mangle]
pub extern "C" fn game_get_mouse_position(game: *mut GameSdk) -> MousePosition {
    let game = unsafe { &*game };
    let position = game.window.get_mouse_position();
    MousePosition {
        x: position.x,
        y: position.y,
    }
}

/// Handles gamepad button input events.
#[no_mangle]
pub extern "C" fn game_handle_gamepad_button(
    game: *mut GameSdk,
    gamepad_id: u32,
    button: u32,
    pressed: bool,
) {
    let game = unsafe { &mut *game };
    game.window
        .handle_gamepad_button(gamepad_id, button, pressed);
}

/// Checks if a gamepad button is currently pressed.
#[no_mangle]
pub extern "C" fn game_is_gamepad_button_pressed(
    game: *mut GameSdk,
    gamepad_id: u32,
    button: u32,
) -> bool {
    let game = unsafe { &*game };
    game.window.is_gamepad_button_pressed(gamepad_id, button)
}

/// Checks for collision between two sprites.
#[no_mangle]
pub extern "C" fn check_collision_between_sprites(
    game: *mut GameSdk,
    entity_id1: c_uint,
    entity_id2: c_uint,
) -> bool {
    let game = unsafe { &*game };
    game.ecs
        .check_collision_between_sprites(entity_id1, entity_id2)
}

/// Determines if the game window should close.
#[no_mangle]
pub extern "C" fn game_should_close(game: *mut GameSdk) -> bool {
    let game = unsafe { &mut *game };
    game.window.should_close()
}

// Helper Functions

/// Converts an integer key code to a `glfw::Key`.
fn from_glfw_key_code(key_code: c_int) -> Key {
    match key_code {
        // Alphabet keys
        65 => Key::A,
        66 => Key::B,
        67 => Key::C,
        68 => Key::D,
        69 => Key::E,
        70 => Key::F,
        71 => Key::G,
        72 => Key::H,
        73 => Key::I,
        74 => Key::J,
        75 => Key::K,
        76 => Key::L,
        77 => Key::M,
        78 => Key::N,
        79 => Key::O,
        80 => Key::P,
        81 => Key::Q,
        82 => Key::R,
        83 => Key::S,
        84 => Key::T,
        85 => Key::U,
        86 => Key::V,
        87 => Key::W,
        88 => Key::X,
        89 => Key::Y,
        90 => Key::Z,

        // Number keys
        48 => Key::Num0,
        49 => Key::Num1,
        50 => Key::Num2,
        51 => Key::Num3,
        52 => Key::Num4,
        53 => Key::Num5,
        54 => Key::Num6,
        55 => Key::Num7,
        56 => Key::Num8,
        57 => Key::Num9,

        // Function keys
        290 => Key::F1,
        291 => Key::F2,
        292 => Key::F3,
        293 => Key::F4,
        294 => Key::F5,
        295 => Key::F6,
        296 => Key::F7,
        297 => Key::F8,
        298 => Key::F9,
        299 => Key::F10,
        300 => Key::F11,
        301 => Key::F12,

        // Control keys
        256 => Key::Escape,
        257 => Key::Enter,
        258 => Key::Tab,
        259 => Key::Backspace,
        260 => Key::Insert,
        261 => Key::Delete,
        262 => Key::Right,
        263 => Key::Left,
        264 => Key::Down,
        265 => Key::Up,
        266 => Key::PageUp,
        267 => Key::PageDown,
        268 => Key::Home,
        269 => Key::End,

        // Modifier keys
        340 => Key::LeftShift,
        341 => Key::LeftControl,
        342 => Key::LeftAlt,
        343 => Key::LeftSuper,
        344 => Key::RightShift,
        345 => Key::RightControl,
        346 => Key::RightAlt,
        347 => Key::RightSuper,

        // Punctuation and miscellaneous keys
        32 => Key::Space,
        39 => Key::Apostrophe,
        44 => Key::Comma,
        45 => Key::Minus,
        46 => Key::Period,
        47 => Key::Slash,
        59 => Key::Semicolon,
        61 => Key::Equal,
        91 => Key::LeftBracket,
        92 => Key::Backslash,
        93 => Key::RightBracket,
        96 => Key::GraveAccent,

        // Keypad keys
        320 => Key::Kp0,
        321 => Key::Kp1,
        322 => Key::Kp2,
        323 => Key::Kp3,
        324 => Key::Kp4,
        325 => Key::Kp5,
        326 => Key::Kp6,
        327 => Key::Kp7,
        328 => Key::Kp8,
        329 => Key::Kp9,
        330 => Key::KpDecimal,
        331 => Key::KpDivide,
        332 => Key::KpMultiply,
        333 => Key::KpSubtract,
        334 => Key::KpAdd,
        335 => Key::KpEnter,
        336 => Key::KpEqual,

        // Other keys
        280 => Key::CapsLock,
        281 => Key::ScrollLock,
        282 => Key::NumLock,
        283 => Key::PrintScreen,
        284 => Key::Pause,

        // Default for unmapped keys
        _ => Key::Unknown,
    }
}

/// Converts an integer mouse button code to a `glfw::MouseButton`.
fn from_glfw_mouse_button(button: c_int) -> glfw::MouseButton {
    match button {
        0 => glfw::MouseButton::Button1,
        1 => glfw::MouseButton::Button2,
        2 => glfw::MouseButton::Button3,
        3 => glfw::MouseButton::Button4,
        4 => glfw::MouseButton::Button5,
        5 => glfw::MouseButton::Button6,
        6 => glfw::MouseButton::Button7,
        7 => glfw::MouseButton::Button8,
        _ => glfw::MouseButton::Button1, // Default case
    }
}
