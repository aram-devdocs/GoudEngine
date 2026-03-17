#![cfg(feature = "native")]

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use goud_engine::sdk::{GameConfig, GoudGame};

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

fn pump_until_window_size_sdk(
    game: &mut GoudGame,
    expected_width: u32,
    expected_height: u32,
) -> (u32, u32) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        game.poll_events().expect("SDK poll_events should succeed");
        if game.get_window_size() == (expected_width, expected_height) {
            return game.get_framebuffer_size();
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for SDK window resize to {expected_width}x{expected_height}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn assert_readback_or_unsupported(
    result: goud_engine::core::error::GoudResult<Vec<u8>>,
    expected_len: usize,
) {
    match result {
        Ok(readback) => assert_eq!(readback.len(), expected_len),
        Err(error) => {
            let message = error.to_string();
            assert!(
                message.contains("readback is not supported"),
                "unexpected readback error: {message}"
            );
        }
    }
}

fn sdk_default_native_smoke() {
    let test_font_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_assets/fonts/test_font.ttf")
        .to_string_lossy()
        .into_owned();
    let mut game = GoudGame::with_platform(
        GameConfig::default()
            .with_title("native-main-thread-sdk")
            .with_size(160, 120),
    )
    .expect("default SDK native stack should initialize");

    assert!(game.has_platform());
    assert!(game.has_2d_renderer());
    assert!(game.has_3d_renderer());

    let (fb_width, fb_height) = game.get_framebuffer_size();
    assert!(fb_width > 0);
    assert!(fb_height > 0);

    assert!(game.begin_render());
    game.clear(0.08, 0.12, 0.18, 1.0);
    assert!(game.begin_2d_render().is_ok());
    assert!(game.draw_quad(24.0, 24.0, 32.0, 32.0, 1.0, 0.2, 0.2, 1.0));
    assert!(game.draw_text(
        &test_font_path,
        "sdk",
        8.0,
        20.0,
        14.0,
        0.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
    ));
    assert!(game.end_2d_render().is_ok());
    let cube = game.create_cube(0, 1.0, 1.0, 1.0);
    assert_ne!(cube, u32::MAX);
    assert!(game.set_object_position(cube, 0.0, 0.0, 0.0));
    assert!(game.configure_grid(true, 4.0, 4));
    assert!(game.render());
    assert!(game.end_render());

    assert_readback_or_unsupported(
        game.read_default_framebuffer_rgba8(),
        (fb_width * fb_height * 4) as usize,
    );

    game.set_window_size(196, 148)
        .expect("SDK resize request should succeed");
    let (resized_fb_width, resized_fb_height) = pump_until_window_size_sdk(&mut game, 196, 148);
    assert!(resized_fb_width > 0);
    assert!(resized_fb_height > 0);

    assert!(game.begin_render());
    game.clear(0.18, 0.10, 0.10, 1.0);
    assert!(game.end_render());
    assert_readback_or_unsupported(
        game.read_default_framebuffer_rgba8(),
        (resized_fb_width * resized_fb_height * 4) as usize,
    );
}

fn main() {
    if should_skip_native_smoke() {
        eprintln!("skipping native main-thread smoke: no display available");
        return;
    }

    sdk_default_native_smoke();
}
