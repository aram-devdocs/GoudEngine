
mod game;

use game::{GameSdk, Renderer2D, Sprite, WindowBuilder};

use std::ffi::CStr;
use std::os::raw::c_char;


#[no_mangle]
pub extern "C" fn game_new(width: u32, height: u32, title: *const c_char) -> *mut GameSdk {
    println!("Creating game instance");
    let title_str = unsafe { CStr::from_ptr(title).to_str().unwrap() };
    let builder = WindowBuilder {
        width,
        height,
        title: title_str.to_string(),
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
            Box::from_raw(game); // Automatically drops the game instance
        }
    }
}
