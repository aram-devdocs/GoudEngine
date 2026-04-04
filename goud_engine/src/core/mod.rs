//! Core engine utilities and infrastructure.
//!
//! This module contains foundational types and patterns used throughout the engine:
//!
//! - **Error handling**: Unified error types with FFI-compatible error codes
//! - **Handles**: Type-safe, generation-counted references to engine objects
//! - **Events**: Decoupled communication between engine systems
//! - **Common Events**: Pre-defined engine events (app lifecycle, window, frame)
//! - **Math**: FFI-safe mathematical types (vectors, matrices, colors)
//!
//! These utilities form the foundation for all other engine subsystems and
//! provide consistent patterns for cross-language interoperability.

pub mod context_id;
pub mod error;
pub mod event;
pub mod events;
pub mod handle;
pub mod math;
pub mod types;

pub mod arena;
pub mod debugger;
pub mod networking;
pub mod pool;
pub mod providers;
pub mod serialization;

/// Fast hash map using `FxHashMap` — ideal for integer keys on hot paths.
pub type FastMap<K, V> = rustc_hash::FxHashMap<K, V>;
/// Fast hash set using `FxHashSet` — ideal for integer keys on hot paths.
pub type FastSet<V> = rustc_hash::FxHashSet<V>;

#[cfg(feature = "native")]
pub mod input_manager;
