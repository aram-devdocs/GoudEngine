//! Frame-scoped bump allocation.
//!
//! Provides a [`FrameArena`] backed by [`bumpalo::Bump`] for fast, temporary
//! allocations that are bulk-freed at the end of each frame.
//!
//! - [`FrameArena`]: Bump allocator with reset semantics.
//! - [`ArenaStats`]: Diagnostic counters for monitoring arena utilisation.

pub mod frame_arena;
pub mod stats;

pub use frame_arena::FrameArena;
pub use stats::ArenaStats;
