mod game;

use game::{GameSdk, WindowBuilder, Texture, Rectangle, Sprite};
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::rc::Rc;
use game::cgmath::Vector2;

#[no_mangle]
pub extern "C" fn game_new(width: u32, height: u32, title: *const c_char) -> *mut GameSdk {
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
pub extern "C" fn game_init(game: *mut GameSdk) {
    let game = unsafe { &mut *game };
    game.init(|_| {});
}

#[no_mangle]
pub extern "C" fn game_run(game: *mut GameSdk) {
    let game = unsafe { &mut *game };
    game.run(|_| {});
}

#[no_mangle]
pub extern "C" fn game_destroy(game: *mut GameSdk) {
    if !game.is_null() {
        unsafe {
            drop(Box::from_raw(game)); // Automatically drops the game instance
        }
    }
}

#[no_mangle]
pub extern "C" fn game_add_sprite(game: *mut GameSdk, texture_path: *const c_char, x: f32, y: f32, scale_x: f32, scale_y: f32, rotation: f32) {
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
pub extern "C" fn game_update_sprite(game: *mut GameSdk, index: usize, x: f32, y: f32, scale_x: f32, scale_y: f32, rotation: f32) {
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

// #[no_mangle]
// pub extern "C" fn game_is_key_pressed(game: *mut GameSdk, key: i32) -> bool {
//     let game = unsafe { &mut *game };
//     game.window.input_handler.is_key_pressed(key.into())
// }

#[no_mangle]
pub extern "C" fn game_close_window(game: *mut GameSdk) {
    let game = unsafe { &mut *game };
    game.window.close_window();
}

// Add sprite


// TODO: This is inefficient.
// Opaque pointer to the game instance
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
