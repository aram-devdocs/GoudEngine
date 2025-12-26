use crate::game::GameSdk;
use crate::libs::graphics::components::light::{Light, LightType};
use crate::libs::graphics::renderer::RendererKind;
use crate::libs::graphics::renderer3d::PrimitiveCreateInfo;
use crate::libs::platform::window::WindowBuilder;
use crate::types::{GridConfig, MousePosition, Rectangle};
use crate::types::{SpriteCreateDto, SpriteUpdateDto, UpdateResponseData};
use cgmath::Vector3;
use glfw::Key;
use std::ffi::{c_uint, CStr, CString};
use std::os::raw::{c_char, c_int};

// ============================================================================
// FFI Safety Helpers
// ============================================================================

/// Safely converts a C string pointer to a Rust &str.
/// Returns None if the pointer is null or contains invalid UTF-8.
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

/// Safely converts a C string pointer to a CString.
/// Returns None if the pointer is null or contains invalid UTF-8.
unsafe fn cstr_to_cstring(ptr: *const c_char) -> Option<CString> {
    cstr_to_str(ptr).and_then(|s| CString::new(s).ok())
}

/// Safely gets a mutable reference to GameSdk from a raw pointer.
/// Returns None if the pointer is null.
unsafe fn game_ref_mut(game: *mut GameSdk) -> Option<&'static mut GameSdk> {
    if game.is_null() {
        None
    } else {
        Some(&mut *game)
    }
}

/// Safely gets an immutable reference to GameSdk from a raw pointer.
/// Returns None if the pointer is null.
unsafe fn game_ref(game: *mut GameSdk) -> Option<&'static GameSdk> {
    if game.is_null() {
        None
    } else {
        Some(&*game)
    }
}

/// Initializes a new game instance with the specified window settings and returns a raw pointer to the `GameSdk`.
///
/// # Arguments
/// * `width` - The width of the game window.
/// * `height` - The height of the game window.
/// * `title` - A pointer to the C-style string for the game window title.
/// * `target_fps` - Target frames per second for the game.
/// * `renderer_type` - 0 for 2D, 1 for 3D
///
/// # Returns
/// * `*mut GameSdk` - A raw pointer to the newly created `GameSdk` instance, or null on failure.
///
/// # Safety
/// Returns null if title is null or contains invalid UTF-8.
#[no_mangle]
pub extern "C" fn game_create(
    width: u32,
    height: u32,
    title: *const c_char,
    target_fps: u32,
    renderer_type: c_int,
) -> *mut GameSdk {
    println!("Creating game instance");

    let title_cstring = match unsafe { cstr_to_cstring(title) } {
        Some(s) => s,
        None => {
            eprintln!("game_create: title is null or contains invalid UTF-8");
            return std::ptr::null_mut();
        }
    };

    let builder = WindowBuilder {
        width,
        height,
        title: title_cstring.as_ptr(),
        target_fps,
    };
    let game = GameSdk::new(builder, renderer_type);
    Box::into_raw(Box::new(game))
}

/// Initializes the game instance by setting up necessary resources.
///
/// # Returns
/// * `bool` - true if successful, false if game pointer is null.
#[no_mangle]
pub extern "C" fn game_initialize(game: *mut GameSdk) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_initialize: game pointer is null");
        return false;
    };
    game.init(|_| {});
    true
}

/// Starts the game loop for the provided game instance.
///
/// # Returns
/// * `bool` - true if successful, false if game pointer is null.
#[no_mangle]
pub extern "C" fn game_start(game: *mut GameSdk) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_start: game pointer is null");
        return false;
    };
    game.start(|_| {});
    true
}

/// Updates the game state and returns update response data, including delta time.
///
/// # Returns
/// * `UpdateResponseData` - Data related to the frame update, including delta time.
///   Returns zeroed struct if game pointer is null.
#[no_mangle]
pub extern "C" fn game_update(game: *mut GameSdk) -> UpdateResponseData {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_update: game pointer is null");
        return UpdateResponseData { delta_time: 0.0 };
    };
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
///
/// # Returns
/// * `u32` - The sprite ID, or 0 if game pointer is null.
#[no_mangle]
pub extern "C" fn game_add_sprite(game: *mut GameSdk, data: SpriteCreateDto) -> u32 {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_add_sprite: game pointer is null");
        return 0;
    };
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
        data.frame,
    );

    game.ecs.add_sprite(sprite)
}

/// Loads a texture into the game and returns its ID.
///
/// # Arguments
/// * `texture_path` - Path to the texture file as a C string.
///
/// # Returns
/// * `c_uint` - The texture ID, or 0 if game/path is null or invalid.
#[no_mangle]
pub extern "C" fn game_create_texture(game: *mut GameSdk, texture_path: *const c_char) -> c_uint {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_create_texture: game pointer is null");
        return 0;
    };
    let Some(texture_path_cstring) = (unsafe { cstr_to_cstring(texture_path) }) else {
        eprintln!("game_create_texture: texture_path is null or contains invalid UTF-8");
        return 0;
    };
    game.texture_manager
        .create_texture(texture_path_cstring.as_ptr())
}

