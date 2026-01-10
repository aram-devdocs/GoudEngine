//! Pre-defined common engine events.
//!
//! This module contains standard events that the engine emits during its lifecycle.
//! Games can subscribe to these events to respond to engine state changes, window
//! events, and frame timing.
//!
//! # Event Categories
//!
//! - **Application Events**: [`AppStarted`], [`AppExiting`] - Engine lifecycle
//! - **Window Events**: [`WindowResized`], [`WindowFocused`], [`WindowMoved`], [`WindowCloseRequested`]
//! - **Frame Events**: [`FrameStarted`], [`FrameEnded`] - Per-frame timing info
//!
//! # Usage
//!
//! Systems can read these events using the standard event system:
//!
//! ```rust
//! use goud_engine::core::events::{WindowResized, FrameStarted};
//! use goud_engine::core::event::{Events, EventReader};
//!
//! // In a system, read window resize events
//! fn handle_resize(events: &Events<WindowResized>) {
//!     let mut reader = events.reader();
//!     for event in reader.read() {
//!         println!("Window resized to {}x{}", event.width, event.height);
//!     }
//! }
//!
//! // Track frame timing
//! fn update_game(frame_events: &Events<FrameStarted>) {
//!     let mut reader = frame_events.reader();
//!     for event in reader.read() {
//!         println!("Frame {}: delta = {}s", event.frame, event.delta);
//!     }
//! }
//! ```
//!
//! # Thread Safety
//!
//! All events in this module are `Send + Sync + 'static` and automatically
//! implement the [`Event`](super::event::Event) trait.

// ============================================================================
// Application Lifecycle Events
// ============================================================================

/// Emitted when the application/engine has fully initialized.
///
/// This event fires once after all core systems are ready but before the
/// first frame begins. Use this for one-time initialization that depends
/// on the engine being fully set up.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::events::AppStarted;
/// use goud_engine::core::event::Events;
///
/// fn on_app_start(events: &Events<AppStarted>) {
///     let mut reader = events.reader();
///     for _ in reader.read() {
///         println!("Application started - performing one-time setup");
///         // Load initial assets, connect to services, etc.
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AppStarted;

/// Emitted when the application is about to exit.
///
/// This event fires before shutdown begins, giving systems a chance to
/// perform cleanup, save state, or release resources gracefully.
///
/// # Exit Reasons
///
/// The `reason` field indicates why the application is exiting:
/// - `User`: User requested exit (close button, quit command)
/// - `Error`: Unrecoverable error occurred
/// - `Programmatic`: Code explicitly requested shutdown
///
/// # Example
///
/// ```rust
/// use goud_engine::core::events::{AppExiting, ExitReason};
/// use goud_engine::core::event::Events;
///
/// fn on_app_exit(events: &Events<AppExiting>) {
///     let mut reader = events.reader();
///     for event in reader.read() {
///         match event.reason {
///             ExitReason::User => println!("User requested exit"),
///             ExitReason::Error => println!("Exiting due to error"),
///             ExitReason::Programmatic => println!("Programmatic shutdown"),
///         }
///         // Save game state, cleanup resources, etc.
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppExiting {
    /// The reason the application is exiting.
    pub reason: ExitReason,
}

impl AppExiting {
    /// Creates an `AppExiting` event with the specified reason.
    #[must_use]
    pub fn new(reason: ExitReason) -> Self {
        Self { reason }
    }

    /// Creates an `AppExiting` event for user-initiated exit.
    #[must_use]
    pub fn user() -> Self {
        Self::new(ExitReason::User)
    }

    /// Creates an `AppExiting` event for error-initiated exit.
    #[must_use]
    pub fn error() -> Self {
        Self::new(ExitReason::Error)
    }

    /// Creates an `AppExiting` event for programmatic exit.
    #[must_use]
    pub fn programmatic() -> Self {
        Self::new(ExitReason::Programmatic)
    }
}

impl Default for AppExiting {
    fn default() -> Self {
        Self::new(ExitReason::User)
    }
}

/// Reason for application exit.
///
/// Used with [`AppExiting`] to indicate why the application is shutting down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ExitReason {
    /// User requested exit (close button, Alt+F4, quit menu, etc.).
    #[default]
    User,
    /// An unrecoverable error occurred.
    Error,
    /// Code explicitly requested shutdown.
    Programmatic,
}

// ============================================================================
// Window Events
// ============================================================================

/// Emitted when the window has been resized.
///
/// This event fires whenever the window dimensions change, whether from
/// user action (dragging window edges) or programmatic resize.
///
/// # Common Uses
///
/// - Update camera aspect ratios
/// - Resize render targets and framebuffers
/// - Adjust UI layout
///
/// # Example
///
/// ```rust
/// use goud_engine::core::events::WindowResized;
/// use goud_engine::core::event::Events;
///
/// fn handle_resize(events: &Events<WindowResized>) {
///     let mut reader = events.reader();
///     for event in reader.read() {
///         let aspect = event.width as f32 / event.height as f32;
///         println!("New size: {}x{}, aspect: {:.2}", event.width, event.height, aspect);
///         // Update camera, resize framebuffers, etc.
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowResized {
    /// New window width in pixels.
    pub width: u32,
    /// New window height in pixels.
    pub height: u32,
}

impl WindowResized {
    /// Creates a new `WindowResized` event.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the aspect ratio (width / height).
    ///
    /// Returns 0.0 if height is 0 to avoid division by zero.
    #[must_use]
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            0.0
        } else {
            self.width as f32 / self.height as f32
        }
    }
}

impl Default for WindowResized {
    fn default() -> Self {
        Self::new(800, 600)
    }
}

