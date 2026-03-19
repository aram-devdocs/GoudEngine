#[cfg(any(feature = "lua", all(feature = "native", not(target_os = "macos"))))]
use super::*;

#[cfg(all(feature = "native", not(target_os = "macos")))]
use std::path::PathBuf;
#[cfg(feature = "lua")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "lua")]
use std::sync::Arc;
#[cfg(all(feature = "native", not(target_os = "macos")))]
use std::thread;
#[cfg(all(feature = "native", not(target_os = "macos")))]
use std::time::Duration;

#[cfg(all(feature = "native", not(target_os = "macos")))]
use crate::sdk::{RenderBackendKind, WindowBackendKind};

#[cfg(all(feature = "native", not(target_os = "macos")))]
fn should_skip_native_runtime_test() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none()
    }

    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

#[cfg(all(feature = "native", not(target_os = "macos")))]
fn test_font_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_assets/fonts/test_font.ttf")
        .to_string_lossy()
        .into_owned()
}

#[cfg(all(feature = "native", not(target_os = "macos")))]
fn assert_readback_or_unsupported(
    result: crate::core::error::GoudResult<Vec<u8>>,
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

#[cfg(feature = "lua")]
#[test]
fn test_new_bootstraps_embedded_lua_runtime() {
    let game = GoudGame::new(GameConfig::default()).expect("headless game should initialize");

    assert!(game.lua_runtime.is_ready());
}

#[cfg(feature = "lua")]
#[test]
fn test_embedded_lua_runtime_drops_with_game() {
    let drop_probe = Arc::new(AtomicUsize::new(0));

    {
        let mut game =
            GoudGame::new(GameConfig::default()).expect("headless game should initialize");
        game.lua_runtime.set_drop_probe(drop_probe.clone());
    }

    assert_eq!(drop_probe.load(Ordering::SeqCst), 1);
}

#[cfg(all(feature = "lua", feature = "native", not(target_os = "macos")))]
#[test]
fn test_with_platform_bootstraps_embedded_lua_runtime() {
    if should_skip_native_runtime_test() {
        eprintln!("skipping native runtime lua test: no display available");
        return;
    }

    let game = GoudGame::with_platform(
        GameConfig::default().with_title("native-lua-bootstrap-instance-test"),
    )
    .expect("native game should initialize with lua enabled");

    assert!(game.has_platform());
    assert!(game.lua_runtime.is_ready());
}

// winit requires the macOS main thread, which the unit-test harness does not provide.
#[cfg(all(feature = "native", not(target_os = "macos")))]
#[test]
fn test_with_platform_default_native_stack_initializes_renderers_and_readback() {
    if should_skip_native_runtime_test() {
        eprintln!("skipping native runtime unit test: no display available");
        return;
    }

    let mut game = GoudGame::with_platform(
        GameConfig::default().with_title("native-stack-smoke-instance-test"),
    )
    .expect("default native stack should initialize");

    assert_eq!(game.config.render_backend, RenderBackendKind::Wgpu);
    assert_eq!(game.config.window_backend, WindowBackendKind::Winit);
    assert!(game.has_platform());
    assert!(game.has_2d_renderer());
    assert!(game.has_3d_renderer());
    let asset_server = game
        .asset_server
        .as_ref()
        .expect("native game should initialize the asset server");
    assert!(asset_server.has_loader_for_type::<crate::assets::loaders::TextureAsset>());
    assert!(asset_server.has_loader_for_type::<crate::assets::loaders::ShaderAsset>());
    assert!(asset_server.has_loader_for_type::<crate::assets::loaders::MaterialAsset>());
    assert!(asset_server.has_loader_for_type::<crate::assets::loaders::MeshAsset>());
    let sprite_batch = game
        .sprite_batch
        .as_ref()
        .expect("native game should initialize the sprite batch");
    assert!(
        sprite_batch.config.shader_asset.is_valid(),
        "native bootstrap should configure a valid sprite shader asset handle"
    );
    assert!(
        asset_server
            .get(&sprite_batch.config.shader_asset)
            .is_some(),
        "configured sprite shader asset should be loaded into the asset server"
    );

    let (fb_width, fb_height) = game.get_framebuffer_size();
    assert!(fb_width > 0);
    assert!(fb_height > 0);

    assert!(game.begin_render());
    game.clear(0.15, 0.25, 0.35, 1.0);
    assert!(game.begin_2d_render().is_ok());
    assert!(game.draw_quad(32.0, 32.0, 24.0, 24.0, 1.0, 0.0, 0.0, 1.0));
    assert!(game.draw_text(
        &test_font_path(),
        "wgpu",
        12.0,
        18.0,
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

    game.set_window_size(176, 132)
        .expect("window resize request should succeed");
    for _ in 0..120 {
        game.poll_events()
            .expect("poll after resize should succeed");
        if game.get_window_size() == (176, 132) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert_eq!(game.get_window_size(), (176, 132));
    let (resized_fb_width, resized_fb_height) = game.get_framebuffer_size();
    assert!(resized_fb_width > 0);
    assert!(resized_fb_height > 0);
    assert_readback_or_unsupported(
        game.read_default_framebuffer_rgba8(),
        (resized_fb_width * resized_fb_height * 4) as usize,
    );
}