/// Updates an existing sprite with new properties.
///
/// # Returns
/// * `bool` - true if successful, false if game is null or sprite not found.
#[no_mangle]
pub extern "C" fn game_update_sprite(game: *mut GameSdk, data: SpriteUpdateDto) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_update_sprite: game pointer is null");
        return false;
    };

    let Some(sprite_ref) = game.ecs.get_sprite(data.id) else {
        eprintln!("game_update_sprite: sprite {} not found", data.id);
        return false;
    };

    // TODO: We need to handle all of the == 0.0 cases as they can cause weird behavior.
    // If I switch to 0, I will always be switched back to the initial value.
    // This should be fixed with an update mask in Phase 3.
    let sprite = SpriteUpdateDto::new(
        data.id,
        data.x,
        data.y,
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
        #[allow(unused_comparisons)]
        data.texture_id,
        data.debug,
        data.frame,
    );

    match game.ecs.update_sprite(sprite) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("game_update_sprite: {}", e);
            false
        }
    }
}

/// Removes a sprite from the game instance.
///
/// # Returns
/// * `bool` - true if successful, false if game is null or sprite not found.
#[no_mangle]
pub extern "C" fn game_remove_sprite(game: *mut GameSdk, id: c_uint) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_remove_sprite: game pointer is null");
        return false;
    };
    match game.ecs.remove_sprite(id) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("game_remove_sprite: {}", e);
            false
        }
    }
}

/// Checks if the specified key is currently pressed.
///
/// # Returns
/// * `bool` - true if pressed, false if not pressed or game is null.
#[no_mangle]
pub extern "C" fn game_is_key_pressed(game: *mut GameSdk, key_code: c_int) -> bool {
    let Some(game) = (unsafe { game_ref(game) }) else {
        return false;
    };
    let key = from_glfw_key_code(key_code);
    game.window.is_key_pressed(key)
}

/// Checks if the specified mouse button is pressed.
///
/// # Returns
/// * `bool` - true if pressed, false if not pressed or game is null.
#[no_mangle]
pub extern "C" fn game_is_mouse_button_pressed(game: *mut GameSdk, button: c_int) -> bool {
    let Some(game) = (unsafe { game_ref(game) }) else {
        return false;
    };
    let button = from_glfw_mouse_button(button);
    game.window.is_mouse_button_pressed(button)
}

/// Retrieves the current mouse position in the game window.
///
/// # Returns
/// * `MousePosition` - The mouse position, or (0,0) if game is null.
#[no_mangle]
pub extern "C" fn game_get_mouse_position(game: *mut GameSdk) -> MousePosition {
    let Some(game) = (unsafe { game_ref(game) }) else {
        return MousePosition { x: 0.0, y: 0.0 };
    };
    let position = game.window.get_mouse_position();
    MousePosition {
        x: position.x,
        y: position.y,
    }
}

/// Handles gamepad button input events.
///
/// # Returns
/// * `bool` - true if successful, false if game is null.
#[no_mangle]
pub extern "C" fn game_handle_gamepad_button(
    game: *mut GameSdk,
    gamepad_id: u32,
    button: u32,
    pressed: bool,
) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };
    game.window
        .handle_gamepad_button(gamepad_id, button, pressed);
    true
}

/// Checks if a gamepad button is currently pressed.
///
/// # Returns
/// * `bool` - true if pressed, false if not pressed or game is null.
#[no_mangle]
pub extern "C" fn game_is_gamepad_button_pressed(
    game: *mut GameSdk,
    gamepad_id: u32,
    button: u32,
) -> bool {
    let Some(game) = (unsafe { game_ref(game) }) else {
        return false;
    };
    game.window.is_gamepad_button_pressed(gamepad_id, button)
}

/// Checks for collision between two sprites.
///
/// # Returns
/// * `bool` - true if colliding, false if not or game is null.
#[no_mangle]
pub extern "C" fn check_collision_between_sprites(
    game: *mut GameSdk,
    entity_id1: c_uint,
    entity_id2: c_uint,
) -> bool {
    let Some(game) = (unsafe { game_ref(game) }) else {
        return false;
    };
    game.ecs
        .check_collision_between_sprites(entity_id1, entity_id2)
}

/// Loads a tiled map into the game.
///
/// # Arguments
/// * `game` - Game instance pointer.
/// * `map_name` - Name identifier for the map.
/// * `map_path` - Path to the .tmx file.
/// * `texture_ids` - Pointer to array of texture IDs.
/// * `texture_count` - Number of textures in the array.
///
/// # Returns
/// * `c_uint` - The tiled map ID, or 0 on failure.
#[no_mangle]
pub extern "C" fn game_load_tiled_map(
    game: *mut GameSdk,
    map_name: *const c_char,
    map_path: *const c_char,
    texture_ids: *const c_uint,
    texture_count: usize,
) -> c_uint {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_load_tiled_map: game pointer is null");
        return 0;
    };

    let Some(map_path_str) = (unsafe { cstr_to_str(map_path) }) else {
        eprintln!("game_load_tiled_map: map_path is null or invalid UTF-8");
        return 0;
    };

    let Some(map_name_str) = (unsafe { cstr_to_str(map_name) }) else {
        eprintln!("game_load_tiled_map: map_name is null or invalid UTF-8");
        return 0;
    };

    if texture_ids.is_null() || texture_count == 0 {
        eprintln!("game_load_tiled_map: texture_ids is null or empty");
        return 0;
    }

    let texture_vec = unsafe { std::slice::from_raw_parts(texture_ids, texture_count).to_vec() };

    match game
        .tiled_manager
        .load_map(map_name_str, map_path_str, texture_vec)
    {
        Ok(id) => id,
        Err(e) => {
            eprintln!("game_load_tiled_map: {}", e);
            0
        }
    }
}

