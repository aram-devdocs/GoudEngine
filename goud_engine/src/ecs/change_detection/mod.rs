//! Change detection for the ECS.
//!
//! This module provides types for tracking when components are added or
//! modified. The engine uses a monotonically increasing tick counter to
//! record when each component was last inserted or mutated.
//!
//! # Key Types
//!
//! - [`Tick`]: A newtype around `u32` representing a point in time.
//! - [`ComponentTicks`]: Stores the `added` and `changed` ticks for a
//!   single component instance.
//!
//! # How It Works
//!
//! Every [`World`](crate::ecs::World) maintains a `change_tick` that is
//! incremented at system boundaries. When a component is inserted, both
//! its `added` and `changed` ticks are set to the current `change_tick`.
//! When a component is mutated (via `get_mut`), only its `changed` tick
//! is updated.
//!
//! Query filters like `Changed<T>` and `Added<T>` compare the stored
//! ticks against the world's `last_change_tick` to determine which
//! components have been modified since the last system ran.

mod ticks;

#[cfg(test)]
mod tests;

pub use ticks::{ComponentTicks, Tick};
