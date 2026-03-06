//! Event system for decoupled communication between engine systems.
//!
//! Events enable systems to communicate without direct dependencies. The event
//! system uses a double-buffered queue pattern where events are written during
//! one frame and read during the next.
//!
//! # Design
//!
//! The event system consists of:
//! - [`Event`]: Marker trait for types that can be sent as events
//! - [`EventQueue<E>`]: Double-buffered storage for events of type E
//! - [`EventReader<E>`]: Read-only accessor for consuming events
//! - [`EventWriter<E>`]: Write-only accessor for sending events
//! - [`Events<E>`]: ECS resource wrapper that manages the full event lifecycle
//!
//! # Usage
//!
//! Any type that is `Send + Sync + 'static` automatically implements `Event`:
//!
//! ```rust
//! use goud_engine::core::event::Event;
//!
//! // Custom event types
//! #[derive(Debug, Clone)]
//! struct PlayerDied {
//!     player_id: u32,
//!     cause: String,
//! }
//!
//! // PlayerDied automatically implements Event
//! fn send_event<E: Event>(event: E) {
//!     // ...
//! }
//!
//! let event = PlayerDied {
//!     player_id: 1,
//!     cause: "fell into lava".to_string(),
//! };
//! send_event(event);
//! ```
//!
//! # Thread Safety
//!
//! Events must be `Send + Sync` to support parallel system execution. The
//! `'static` bound ensures events don't contain borrowed data, which would
//! complicate lifetime management across frame boundaries.

pub mod queue;
pub mod reader;
pub mod resource;
pub(crate) mod traits;
pub mod writer;

pub use queue::EventQueue;
pub use reader::{EventReader, EventReaderIter};
pub use resource::Events;
pub use traits::Event;
pub use writer::EventWriter;

#[cfg(test)]
mod tests;