/// Sets the selected tiled map by ID.
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_set_selected_map_by_id(game: *mut GameSdk, map_id: c_uint) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_set_selected_map_by_id: game pointer is null");
        return false;
    };
    game.new_tileset = true;
    match game.tiled_manager.set_selected_map_by_id(map_id) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("game_set_selected_map_by_id: {}", e);
            false
        }
    }
}

/// Clears the selected tiled map.
///
/// # Returns
/// * `bool` - true if successful, false if game is null.
#[no_mangle]
pub extern "C" fn game_clear_selected_map(game: *mut GameSdk) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_clear_selected_map: game pointer is null");
        return false;
    };
    game.tiled_manager.clear_selected_map();
    true
}

/// Determines if the game window should close.
///
/// # Returns
/// * `bool` - true if should close, false otherwise or if game is null.
#[no_mangle]
pub extern "C" fn game_should_close(game: *mut GameSdk) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return true; // If game is null, we should close
    };
    game.window.should_close()
}

/// Logs a message to stdout.
///
/// # Returns
/// * `bool` - true if successful, false if message is null or invalid.
#[no_mangle]
pub extern "C" fn game_log(_game: *mut GameSdk, message: *const c_char) -> bool {
    let Some(message_str) = (unsafe { cstr_to_str(message) }) else {
        return false;
    };
    println!("{}", message_str);
    true
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

/// Sets the camera position.
///
/// # Arguments
/// * `x` - The x-coordinate of the camera position.
/// * `y` - The y-coordinate of the camera position.
///
/// # Returns
/// * `bool` - true if successful, false if game is null.
#[no_mangle]
pub extern "C" fn game_set_camera_position(game: *mut GameSdk, x: f32, y: f32) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };
    if let Some(renderer) = &mut game.renderer {
        renderer.set_camera_position(x, y);
        true
    } else {
        false
    }
}

/// Sets the camera zoom level.
///
/// # Arguments
/// * `zoom` - The zoom level of the camera.
///
/// # Returns
/// * `bool` - true if successful, false if game is null.
#[no_mangle]
pub extern "C" fn game_set_camera_zoom(game: *mut GameSdk, zoom: f32) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };
    if let Some(renderer) = &mut game.renderer {
        renderer.set_camera_zoom(zoom);
        true
    } else {
        false
    }
}

/// Sets the full camera position in 3D space.
///
/// # Arguments
/// * `x` - The x-coordinate of the camera position.
/// * `y` - The y-coordinate of the camera position.
/// * `z` - The z-coordinate of the camera position.
///
/// # Returns
/// * `bool` - true if successful, false if game is null.
#[no_mangle]
pub extern "C" fn game_set_camera_position_3d(game: *mut GameSdk, x: f32, y: f32, z: f32) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };
    if let Some(renderer) = &mut game.renderer {
        renderer.set_camera_position_3d(x, y, z);
        true
    } else {
        false
    }
}

/// Gets the camera position and stores it in the provided out parameter.
///
/// # Arguments
/// * `out_position` - Pointer to an array of 3 floats that will hold the position [x, y, z].
///
/// # Returns
/// * `bool` - true if successful, false if game or out_position is null.
#[no_mangle]
pub extern "C" fn game_get_camera_position(game: *mut GameSdk, out_position: *mut f32) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };

    if out_position.is_null() {
        return false;
    }

    if let Some(renderer) = &mut game.renderer {
        let pos = renderer.get_camera_position();
        unsafe {
            *out_position = pos.x;
            *out_position.add(1) = pos.y;
            *out_position.add(2) = pos.z;
        }
        true
    } else {
        unsafe {
            *out_position = 0.0;
            *out_position.add(1) = 0.0;
            *out_position.add(2) = 0.0;
        }
        true
    }
}

/// Sets the camera rotation using Euler angles in degrees.
///
/// # Arguments
/// * `pitch` - The pitch (x-axis rotation) in degrees.
/// * `yaw` - The yaw (y-axis rotation) in degrees.
/// * `roll` - The roll (z-axis rotation) in degrees.
///
/// # Returns
/// * `bool` - true if successful, false if game is null.
#[no_mangle]
pub extern "C" fn game_set_camera_rotation(
    game: *mut GameSdk,
    pitch: f32,
    yaw: f32,
    roll: f32,
) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };
    if let Some(renderer) = &mut game.renderer {
        renderer.set_camera_rotation(pitch, yaw, roll);
        true
    } else {
        false
    }
}

/// Gets the camera rotation as Euler angles in degrees and stores it in the provided out parameter.
///
/// # Arguments
/// * `out_rotation` - Pointer to an array of 3 floats that will hold the rotation [pitch, yaw, roll].
///
/// # Returns
/// * `bool` - true if successful, false if game or out_rotation is null.
#[no_mangle]
pub extern "C" fn game_get_camera_rotation(game: *mut GameSdk, out_rotation: *mut f32) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };

    if out_rotation.is_null() {
        return false;
    }

    if let Some(renderer) = &mut game.renderer {
        let rot = renderer.get_camera_rotation();
        unsafe {
            *out_rotation = rot.x;
            *out_rotation.add(1) = rot.y;
            *out_rotation.add(2) = rot.z;
        }
        true
    } else {
        unsafe {
            *out_rotation = 0.0;
            *out_rotation.add(1) = 0.0;
            *out_rotation.add(2) = 0.0;
        }
        true
    }
}

