use crate::game::cgmath::Vector2;
use crate::game::{GameSdk, WindowBuilder};
// use crate::libs::platform::graphics::rendering::{Rectangle, Sprite, Texture};
use crate::types::{SpriteDto, UpdateResponseData};
use crate::types::{Rectangle, Sprite, Texture};
use glfw::Key;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

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
pub extern "C" fn game_update(game: *mut GameSdk) -> UpdateResponseData {
    let game = unsafe { &mut *game };
    game.update(&|_| {});
    UpdateResponseData {
        delta_time: game.window.delta_time,
    }
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
    data: SpriteDto,
) -> usize {
    let game = unsafe { &mut *game };
    let texture_path_str = unsafe { CStr::from_ptr(texture_path).to_str().unwrap() };
    let texture = Texture::new(texture_path_str).expect("Failed to load texture");

    let source_rect = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };


    let texture_clone = texture.clone();
    let sprite = Sprite::new(
        texture,
        data.x,
        data.y,
        data.scale_x.unwrap_or(1.0),
        data.scale_y.unwrap_or(1.0),
        data.dimension_x.unwrap_or(texture_clone.width() as f32),
        data.dimension_y.unwrap_or(texture_clone.height() as f32),
        data.rotation,
        Some(source_rect),
    );

    game.renderer_2d.as_mut().unwrap().add_sprite(sprite);

    game.renderer_2d.as_ref().unwrap().sprites.len() - 1
}

#[no_mangle]
pub extern "C" fn game_update_sprite(game: *mut GameSdk, index: usize, data: SpriteDto) {
    let game = unsafe { &mut *game };
    let renderer = game.renderer_2d.as_ref().unwrap();
    let sprite_ref = &renderer.sprites[index];
    let texture = sprite_ref.texture.clone();

    let texture_clone = texture.clone();
    let sprite = Sprite::new(
        texture,
        data.x,
        data.y,
        data.scale_x.unwrap_or(1.0),
        data.scale_y.unwrap_or(1.0),
        data.dimension_x.unwrap_or(texture_clone.width() as f32),
        data.dimension_y.unwrap_or(texture_clone.height() as f32),
        data.rotation,
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
        87 => Key::W,      // W
        65 => Key::A,      // A
        83 => Key::S,      // S
        68 => Key::D,      // D
        69 => Key::E,      // E
        81 => Key::Q,      // Q
        32 => Key::Space,  // Space
        27 => Key::Escape, // Escape
        90 => Key::Z,      // Z
        88 => Key::X,      // X
        82 => Key::R,
        // TODO: https://github.com/aram-devdocs/GoudEngine/issues/9
        _ => Key::Unknown,
    }
}

#[no_mangle]
pub extern "C" fn game_should_close(game: *mut GameSdk) -> bool {
    let game = unsafe { &mut *game };
    game.window.should_close()
}
