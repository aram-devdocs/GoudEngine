use super::*;
use crate::sdk::GameConfig;
#[cfg(feature = "native")]
use crate::{core::input_manager::InputManager, libs::platform::PlatformBackend};

#[cfg(feature = "native")]
struct TestPlatform {
    should_close: bool,
    logical_size: (u32, u32),
    framebuffer_size: (u32, u32),
    first_poll_size: Option<((u32, u32), (u32, u32))>,
}

#[cfg(feature = "native")]
impl TestPlatform {
    fn with_first_poll_size(logical_size: (u32, u32), framebuffer_size: (u32, u32)) -> Self {
        Self {
            should_close: false,
            logical_size: (800, 600),
            framebuffer_size: (800, 600),
            first_poll_size: Some((logical_size, framebuffer_size)),
        }
    }
}

#[cfg(feature = "native")]
impl PlatformBackend for TestPlatform {
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn set_should_close(&mut self, should_close: bool) {
        self.should_close = should_close;
    }

    fn poll_events(&mut self, _input: &mut InputManager) -> f32 {
        if let Some((logical_size, framebuffer_size)) = self.first_poll_size.take() {
            self.logical_size = logical_size;
            self.framebuffer_size = framebuffer_size;
        }
        1.0 / 60.0
    }

    fn swap_buffers(&mut self) {}

    fn get_size(&self) -> (u32, u32) {
        self.logical_size
    }

    fn request_size(&mut self, width: u32, height: u32) -> bool {
        self.logical_size = (width, height);
        self.framebuffer_size = (width, height);
        true
    }

    fn get_framebuffer_size(&self) -> (u32, u32) {
        self.framebuffer_size
    }
}

#[test]
fn test_should_close_headless() {
    let game = GoudGame::new(GameConfig::default()).unwrap();
    // No platform => should_close returns false
    assert!(!game.should_close());
}

#[test]
fn test_set_should_close_headless_returns_error() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    let result = game.set_should_close(true);
    assert!(result.is_err());
}

#[test]
fn test_poll_events_headless_returns_error() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    let result = game.poll_events();
    assert!(result.is_err());
}

#[test]
fn test_swap_buffers_headless_returns_error() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    let result = game.swap_buffers();
    assert!(result.is_err());
}

#[test]
fn test_clear_headless_no_panic() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    // No backend => clear is a no-op, should not panic
    game.clear(0.0, 0.0, 0.0, 1.0);
}

#[test]
fn test_get_window_size_headless() {
    let game = GoudGame::new(GameConfig::new("Test", 1280, 720)).unwrap();
    // No platform => falls back to config size
    assert_eq!(game.get_window_size(), (1280, 720));
}

#[test]
fn test_has_platform_headless() {
    let game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.has_platform());
}

#[test]
fn test_get_delta_time_initial() {
    let game = GoudGame::new(GameConfig::default()).unwrap();
    assert!((game.get_delta_time() - 0.0).abs() < 0.001);
}

#[test]
fn test_get_framebuffer_size_headless() {
    let game = GoudGame::new(GameConfig::new("Test", 1920, 1080)).unwrap();
    // No platform => falls back to config size
    assert_eq!(game.get_framebuffer_size(), (1920, 1080));
}

#[test]
fn test_set_window_size_headless_returns_error() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(game.set_window_size(640, 480).is_err());
}

#[test]
fn test_read_default_framebuffer_rgba8_headless_returns_error() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(game.read_default_framebuffer_rgba8().is_err());
}

#[cfg(feature = "native")]
#[test]
fn test_poll_events_clears_stale_resize_events_on_following_frames() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    game.platform = Some(Box::new(TestPlatform::with_first_poll_size(
        (1024, 768),
        (1024, 768),
    )));

    game.poll_events().expect("first poll should succeed");
    let mut first_reader = game.window_resized_events.reader();
    let first_events: Vec<_> = first_reader.read().copied().collect();
    assert_eq!(first_events, vec![WindowResized::new(1024, 768)]);

    game.poll_events().expect("second poll should succeed");
    let mut second_reader = game.window_resized_events.reader();
    assert!(
        second_reader.read().next().is_none(),
        "resize event should be cleared on the next frame when no new resize occurs"
    );
}

#[test]
fn test_window_destroy_invalid_returns_false() {
    assert!(!Window::destroy(GOUD_INVALID_CONTEXT_ID));
}