/// Gets the camera zoom level.
///
/// # Returns
/// * `f32` - The current camera zoom level, or 1.0 if game is null.
#[no_mangle]
pub extern "C" fn game_get_camera_zoom(game: *mut GameSdk) -> f32 {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return 1.0;
    };
    if let Some(renderer) = &mut game.renderer {
        renderer.get_camera_zoom()
    } else {
        1.0
    }
}

// #[no_mangle]
// pub extern "C" fn game_create_cube(game: *mut GameSdk, texture_id: c_uint) -> c_uint {
//     let game = unsafe { &mut *game };
//     if let Some(renderer) = &mut game.renderer {
//         if let RendererKind::Renderer3D = renderer.kind {
//             unsafe {
//                 if !renderer.renderer_3d.is_null() {
//                     let create_info = PrimitiveCreateInfo {
//                         primitive_type: PrimitiveType::Cube,
//                         width: 1.0,
//                         height: 1.0,
//                         depth: 1.0,
//                         segments: 1,
//                         texture_id,
//                     };
//                     match (*renderer.renderer_3d).create_primitive(create_info) {
//                         Ok(id) => id,
//                         Err(_) => 0,
//                     }
//                 } else {
//                     0
//                 }
//             }
//         } else {
//             0
//         }
//     } else {
//         0
//     }
// }

/// Creates a 3D primitive (cube, sphere, etc.).
///
/// # Returns
/// * `c_uint` - The object ID, or 0 on failure.
#[no_mangle]
pub extern "C" fn game_create_primitive(
    game: *mut GameSdk,
    create_info: PrimitiveCreateInfo,
) -> c_uint {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_create_primitive: game pointer is null");
        return 0;
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                renderer_3d.create_primitive(create_info).unwrap_or(0)
            } else {
                0
            }
        } else {
            eprintln!("game_create_primitive: 2D renderer cannot create 3D primitives");
            0
        }
    } else {
        0
    }
}

/// Sets the position of a 3D object.
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_set_object_position(
    game: *mut GameSdk,
    object_id: c_uint,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_set_object_position: game pointer is null");
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        match renderer.kind {
            RendererKind::Renderer2D => {
                eprintln!(
                    "game_set_object_position: cannot set 3D object position with 2D renderer"
                );
                false
            }
            RendererKind::Renderer3D => {
                if let Some(renderer_3d) = renderer.as_3d_mut() {
                    renderer_3d.set_object_position(object_id, x, y, z).is_ok()
                } else {
                    false
                }
            }
        }
    } else {
        false
    }
}

/// Sets the rotation of a 3D object.
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_set_object_rotation(
    game: *mut GameSdk,
    object_id: c_uint,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_set_object_rotation: game pointer is null");
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        match renderer.kind {
            RendererKind::Renderer2D => {
                eprintln!(
                    "game_set_object_rotation: cannot set 3D object rotation with 2D renderer"
                );
                false
            }
            RendererKind::Renderer3D => {
                if let Some(renderer_3d) = renderer.as_3d_mut() {
                    renderer_3d.set_object_rotation(object_id, x, y, z).is_ok()
                } else {
                    false
                }
            }
        }
    } else {
        false
    }
}

/// Sets the scale of a 3D object.
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_set_object_scale(
    game: *mut GameSdk,
    object_id: c_uint,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_set_object_scale: game pointer is null");
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        match renderer.kind {
            RendererKind::Renderer2D => {
                eprintln!("game_set_object_scale: cannot set 3D object scale with 2D renderer");
                false
            }
            RendererKind::Renderer3D => {
                if let Some(renderer_3d) = renderer.as_3d_mut() {
                    renderer_3d.set_object_scale(object_id, x, y, z).is_ok()
                } else {
                    false
                }
            }
        }
    } else {
        false
    }
}

/// Adds a light to the 3D scene.
///
/// # Returns
/// * `c_uint` - The light ID, or 0 on failure.
#[no_mangle]
pub extern "C" fn game_add_light(
    game: *mut GameSdk,
    light_type: c_int,
    position_x: f32,
    position_y: f32,
    position_z: f32,
    direction_x: f32,
    direction_y: f32,
    direction_z: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
    intensity: f32,
    temperature: f32,
    range: f32,
    spot_angle: f32,
) -> c_uint {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_add_light: game pointer is null");
        return 0;
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                let light_type = match light_type {
                    0 => LightType::Point,
                    1 => LightType::Directional,
                    2 => LightType::Spot,
                    _ => LightType::Point,
                };

                let light = Light::new(
                    0, // Will be set by LightManager
                    light_type,
                    Vector3::new(position_x, position_y, position_z),
                    Vector3::new(direction_x, direction_y, direction_z),
                    Vector3::new(color_r, color_g, color_b),
                    intensity,
                    temperature,
                    range,
                    spot_angle,
                );

                renderer_3d.add_light(light)
            } else {
                0
            }
        } else {
            eprintln!("game_add_light: 2D renderer does not support lighting");
            0
        }
    } else {
        0
    }
}

