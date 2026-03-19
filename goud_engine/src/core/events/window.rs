//! Window events.
//!
//! Contains events emitted when the window changes size, position, focus, or
//! receives a close request.

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

/// Emitted when the fullscreen mode changes.
///
/// Contains the new fullscreen mode and the resulting window dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FullscreenChanged {
    /// The new fullscreen mode.
    pub mode: crate::libs::platform::FullscreenMode,
    /// Window width after the mode change.
    pub width: u32,
    /// Window height after the mode change.
    pub height: u32,
}

impl FullscreenChanged {
    /// Creates a new `FullscreenChanged` event.
    #[must_use]
    pub fn new(mode: crate::libs::platform::FullscreenMode, width: u32, height: u32) -> Self {
        Self {
            mode,
            width,
            height,
        }
    }
}

impl Default for FullscreenChanged {
    fn default() -> Self {
        Self::new(crate::libs::platform::FullscreenMode::Windowed, 800, 600)
    }
}
