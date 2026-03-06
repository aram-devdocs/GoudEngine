//! Tests for all engine events.

use crate::core::event::Event;
use crate::core::events::{
    AppExiting, AppStarted, ExitReason, FrameEnded, FrameStarted, WindowCloseRequested,
    WindowFocused, WindowMoved, WindowResized,
};

// =========================================================================
// Event Trait Bound Tests
// =========================================================================

/// Test that all events implement the Event trait
#[test]
fn test_all_events_implement_event_trait() {
    fn accepts_event<E: Event>(_: E) {}

    // Application events
    accepts_event(AppStarted);
    accepts_event(AppExiting::user());

    // Window events
    accepts_event(WindowResized::new(800, 600));
    accepts_event(WindowFocused::gained());
    accepts_event(WindowMoved::new(100, 100));
    accepts_event(WindowCloseRequested);

    // Frame events
    accepts_event(FrameStarted::new(0, 0.016, 0.0));
    accepts_event(FrameEnded::new(0, 16.0));
}

/// Test that all events are Send + Sync
#[test]
fn test_all_events_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<AppStarted>();
    assert_send_sync::<AppExiting>();
    assert_send_sync::<ExitReason>();
    assert_send_sync::<WindowResized>();
    assert_send_sync::<WindowFocused>();
    assert_send_sync::<WindowMoved>();
    assert_send_sync::<WindowCloseRequested>();
    assert_send_sync::<FrameStarted>();
    assert_send_sync::<FrameEnded>();
}

/// Test that all events are 'static
#[test]
fn test_all_events_are_static() {
    fn assert_static<T: 'static>() {}

    assert_static::<AppStarted>();
    assert_static::<AppExiting>();
    assert_static::<ExitReason>();
    assert_static::<WindowResized>();
    assert_static::<WindowFocused>();
    assert_static::<WindowMoved>();
    assert_static::<WindowCloseRequested>();
    assert_static::<FrameStarted>();
    assert_static::<FrameEnded>();
}

// =========================================================================
// AppStarted Tests
// =========================================================================

#[test]
fn test_app_started_default() {
    let event = AppStarted::default();
    assert_eq!(event, AppStarted);
}

#[test]
fn test_app_started_debug() {
    let event = AppStarted;
    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("AppStarted"));
}

// =========================================================================
// AppExiting Tests
// =========================================================================

#[test]
fn test_app_exiting_user() {
    let event = AppExiting::user();
    assert_eq!(event.reason, ExitReason::User);
}

#[test]
fn test_app_exiting_error() {
    let event = AppExiting::error();
    assert_eq!(event.reason, ExitReason::Error);
}

#[test]
fn test_app_exiting_programmatic() {
    let event = AppExiting::programmatic();
    assert_eq!(event.reason, ExitReason::Programmatic);
}

#[test]
fn test_app_exiting_default() {
    let event = AppExiting::default();
    assert_eq!(event.reason, ExitReason::User);
}

#[test]
fn test_exit_reason_default() {
    let reason = ExitReason::default();
    assert_eq!(reason, ExitReason::User);
}

// =========================================================================
// WindowResized Tests
// =========================================================================

#[test]
fn test_window_resized_new() {
    let event = WindowResized::new(1920, 1080);
    assert_eq!(event.width, 1920);
    assert_eq!(event.height, 1080);
}

#[test]
fn test_window_resized_aspect_ratio() {
    let event = WindowResized::new(1920, 1080);
    let aspect = event.aspect_ratio();
    assert!((aspect - 16.0 / 9.0).abs() < 0.001);
}

#[test]
fn test_window_resized_aspect_ratio_zero_height() {
    let event = WindowResized::new(800, 0);
    assert_eq!(event.aspect_ratio(), 0.0);
}

#[test]
fn test_window_resized_default() {
    let event = WindowResized::default();
    assert_eq!(event.width, 800);
    assert_eq!(event.height, 600);
}

// =========================================================================
// WindowFocused Tests
// =========================================================================

