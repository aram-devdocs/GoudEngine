//! Physics world module.
//!
//! Re-exports [`PhysicsWorld`] as the public API. Internal implementation is
//! split across focused submodules:
//!
//! - `resource`: struct definition, construction, and builder methods
//! - `simulation`: accessors, mutators, simulation control, and utilities

mod resource;
mod simulation;
mod tests;

pub use resource::PhysicsWorld;
