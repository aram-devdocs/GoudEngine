//! Main game abstraction for Rust-native game development.
//!
//! Contains [`GoudGame`], the primary entry point managing the ECS world,
//! game loop, and convenient methods for entity and component operations.

mod instance;
mod instance_transitions;

#[cfg(test)]
mod tests;

pub use instance::GoudGame;
