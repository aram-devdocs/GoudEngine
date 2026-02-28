//! GoudEngine Rust SDK
//!
//! This crate re-exports the internal `goud_engine::sdk` module directly.
//! Unlike C#, Python, and TypeScript SDKs which call through the C FFI boundary,
//! the Rust SDK links directly against the engine for zero overhead.
//!
//! This is a documented exception to the FFI-only rule -- Rust calling Rust
//! through C-ABI FFI adds mutex locks and context lookups for zero benefit.

pub use goud_engine::sdk::*;
