//! Main game abstraction for Rust-native game development.
//!
//! Contains [`GoudGame`], the primary entry point managing the ECS world,
//! game loop, and convenient methods for entity and component operations.

pub(crate) mod instance;
mod instance_runtime;
mod instance_transitions;
mod providers;
mod ui_frame;

#[cfg(test)]
mod tests;

pub use instance::GoudGame;