/// Removes a light from the 3D scene.
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_remove_light(game: *mut GameSdk, light_id: c_uint) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_remove_light: game pointer is null");
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                renderer_3d.remove_light(light_id);
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Updates a light's properties.
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_update_light(
    game: *mut GameSdk,
    light_id: c_uint,
    light_type: c_int,
    position_x: f32,
    position_y: f32,
    position_z: f32,
    direction_x: f32,
    direction_y: f32,
    direction_z: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
    intensity: f32,
    temperature: f32,
    range: f32,
    spot_angle: f32,
) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_update_light: game pointer is null");
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                let light_type = match light_type {
                    0 => LightType::Point,
                    1 => LightType::Directional,
                    2 => LightType::Spot,
                    _ => LightType::Point,
                };

                let new_light = Light::new(
                    light_id,
                    light_type,
                    Vector3::new(position_x, position_y, position_z),
                    Vector3::new(direction_x, direction_y, direction_z),
                    Vector3::new(color_r, color_g, color_b),
                    intensity,
                    temperature,
                    range,
                    spot_angle,
                );

                renderer_3d.update_light(light_id, new_light).is_ok()
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Configures the 3D grid with full options.
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_configure_grid(
    game: *mut GameSdk,
    enabled: bool,
    size: f32,
    divisions: u32,
    xz_color_r: f32,
    xz_color_g: f32,
    xz_color_b: f32,
    xy_color_r: f32,
    xy_color_g: f32,
    xy_color_b: f32,
    yz_color_r: f32,
    yz_color_g: f32,
    yz_color_b: f32,
    x_axis_color_r: f32,
    x_axis_color_g: f32,
    x_axis_color_b: f32,
    y_axis_color_r: f32,
    y_axis_color_g: f32,
    y_axis_color_b: f32,
    z_axis_color_r: f32,
    z_axis_color_g: f32,
    z_axis_color_b: f32,
    line_width: f32,
    axis_line_width: f32,
    show_axes: bool,
    show_xz_plane: bool,
    show_xy_plane: bool,
    show_yz_plane: bool,
    render_mode: c_int,
) -> bool {
    use crate::types::GridRenderMode;

    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_configure_grid: game pointer is null");
        return false;
    };

    let grid_config = GridConfig {
        enabled,
        size,
        divisions,
        xz_color: Vector3::new(xz_color_r, xz_color_g, xz_color_b),
        xy_color: Vector3::new(xy_color_r, xy_color_g, xy_color_b),
        yz_color: Vector3::new(yz_color_r, yz_color_g, yz_color_b),
        x_axis_color: Vector3::new(x_axis_color_r, x_axis_color_g, x_axis_color_b),
        y_axis_color: Vector3::new(y_axis_color_r, y_axis_color_g, y_axis_color_b),
        z_axis_color: Vector3::new(z_axis_color_r, z_axis_color_g, z_axis_color_b),
        line_width,
        axis_line_width,
        show_axes,
        show_xz_plane,
        show_xy_plane,
        show_yz_plane,
        render_mode: match render_mode {
            0 => GridRenderMode::Blend,
            _ => GridRenderMode::Overlap,
        },
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                renderer_3d.configure_grid(grid_config);
                return true;
            }
        }
    }

    false
}

/// Toggles the grid on or off.
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_set_grid_enabled(game: *mut GameSdk, enabled: bool) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                let mut config = renderer_3d.get_grid_config();
                config.enabled = enabled;
                renderer_3d.configure_grid(config);
                return true;
            }
        }
    }

    false
}

/// Sets which grid planes are visible.
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_set_grid_planes(
    game: *mut GameSdk,
    show_xz: bool,
    show_xy: bool,
    show_yz: bool,
) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                let mut config = renderer_3d.get_grid_config();
                config.show_xz_plane = show_xz;
                config.show_xy_plane = show_xy;
                config.show_yz_plane = show_yz;
                renderer_3d.configure_grid(config);
                return true;
            }
        }
    }

    false
}

/// Sets the grid render mode (blend vs overlap).
///
/// # Returns
/// * `bool` - true if successful, false on failure.
#[no_mangle]
pub extern "C" fn game_set_grid_render_mode(game: *mut GameSdk, blend_mode: bool) -> bool {
    use crate::types::GridRenderMode;

    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                let mut config = renderer_3d.get_grid_config();
                config.render_mode = if blend_mode {
                    GridRenderMode::Blend
                } else {
                    GridRenderMode::Overlap
                };
                renderer_3d.configure_grid(config);
                return true;
            }
        }
    }

    false
}

// Skybox configuration functions

