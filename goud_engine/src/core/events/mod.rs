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

mod app;
mod frame;
#[cfg(test)]
mod tests;
mod window;

pub use app::{AppExiting, AppStarted, ExitReason};
pub use frame::{FrameEnded, FrameStarted};
pub use window::{
    FullscreenChanged, WindowCloseRequested, WindowFocused, WindowMoved, WindowResized,
};
