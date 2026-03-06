//! RigidBody component for physics simulation.
//!
//! The [`RigidBody`] component marks an entity as a physics object that participates
//! in physics simulation. It controls the entity's physics behavior type (dynamic,
//! kinematic, or static) and stores physics state like velocity and forces.
//!
//! # Physics Behavior Types
//!
//! - **Dynamic**: Fully simulated, affected by forces and collisions
//! - **Kinematic**: Moves via velocity but not affected by forces
//! - **Static**: Immovable, acts as obstacles for other bodies
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::components::{RigidBody, RigidBodyType};
//! use goud_engine::core::math::Vec2;
//!
//! // Create a dynamic body (player, enemies, etc.)
//! let player = RigidBody::dynamic()
//!     .with_velocity(Vec2::new(100.0, 0.0))
//!     .with_mass(1.0);
//!
//! // Create a kinematic body (moving platforms)
//! let platform = RigidBody::kinematic()
//!     .with_velocity(Vec2::new(50.0, 0.0));
//!
//! // Create a static body (walls, floors)
//! let wall = RigidBody::static_body();
//! ```
//!
//! # Integration with Physics Systems
//!
//! The physics system reads RigidBody components to:
//! - Apply forces and impulses
//! - Integrate velocity to update position
//! - Handle collisions and constraints
//! - Implement sleeping for optimization
//!
//! # Thread Safety
//!
//! RigidBody is `Send + Sync` and can be safely used in parallel systems.

pub mod body;
pub mod body_type;
pub mod physics_ops;

#[cfg(test)]
mod tests;

pub use body::RigidBody;
pub use body_type::RigidBodyType;
