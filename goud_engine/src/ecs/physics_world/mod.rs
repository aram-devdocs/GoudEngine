//! Physics world module.
//!
//! Re-exports [`PhysicsWorld`] as the public API. Internal implementation is
//! split across focused submodules:
//!
//! - `resource`: struct definition, construction, and builder methods
//! - `simulation`: accessors, mutators, simulation control, and utilities

pub mod interpolation;
mod resource;
mod simulation;
#[cfg(test)]
mod tests;

pub use interpolation::PhysicsInterpolation;
pub use resource::PhysicsWorld;
