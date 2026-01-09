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

pub mod error;
pub mod event;
pub mod events;
pub mod handle;
pub mod math;
