use game::Game;
use platform::graphics::window::WindowBuilder;
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn game_create(width: u32, height: u32, title: *const c_char) -> *mut Game {
    let title_str = unsafe { CStr::from_ptr(title).to_str().unwrap() };
    let window_builder = WindowBuilder {
        width,
        height,
        title: title_str.to_string(),
    };
    Box::into_raw(Box::new(Game::new(window_builder)))
}

#[no_mangle]
pub extern "C" fn game_init(game: *mut Game) {
    unsafe {
        (*game).init(|_| {});
    }
}

#[no_mangle]
pub extern "C" fn game_run(game: *mut Game) {
    unsafe {
        (*game).run(|_| {});
    }
}

#[no_mangle]
pub extern "C" fn game_destroy(game: *mut Game) {
    if !game.is_null() {
        unsafe { Box::from_raw(game) }; // Automatically deallocates
    }
}