/// Emitted when the window gains or loses focus.
///
/// Use this to pause/resume gameplay, mute audio, or adjust input handling
/// based on whether the window is in the foreground.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::events::WindowFocused;
/// use goud_engine::core::event::Events;
///
/// fn handle_focus(events: &Events<WindowFocused>) {
///     let mut reader = events.reader();
///     for event in reader.read() {
///         if event.focused {
///             println!("Window gained focus - resuming");
///             // Resume game, unmute audio
///         } else {
///             println!("Window lost focus - pausing");
///             // Pause game, mute audio
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowFocused {
    /// `true` if the window now has focus, `false` if it lost focus.
    pub focused: bool,
}

impl WindowFocused {
    /// Creates a new `WindowFocused` event.
    #[must_use]
    pub fn new(focused: bool) -> Self {
        Self { focused }
    }

    /// Creates an event indicating the window gained focus.
    #[must_use]
    pub fn gained() -> Self {
        Self::new(true)
    }

    /// Creates an event indicating the window lost focus.
    #[must_use]
    pub fn lost() -> Self {
        Self::new(false)
    }
}

impl Default for WindowFocused {
    fn default() -> Self {
        Self::new(true)
    }
}

/// Emitted when the window has been moved.
///
/// Contains the new position of the window's top-left corner in screen
/// coordinates.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::events::WindowMoved;
/// use goud_engine::core::event::Events;
///
/// fn handle_move(events: &Events<WindowMoved>) {
///     let mut reader = events.reader();
///     for event in reader.read() {
///         println!("Window moved to ({}, {})", event.x, event.y);
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowMoved {
    /// New X position of window's top-left corner in screen coordinates.
    pub x: i32,
    /// New Y position of window's top-left corner in screen coordinates.
    pub y: i32,
}

impl WindowMoved {
    /// Creates a new `WindowMoved` event.
    #[must_use]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Default for WindowMoved {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Emitted when the user requests to close the window.
///
/// This is a request, not a command. Games can intercept this event to
/// show confirmation dialogs, save progress, or prevent accidental closure.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::events::WindowCloseRequested;
/// use goud_engine::core::event::Events;
///
/// fn handle_close_request(events: &Events<WindowCloseRequested>) {
///     let mut reader = events.reader();
///     for _event in reader.read() {
///         println!("Close requested - showing confirmation dialog");
///         // Show "Are you sure?" dialog, save game, etc.
///         // Only proceed with exit if user confirms
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WindowCloseRequested;

// ============================================================================
// Frame Events
// ============================================================================

/// Emitted at the beginning of each frame.
///
/// This event provides timing information for the new frame. Use it to
/// update game logic, animations, and physics with consistent delta time.
///
/// # Timing Fields
///
/// - `frame`: Monotonically increasing frame counter (starts at 0)
/// - `delta`: Time elapsed since the previous frame in seconds
/// - `total_time`: Total time elapsed since engine start in seconds
///
/// # Example
///
/// ```ignore
/// use goud_engine::core::events::FrameStarted;
/// use goud_engine::core::event::Events;
///
/// fn update_game(events: &Events<FrameStarted>) {
///     let mut reader = events.reader();
///     for event in reader.read() {
///         // Update physics with delta time
///         let velocity_change = 9.8 * event.delta; // gravity
///
///         // Check if it's a new second for FPS display
///         let prev_second = (event.total_time - event.delta) as u64;
///         let curr_second = event.total_time as u64;
///         if curr_second > prev_second {
///             println!("Frame {} - Total time: {:.1}s", event.frame, event.total_time);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameStarted {
    /// The frame number (0-indexed, monotonically increasing).
    pub frame: u64,
    /// Time elapsed since the previous frame, in seconds.
    pub delta: f32,
    /// Total time elapsed since engine start, in seconds.
    pub total_time: f64,
}

impl FrameStarted {
    /// Creates a new `FrameStarted` event.
    #[must_use]
    pub fn new(frame: u64, delta: f32, total_time: f64) -> Self {
        Self {
            frame,
            delta,
            total_time,
        }
    }

    /// Returns the current frames per second based on delta time.
    ///
    /// Returns `f32::INFINITY` if delta is 0 (to avoid division by zero).
    #[must_use]
    pub fn fps(&self) -> f32 {
        if self.delta == 0.0 {
            f32::INFINITY
        } else {
            1.0 / self.delta
        }
    }
}

impl Default for FrameStarted {
    fn default() -> Self {
        Self::new(0, 0.0, 0.0)
    }
}

/// Emitted at the end of each frame, after all systems have run.
///
/// Use this for cleanup, profiling, or operations that should happen
/// after all game logic but before the next frame begins.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::events::FrameEnded;
/// use goud_engine::core::event::Events;
///
/// fn end_frame_profiling(events: &Events<FrameEnded>) {
///     let mut reader = events.reader();
///     for event in reader.read() {
///         // Record frame timing for profiling
///         println!("Frame {} completed in {:.2}ms", event.frame, event.frame_time_ms);
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameEnded {
    /// The frame number that just completed.
    pub frame: u64,
    /// Time taken to process this frame, in milliseconds.
    pub frame_time_ms: f32,
}

impl FrameEnded {
    /// Creates a new `FrameEnded` event.
    #[must_use]
    pub fn new(frame: u64, frame_time_ms: f32) -> Self {
        Self {
            frame,
            frame_time_ms,
        }
    }

    /// Returns the frame time in seconds.
    #[must_use]
    pub fn frame_time_secs(&self) -> f32 {
        self.frame_time_ms / 1000.0
    }
}

impl Default for FrameEnded {
    fn default() -> Self {
        Self::new(0, 0.0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::Event;

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
        let debug_str = format!("{event:?}");
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
}
