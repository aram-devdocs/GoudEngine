mod game;

use game::{GameSdk, WindowBuilder};

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

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
