//! Frame timing events.
//!
//! Contains events emitted at the start and end of each engine frame.

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
///         let prev_second = (event.total_time - event.delta as f64) as u64;
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
