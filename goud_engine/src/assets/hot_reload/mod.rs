//! Hot reloading system for detecting and reloading changed assets.
//!
//! # Overview
//!
//! The hot reload system watches asset files for changes and automatically
//! reloads them without restarting the application. This is essential for
//! rapid iteration during development.
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                     HotReloadWatcher                        │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
//! │  │ File System  │  │   Channel    │  │    Event     │    │
//! │  │   Watcher    │──│   Receiver   │──│   Processor  │    │
//! │  └──────────────┘  └──────────────┘  └──────────────┘    │
//! └────────────────────────────────────────────────────────────┘
//!         │                                          │
//!         ▼                                          ▼
//!    File Change                              Asset Reload
//!     Detected                                  Triggered
//! ```
//!
//! # Example
//!
//! ```no_run
//! use goud_engine::assets::{AssetServer, HotReloadWatcher};
//!
//! let mut server = AssetServer::new();
//! let mut watcher = HotReloadWatcher::new(&server).unwrap();
//!
//! // In your game loop
//! loop {
//!     watcher.process_events(&mut server);
//!     // ... rest of game loop
//! }
//! ```

mod config;
mod events;
mod watcher;

pub use config::HotReloadConfig;
pub use events::AssetChangeEvent;
pub use watcher::HotReloadWatcher;
