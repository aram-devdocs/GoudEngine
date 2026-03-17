#![cfg(feature = "native")]

use std::ffi::CString;
use std::thread;
use std::time::{Duration, Instant};

use goud_engine::ffi::context::GOUD_INVALID_CONTEXT_ID;
use goud_engine::ffi::engine_config::{
    goud_engine_config_create, goud_engine_config_set_size, goud_engine_config_set_title,
    goud_engine_create,
};
use goud_engine::ffi::renderer::{goud_renderer_begin, goud_renderer_end};
use goud_engine::ffi::renderer3d::{
    goud_renderer3d_create_cube, goud_renderer3d_render, goud_renderer3d_set_object_position,
};
use goud_engine::ffi::window::{
    goud_window_clear, goud_window_destroy, goud_window_get_size, goud_window_poll_events,
    goud_window_set_size, read_default_framebuffer_rgba8_for_context, with_window_state,
};

fn should_skip_native_smoke() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none()
    }

    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

fn pump_until_window_size_ffi(
    context_id: goud_engine::ffi::context::GoudContextId,
    expected_width: u32,
    expected_height: u32,
) -> (u32, u32) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let _ = goud_window_poll_events(context_id);
        let mut width = 0;
        let mut height = 0;
        // SAFETY: valid output pointers for the duration of this call.
        let ok = unsafe { goud_window_get_size(context_id, &mut width, &mut height) };
        assert!(ok, "FFI window size query should succeed");
        if (width, height) == (expected_width, expected_height) {
            return with_window_state(context_id, |state| state.get_framebuffer_size())
                .expect("window state should exist");
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for FFI window resize to {expected_width}x{expected_height}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn ffi_default_native_smoke() {
    let title = CString::new("native-main-thread-ffi").expect("title should be valid");
    let handle = goud_engine_config_create();

    // SAFETY: `handle` was returned by `goud_engine_config_create` and `title`
    // remains alive for the duration of these calls.
    unsafe {
        assert!(goud_engine_config_set_title(handle, title.as_ptr()));
        assert!(goud_engine_config_set_size(handle, 160, 120));
    }

    // SAFETY: `handle` is valid and consumed exactly once here.
    let context_id = unsafe { goud_engine_create(handle) };
    assert_ne!(
        context_id, GOUD_INVALID_CONTEXT_ID,
        "FFI engine creation should succeed on the default stack"
    );

    let mut width = 0;
    let mut height = 0;
    // SAFETY: valid output pointers for the duration of the call.
    let got_size = unsafe { goud_window_get_size(context_id, &mut width, &mut height) };
    assert!(got_size, "FFI window size query should succeed");
    assert_eq!((width, height), (160, 120));

    let _ = goud_window_poll_events(context_id);
    let (fb_width, fb_height) = with_window_state(context_id, |state| state.get_framebuffer_size())
        .expect("window state should exist");
    assert!(fb_width > 0);
    assert!(fb_height > 0);

    assert!(goud_renderer_begin(context_id));
    goud_window_clear(context_id, 0.05, 0.10, 0.15, 1.0);
    let cube = goud_renderer3d_create_cube(context_id, 0, 1.0, 1.0, 1.0);
    assert_ne!(cube, u32::MAX, "FFI 3D cube creation should succeed");
    assert!(goud_renderer3d_set_object_position(
        context_id, cube, 0.0, 0.0, 0.0
    ));
    assert!(goud_renderer3d_render(context_id));
    assert!(goud_renderer_end(context_id));

    let readback = read_default_framebuffer_rgba8_for_context(context_id, fb_width, fb_height)
        .expect("FFI framebuffer readback should succeed");
    assert_eq!(readback.len(), (fb_width * fb_height * 4) as usize);

    assert!(goud_window_set_size(context_id, 196, 148));
    let (resized_fb_width, resized_fb_height) = pump_until_window_size_ffi(context_id, 196, 148);
    assert!(resized_fb_width > 0);
    assert!(resized_fb_height > 0);

    assert!(goud_renderer_begin(context_id));
    goud_window_clear(context_id, 0.20, 0.10, 0.05, 1.0);
    assert!(goud_renderer_end(context_id));
    let resized_readback =
        read_default_framebuffer_rgba8_for_context(context_id, resized_fb_width, resized_fb_height)
            .expect("FFI resized framebuffer readback should succeed");
    assert_eq!(
        resized_readback.len(),
        (resized_fb_width * resized_fb_height * 4) as usize
    );

    assert!(
        goud_window_destroy(context_id),
        "window destroy should succeed"
    );
}

fn main() {
    if should_skip_native_smoke() {
        eprintln!("skipping native main-thread ffi smoke: no display available");
        return;
    }

    ffi_default_native_smoke();
}