#[no_mangle]
pub extern "C" fn game_configure_skybox(
    game: *mut GameSdk,
    enabled: bool,
    size: f32,
    texture_size: u32,
    right_face_r: f32,
    right_face_g: f32,
    right_face_b: f32,
    left_face_r: f32,
    left_face_g: f32,
    left_face_b: f32,
    top_face_r: f32,
    top_face_g: f32,
    top_face_b: f32,
    bottom_face_r: f32,
    bottom_face_g: f32,
    bottom_face_b: f32,
    front_face_r: f32,
    front_face_g: f32,
    front_face_b: f32,
    back_face_r: f32,
    back_face_g: f32,
    back_face_b: f32,
    blend_factor: f32,
    min_color_r: f32,
    min_color_g: f32,
    min_color_b: f32,
    use_custom_textures: bool,
) -> bool {
    use crate::types::SkyboxConfig;
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_configure_skybox: game pointer is null");
        return false;
    };

    // Create a skybox configuration
    let skybox_config = SkyboxConfig {
        enabled,
        size,
        texture_size,
        face_colors: [
            Vector3::new(right_face_r, right_face_g, right_face_b), // Right face
            Vector3::new(left_face_r, left_face_g, left_face_b),    // Left face
            Vector3::new(top_face_r, top_face_g, top_face_b),       // Top face
            Vector3::new(bottom_face_r, bottom_face_g, bottom_face_b), // Bottom face
            Vector3::new(front_face_r, front_face_g, front_face_b), // Front face
            Vector3::new(back_face_r, back_face_g, back_face_b),    // Back face
        ],
        blend_factor,
        min_color: Vector3::new(min_color_r, min_color_g, min_color_b),
        use_custom_textures,
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                if let Some(skybox) = &mut renderer_3d.skybox {
                    return skybox.configure(skybox_config).is_ok();
                }
            }
        }
    }

    false
}

#[no_mangle]
pub extern "C" fn game_set_skybox_enabled(game: *mut GameSdk, enabled: bool) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_set_skybox_enabled: game pointer is null");
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                if let Some(skybox) = &mut renderer_3d.skybox {
                    // Get current config and only update the enabled flag
                    let mut config = skybox.get_config();
                    config.enabled = enabled;
                    return skybox.configure(config).is_ok();
                }
            }
        }
    }

    false
}

