//! Core engine utilities and infrastructure.
//!
//! This module contains foundational types and patterns used throughout the engine:
//!
//! - **Error handling**: Unified error types with FFI-compatible error codes
//! - **Handles**: Type-safe, generation-counted references to engine objects
//! - **Events**: Decoupled communication between engine systems
//! - **Math**: FFI-safe mathematical types (vectors, matrices, colors)
//!
//! These utilities form the foundation for all other engine subsystems and
//! provide consistent patterns for cross-language interoperability.

pub mod error;

// Future submodules will be added as they are implemented:
// pub mod handle;   // Step 1.2.1-1.2.6
// pub mod event;    // Step 1.3.1-1.3.4
// pub mod events;   // Step 1.3.5
// pub mod math;     // Step 1.4.1-1.4.4
