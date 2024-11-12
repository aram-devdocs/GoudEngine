use crate::game::{GameSdk, WindowBuilder};
use crate::types::{EntityId, Rectangle, Sprite};
use crate::types::{SpriteCreateDto, SpriteUpdateDto, UpdateResponseData};
use glfw::Key;
use std::ffi::{c_uint, CStr, CString};
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
        let game = unsafe { &mut *game };
        game.terminate();
        println!("Terminating game instance");
        unsafe {
            drop(Box::from_raw(game));
        }
    }
}

#[no_mangle]
pub extern "C" fn game_add_sprite(game: *mut GameSdk, data: SpriteCreateDto) -> u32 {
    let game = unsafe { &mut *game };
    let texture_clone = game.texture_manager.get_texture(data.texture_id).clone();

    let sprite = Sprite::new(
        data.texture_id,
        data.x,
        data.y,
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
        // TODO:Placeholder needs to be fixed for collision detection
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        data.debug,
    );

    let id = game.ecs.add_sprite(sprite);
    id
}

#[no_mangle]
pub extern "C" fn game_create_texture(game: *mut GameSdk, texture_path: *const c_char) -> c_uint {
    let game = unsafe { &mut *game };
    let texture_path_str = unsafe { CStr::from_ptr(texture_path).to_str().unwrap() };
    let texture_path_cstring = CString::new(texture_path_str).unwrap();
    game.texture_manager
        .create_texture(texture_path_cstring.as_ptr())
}

#[no_mangle]
pub extern "C" fn game_update_sprite(game: *mut GameSdk, id: EntityId, data: SpriteUpdateDto) {
    let game = unsafe { &mut *game };
    let sprite_ref = game.ecs.get_sprite(id).expect("Sprite not found");
    let sprite = Sprite::new(
        if data.texture_id < 0 {
            // TODO: we need to handle optionals in c ffi
            sprite_ref.texture_id
        } else {
            data.texture_id
        },
        data.x,
        data.y,
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
        // TODO:Placeholder needs to be fixed for collision detection
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        data.debug,
    );

    game.ecs
        .update_sprite(id, sprite)
        .expect("Failed to update sprite");
}

#[no_mangle]
pub extern "C" fn game_remove_sprite(game: *mut GameSdk, id: EntityId) {
    let game = unsafe { &mut *game };
    println!("Removing sprite with id: {}", id);
    game.ecs.remove_sprite(id).expect("Failed to remove sprite");
}

#[no_mangle]
pub extern "C" fn game_is_key_pressed(game: *mut GameSdk, key_code: c_int) -> bool {
    let game = unsafe { &*game };
    let key = from_glfw_key_code(key_code);
    game.window.is_key_pressed(key)
}

#[no_mangle]
pub extern "C" fn check_collision_between_sprites(
    game: *mut GameSdk,
    entity_id1: EntityId,
    entity_id2: EntityId,
) -> bool {
    let game = unsafe { &*game };
    game.ecs
        .check_collision_between_sprites(entity_id1, entity_id2)
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