#[no_mangle]
pub extern "C" fn game_set_skybox_colors(
    game: *mut GameSdk,
    right_face_r: f32,
    right_face_g: f32,
    right_face_b: f32,
    left_face_r: f32,
    left_face_g: f32,
    left_face_b: f32,
    top_face_r: f32,
    top_face_g: f32,
    top_face_b: f32,
    bottom_face_r: f32,
    bottom_face_g: f32,
    bottom_face_b: f32,
    front_face_r: f32,
    front_face_g: f32,
    front_face_b: f32,
    back_face_r: f32,
    back_face_g: f32,
    back_face_b: f32,
) -> bool {
    let Some(game) = (unsafe { game_ref_mut(game) }) else {
        eprintln!("game_set_skybox_colors: game pointer is null");
        return false;
    };

    if let Some(renderer) = &mut game.renderer {
        if let RendererKind::Renderer3D = renderer.kind {
            if let Some(renderer_3d) = renderer.as_3d_mut() {
                if let Some(skybox) = &mut renderer_3d.skybox {
                    // Get current config and update face colors
                    let mut config = skybox.get_config();
                    config.face_colors = [
                        Vector3::new(right_face_r, right_face_g, right_face_b), // Right face
                        Vector3::new(left_face_r, left_face_g, left_face_b),    // Left face
                        Vector3::new(top_face_r, top_face_g, top_face_b),       // Top face
                        Vector3::new(bottom_face_r, bottom_face_g, bottom_face_b), // Bottom face
                        Vector3::new(front_face_r, front_face_g, front_face_b), // Front face
                        Vector3::new(back_face_r, back_face_g, back_face_b),    // Back face
                    ];
                    return skybox.configure(config).is_ok();
                }
            }
        }
    }

    false
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    // Helper to create a test SpriteCreateDto
    fn test_sprite_create() -> SpriteCreateDto {
        SpriteCreateDto {
            x: 0.0,
            y: 0.0,
            z_layer: 0,
            scale_x: 1.0,
            scale_y: 1.0,
            dimension_x: 32.0,
            dimension_y: 32.0,
            rotation: 0.0,
            source_rect: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            texture_id: 1,
            debug: false,
            frame: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
        }
    }

    // Helper to create a test SpriteUpdateDto
    fn test_sprite_update() -> SpriteUpdateDto {
        SpriteUpdateDto {
            id: 1,
            x: 0.0,
            y: 0.0,
            z_layer: 0,
            scale_x: 1.0,
            scale_y: 1.0,
            dimension_x: 32.0,
            dimension_y: 32.0,
            rotation: 0.0,
            source_rect: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            texture_id: 1,
            debug: false,
            frame: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
        }
    }

    // ========================================================================
    // FFI Safety Helper Tests
    // ========================================================================

    #[test]
    fn test_cstr_to_str_valid() {
        let s = CString::new("hello").unwrap();
        let result = unsafe { cstr_to_str(s.as_ptr()) };
        assert_eq!(result, Some("hello"));
    }

    #[test]
    fn test_cstr_to_str_null() {
        let result = unsafe { cstr_to_str(ptr::null()) };
        assert_eq!(result, None);
    }

    #[test]
    fn test_cstr_to_str_empty() {
        let s = CString::new("").unwrap();
        let result = unsafe { cstr_to_str(s.as_ptr()) };
        assert_eq!(result, Some(""));
    }

    #[test]
    fn test_cstr_to_str_unicode() {
        let s = CString::new("Hello 世界").unwrap();
        let result = unsafe { cstr_to_str(s.as_ptr()) };
        assert_eq!(result, Some("Hello 世界"));
    }

    #[test]
    fn test_cstr_to_cstring_valid() {
        let s = CString::new("test").unwrap();
        let result = unsafe { cstr_to_cstring(s.as_ptr()) };
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_str().unwrap(), "test");
    }

    #[test]
    fn test_cstr_to_cstring_null() {
        let result = unsafe { cstr_to_cstring(ptr::null()) };
        assert!(result.is_none());
    }

    #[test]
    fn test_game_ref_mut_null() {
        let result = unsafe { game_ref_mut(ptr::null_mut()) };
        assert!(result.is_none());
    }

    #[test]
    fn test_game_ref_null() {
        let result = unsafe { game_ref(ptr::null_mut()) };
        assert!(result.is_none());
    }

    // ========================================================================
    // FFI Function Null Pointer Handling Tests
    // ========================================================================

    #[test]
    fn test_game_create_null_title() {
        let result = game_create(800, 600, ptr::null(), 60, 0);
        assert!(result.is_null());
    }

    #[test]
    fn test_game_initialize_null_game() {
        let result = game_initialize(ptr::null_mut());
        assert!(!result);
    }

    #[test]
    fn test_game_start_null_game() {
        let result = game_start(ptr::null_mut());
        assert!(!result);
    }

    #[test]
    fn test_game_update_null_game() {
        let result = game_update(ptr::null_mut());
        assert_eq!(result.delta_time, 0.0);
    }

    #[test]
    fn test_game_terminate_null_game() {
        // Should not panic
        game_terminate(ptr::null_mut());
    }

    #[test]
    fn test_game_should_close_null_game() {
        let result = game_should_close(ptr::null_mut());
        assert!(result); // Should return true (closed) when null
    }

    #[test]
    fn test_game_add_sprite_null_game() {
        let sprite = test_sprite_create();
        let result = game_add_sprite(ptr::null_mut(), sprite);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_game_update_sprite_null_game() {
        let update = test_sprite_update();
        let result = game_update_sprite(ptr::null_mut(), update);
        assert!(!result);
    }

    #[test]
    fn test_game_remove_sprite_null_game() {
        let result = game_remove_sprite(ptr::null_mut(), 1);
        assert!(!result);
    }

    #[test]
    fn test_game_is_key_pressed_null_game() {
        let result = game_is_key_pressed(ptr::null_mut(), Key::A as i32);
        assert!(!result);
    }

    #[test]
    fn test_game_get_mouse_position_null_game() {
        let result = game_get_mouse_position(ptr::null_mut());
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 0.0);
    }

    #[test]
    fn test_game_is_mouse_button_pressed_null_game() {
        let result = game_is_mouse_button_pressed(ptr::null_mut(), 0);
        assert!(!result);
    }

    #[test]
    fn test_game_create_texture_null_game() {
        let path = CString::new("test.png").unwrap();
        let result = game_create_texture(ptr::null_mut(), path.as_ptr());
        assert_eq!(result, 0);
    }

    #[test]
    fn test_game_create_texture_null_path() {
        let result = game_create_texture(ptr::null_mut(), ptr::null());
        assert_eq!(result, 0);
    }

    #[test]
    fn test_game_log_null_message() {
        let result = game_log(ptr::null_mut(), ptr::null());
        assert!(!result);
    }

    #[test]
    fn test_game_log_valid_message() {
        let msg = CString::new("test message").unwrap();
        let result = game_log(ptr::null_mut(), msg.as_ptr());
        assert!(result); // game_log doesn't require game pointer
    }

    // ========================================================================
    // 3D Functions Null Pointer Handling Tests
    // ========================================================================

    #[test]
    fn test_game_create_primitive_null_game() {
        let create_info = PrimitiveCreateInfo {
            primitive_type: crate::libs::graphics::renderer3d::PrimitiveType::Cube,
            width: 1.0,
            height: 1.0,
            depth: 1.0,
            segments: 1,
            texture_id: 0,
        };
        let result = game_create_primitive(ptr::null_mut(), create_info);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_game_add_light_null_game() {
        let result = game_add_light(
            ptr::null_mut(),
            0, // point light
            0.0,
            0.0,
            0.0, // position
            0.0,
            -1.0,
            0.0, // direction
            1.0,
            1.0,
            1.0,    // color
            1.0,    // intensity
            5500.0, // temperature
            10.0,   // range
            45.0,   // spot angle
        );
        assert_eq!(result, 0);
    }

    #[test]
    fn test_game_remove_light_null_game() {
        let result = game_remove_light(ptr::null_mut(), 1);
        assert!(!result);
    }

    #[test]
    fn test_game_update_light_null_game() {
        let result = game_update_light(
            ptr::null_mut(),
            1, // light_id
            0, // light_type
            0.0,
            0.0,
            0.0, // position
            0.0,
            -1.0,
            0.0, // direction
            1.0,
            1.0,
            1.0,    // color
            1.0,    // intensity
            5500.0, // temperature
            10.0,   // range
            45.0,   // spot angle
        );
        assert!(!result);
    }

    #[test]
    fn test_game_configure_grid_null_game() {
        // game_configure_grid takes many individual parameters (29 total)
        let result = game_configure_grid(
            ptr::null_mut(),
            true, // enabled
            20.0, // size
            20,   // divisions
            0.7,
            0.7,
            0.7, // xz_color
            0.8,
            0.6,
            0.6, // xy_color
            0.6,
            0.6,
            0.8, // yz_color
            0.9,
            0.2,
            0.2, // x_axis_color
            0.2,
            0.9,
            0.2, // y_axis_color
            0.2,
            0.2,
            0.9,  // z_axis_color
            1.5,  // line_width
            2.5,  // axis_line_width
            true, // show_axes
            true, // show_xz_plane
            true, // show_xy_plane
            true, // show_yz_plane
            0,    // render_mode (GridRenderMode::Blend = 0)
        );
        assert!(!result);
    }

    #[test]
    fn test_game_set_grid_enabled_null_game() {
        let result = game_set_grid_enabled(ptr::null_mut(), true);
        assert!(!result);
    }

    #[test]
    fn test_game_set_skybox_enabled_null_game() {
        let result = game_set_skybox_enabled(ptr::null_mut(), true);
        assert!(!result);
    }

    // ========================================================================
    // Camera Functions Null Pointer Handling Tests
    // ========================================================================

    #[test]
    fn test_game_set_camera_position_null_game() {
        let result = game_set_camera_position(ptr::null_mut(), 0.0, 0.0);
        assert!(!result);
    }

    #[test]
    fn test_game_set_camera_position_3d_null_game() {
        let result = game_set_camera_position_3d(ptr::null_mut(), 0.0, 0.0, 0.0);
        assert!(!result);
    }

    #[test]
    fn test_game_set_camera_rotation_null_game() {
        let result = game_set_camera_rotation(ptr::null_mut(), 0.0, 0.0, 0.0);
        assert!(!result);
    }

    #[test]
    fn test_game_set_camera_zoom_null_game() {
        let result = game_set_camera_zoom(ptr::null_mut(), 1.0);
        assert!(!result);
    }

    #[test]
    fn test_game_get_camera_position_null_game() {
        let mut pos: [f32; 3] = [0.0; 3];
        let result = game_get_camera_position(ptr::null_mut(), pos.as_mut_ptr());
        assert!(!result);
    }

    #[test]
    fn test_game_get_camera_rotation_null_game() {
        let mut rot: [f32; 3] = [0.0; 3];
        let result = game_get_camera_rotation(ptr::null_mut(), rot.as_mut_ptr());
        assert!(!result);
    }

    #[test]
    fn test_game_get_camera_zoom_null_game() {
        let result = game_get_camera_zoom(ptr::null_mut());
        // Returns default zoom (1.0) when game is null
        assert_eq!(result, 1.0);
    }

    // ========================================================================
    // Tiled Map Functions Null Pointer Handling Tests
    // ========================================================================

    #[test]
    fn test_game_load_tiled_map_null_game() {
        let name = CString::new("test_map").unwrap();
        let path = CString::new("map.json").unwrap();
        let textures: [c_uint; 1] = [1];
        let result = game_load_tiled_map(
            ptr::null_mut(),
            name.as_ptr(),
            path.as_ptr(),
            textures.as_ptr(),
            1,
        );
        assert_eq!(result, 0);
    }

    #[test]
    fn test_game_load_tiled_map_null_name() {
        let path = CString::new("map.json").unwrap();
        let textures: [c_uint; 1] = [1];
        let result = game_load_tiled_map(
            ptr::null_mut(),
            ptr::null(),
            path.as_ptr(),
            textures.as_ptr(),
            1,
        );
        assert_eq!(result, 0);
    }

    #[test]
    fn test_game_load_tiled_map_null_path() {
        let name = CString::new("test_map").unwrap();
        let textures: [c_uint; 1] = [1];
        let result = game_load_tiled_map(
            ptr::null_mut(),
            name.as_ptr(),
            ptr::null(),
            textures.as_ptr(),
            1,
        );
        assert_eq!(result, 0);
    }

    #[test]
    fn test_game_set_selected_map_null_game() {
        let result = game_set_selected_map_by_id(ptr::null_mut(), 1);
        assert!(!result);
    }

    #[test]
    fn test_game_clear_selected_map_null_game() {
        let result = game_clear_selected_map(ptr::null_mut());
        assert!(!result);
    }

    // ========================================================================
    // Object Transform Functions Null Pointer Tests
    // ========================================================================

    #[test]
    fn test_game_set_object_position_null_game() {
        let result = game_set_object_position(ptr::null_mut(), 1, 0.0, 0.0, 0.0);
        assert!(!result);
    }

    #[test]
    fn test_game_set_object_rotation_null_game() {
        let result = game_set_object_rotation(ptr::null_mut(), 1, 0.0, 0.0, 0.0);
        assert!(!result);
    }

    #[test]
    fn test_game_set_object_scale_null_game() {
        let result = game_set_object_scale(ptr::null_mut(), 1, 1.0, 1.0, 1.0);
        assert!(!result);
    }

    // ========================================================================
    // Gamepad Functions Null Pointer Tests
    // ========================================================================

    #[test]
    fn test_game_is_gamepad_button_pressed_null_game() {
        let result = game_is_gamepad_button_pressed(ptr::null_mut(), 0, 0);
        assert!(!result);
    }
}
