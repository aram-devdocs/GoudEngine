mod game;

use game::cgmath::Vector2;
use game::{GameSdk, Rectangle, Sprite, Texture, WindowBuilder};
use glfw::Key;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[no_mangle]
pub extern "C" fn game_create(width: u32, height: u32, title: *const c_char) -> *mut GameSdk {
    println!("Creating game instance");
    let title_str = unsafe { CStr::from_ptr(title).to_str().unwrap() };
    let title_cstring = CString::new(title_str).unwrap();
    let builder = WindowBuilder {
        width,
        height,
        title: title_cstring.as_ptr(),
    };
    let game = GameSdk::new(builder);
    Box::into_raw(Box::new(game))
}

#[no_mangle]
pub extern "C" fn game_initialize(game: *mut GameSdk) {
    let game = unsafe { &mut *game };
    game.init(|_| {});
}

#[no_mangle]
pub extern "C" fn game_start(game: *mut GameSdk) {
    let game = unsafe { &mut *game };
    game.start(|_| {});
}

#[no_mangle]
pub extern "C" fn game_update(game: *mut GameSdk) {
    let game = unsafe { &mut *game };
    game.update(&|_| {});
}

#[no_mangle]
pub extern "C" fn game_terminate(game: *mut GameSdk) {
    if !game.is_null() {
        println!("Terminating game instance");
        unsafe {
            drop(Box::from_raw(game));
        }
    }
}

#[no_mangle]
pub extern "C" fn game_add_sprite(
    game: *mut GameSdk,
    texture_path: *const c_char,
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
) {
    let game = unsafe { &mut *game };
    let texture_path_str = unsafe { CStr::from_ptr(texture_path).to_str().unwrap() };
    let texture = Texture::new(texture_path_str).expect("Failed to load texture");

    let source_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };

    let sprite = Sprite::new(
        texture,
        Vector2::new(x, y),
        Vector2::new(scale_x, scale_y),
        rotation,
        Some(source_rect),
    );

    game.renderer_2d.as_mut().unwrap().add_sprite(sprite);
}

#[no_mangle]
pub extern "C" fn game_update_sprite(
    game: *mut GameSdk,
    index: usize,
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
) {
    let game = unsafe { &mut *game };
    let sprite = Sprite::new(
        game.renderer_2d.as_ref().unwrap().sprites[index]
            .texture
            .clone(),
        Vector2::new(x, y),
        Vector2::new(scale_x, scale_y),
        rotation,
        Some(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }),
    );

    game.renderer_2d
        .as_mut()
        .unwrap()
        .update_sprite(index, sprite)
        .expect("Failed to update sprite");
}

#[no_mangle]
pub extern "C" fn game_is_key_pressed(game: *mut GameSdk, key_code: c_int) -> bool {
    let game = unsafe { &*game };
    let key = from_glfw_key_code(key_code);
    game.window.is_key_pressed(key)
}

fn from_glfw_key_code(key_code: c_int) -> Key {
    match key_code {
        87 => Key::W, // W
        65 => Key::A, // A
        83 => Key::S, // S
        68 => Key::D, // D
        _ => Key::Unknown,
    }
}

#[no_mangle]
pub extern "C" fn game_should_close(game: *mut GameSdk) -> bool {
    let game = unsafe { &mut *game };
    game.window.should_close()
}

// Opaque pointers for additional structures
#[repr(C)]
pub struct Glfw {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Receiver {
    _private: [u8; 0],
}

#[repr(C)]
pub struct HashSet {
    _private: [u8; 0],
}
