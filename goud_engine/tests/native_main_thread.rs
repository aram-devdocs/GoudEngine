#![cfg(feature = "native")]

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

fn sdk_default_native_smoke() {
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
        "/Users/aramhammoudeh/dev/game/GoudEngine-issue-280/goud_engine/test_assets/fonts/test_font.ttf",
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

    let readback = game
        .read_default_framebuffer_rgba8()
        .expect("SDK framebuffer readback should succeed");
    assert_eq!(readback.len(), (fb_width * fb_height * 4) as usize);

    game.set_window_size(196, 148)
        .expect("SDK resize request should succeed");
    let (resized_fb_width, resized_fb_height) = pump_until_window_size_sdk(&mut game, 196, 148);
    assert!(resized_fb_width > 0);
    assert!(resized_fb_height > 0);

    assert!(game.begin_render());
    game.clear(0.18, 0.10, 0.10, 1.0);
    assert!(game.end_render());
    let resized_readback = game
        .read_default_framebuffer_rgba8()
        .expect("SDK resized framebuffer readback should succeed");
    assert_eq!(
        resized_readback.len(),
        (resized_fb_width * resized_fb_height * 4) as usize
    );
}

fn main() {
    if should_skip_native_smoke() {
        eprintln!("skipping native main-thread smoke: no display available");
        return;
    }

    sdk_default_native_smoke();
}