#[test]
fn test_window_focused_gained() {
    let event = WindowFocused::gained();
    assert!(event.focused);
}

#[test]
fn test_window_focused_lost() {
    let event = WindowFocused::lost();
    assert!(!event.focused);
}

#[test]
fn test_window_focused_new() {
    let event = WindowFocused::new(true);
    assert!(event.focused);

    let event = WindowFocused::new(false);
    assert!(!event.focused);
}

#[test]
fn test_window_focused_default() {
    let event = WindowFocused::default();
    assert!(event.focused);
}

// =========================================================================
// WindowMoved Tests
// =========================================================================

#[test]
fn test_window_moved_new() {
    let event = WindowMoved::new(100, 200);
    assert_eq!(event.x, 100);
    assert_eq!(event.y, 200);
}

#[test]
fn test_window_moved_negative() {
    let event = WindowMoved::new(-50, -25);
    assert_eq!(event.x, -50);
    assert_eq!(event.y, -25);
}

#[test]
fn test_window_moved_default() {
    let event = WindowMoved::default();
    assert_eq!(event.x, 0);
    assert_eq!(event.y, 0);
}

// =========================================================================
// WindowCloseRequested Tests
// =========================================================================

#[test]
fn test_window_close_requested_default() {
    let event = WindowCloseRequested::default();
    assert_eq!(event, WindowCloseRequested);
}

// =========================================================================
// FrameStarted Tests
// =========================================================================

#[test]
fn test_frame_started_new() {
    let event = FrameStarted::new(42, 0.016667, 10.5);
    assert_eq!(event.frame, 42);
    assert!((event.delta - 0.016667).abs() < 0.0001);
    assert!((event.total_time - 10.5).abs() < 0.0001);
}

#[test]
fn test_frame_started_fps() {
    let event = FrameStarted::new(0, 0.016667, 0.0);
    let fps = event.fps();
    assert!((fps - 60.0).abs() < 1.0); // ~60 FPS
}

#[test]
fn test_frame_started_fps_zero_delta() {
    let event = FrameStarted::new(0, 0.0, 0.0);
    assert!(event.fps().is_infinite());
}

#[test]
fn test_frame_started_default() {
    let event = FrameStarted::default();
    assert_eq!(event.frame, 0);
    assert_eq!(event.delta, 0.0);
    assert_eq!(event.total_time, 0.0);
}

// =========================================================================
// FrameEnded Tests
// =========================================================================

#[test]
fn test_frame_ended_new() {
    let event = FrameEnded::new(100, 16.5);
    assert_eq!(event.frame, 100);
    assert!((event.frame_time_ms - 16.5).abs() < 0.001);
}

#[test]
fn test_frame_ended_frame_time_secs() {
    let event = FrameEnded::new(0, 16.5);
    let secs = event.frame_time_secs();
    assert!((secs - 0.0165).abs() < 0.0001);
}

#[test]
fn test_frame_ended_default() {
    let event = FrameEnded::default();
    assert_eq!(event.frame, 0);
    assert_eq!(event.frame_time_ms, 0.0);
}

// =========================================================================
// Clone and Copy Tests
// =========================================================================

#[test]
fn test_events_are_copy() {
    // Verify Copy trait by assignment
    let event1 = WindowResized::new(800, 600);
    let event2 = event1; // Copy
    let _event3 = event1; // event1 still valid due to Copy
    assert_eq!(event2.width, 800);
}

#[test]
fn test_events_are_clone() {
    let event = FrameStarted::new(1, 0.016, 1.0);
    let cloned = event.clone();
    assert_eq!(event.frame, cloned.frame);
    assert_eq!(event.delta, cloned.delta);
}

// =========================================================================
// Hash Tests
// =========================================================================

#[test]
fn test_events_are_hashable() {
    use std::collections::HashSet;

    let mut set: HashSet<WindowResized> = HashSet::new();
    set.insert(WindowResized::new(800, 600));
    set.insert(WindowResized::new(1920, 1080));
    set.insert(WindowResized::new(800, 600)); // Duplicate

    assert_eq!(set.len(), 2);
}
