//! Resource system for the ECS.
//!
//! Resources are singleton data that exists outside the entity-component model.
//! Unlike components, resources are not attached to entities - they are globally
//! accessible within the World.
//!
//! # Examples of Resources
//!
//! - **Time**: Delta time, total elapsed time, frame count
//! - **Input State**: Keyboard, mouse, and gamepad state
//! - **Asset Manager**: Loaded textures, sounds, and other assets
//! - **Configuration**: Game settings, debug flags
//!
//! # Resource vs Component
//!
//! | Aspect | Resource | Component |
//! |--------|----------|-----------|
//! | Cardinality | One per type | Many per type |
//! | Ownership | Owned by World | Attached to Entity |
//! | Access | `Res<T>`, `ResMut<T>` | `Query<&T>`, `Query<&mut T>` |
//! | Use Case | Global state | Per-entity state |
//!
//! # Usage
//!
//! ```ignore
//! use goud_engine::ecs::{World, Resource};
//!
//! // Define a resource
//! struct Time {
//!     delta: f32,
//!     total: f32,
//! }
//! impl Resource for Time {}
//!
//! // Insert resource into World
//! let mut world = World::new();
//! world.insert_resource(Time { delta: 0.016, total: 0.0 });
//!
//! // Access resource (immutably)
//! let time = world.get_resource::<Time>().unwrap();
//! println!("Delta: {}", time.delta);
//!
//! // Access resource (mutably)
//! let time = world.get_resource_mut::<Time>().unwrap();
//! time.total += time.delta;
//! ```
//!
//! # System Parameters
//!
//! In systems, use `Res<T>` and `ResMut<T>` for resource access:
//!
//! ```ignore
//! fn update_system(time: Res<Time>, mut query: Query<&mut Position>) {
//!     for mut pos in query.iter_mut() {
//!         pos.x += time.delta * 10.0;
//!     }
//! }
//! ```
//!
//! # Thread Safety
//!
//! Resources must be `Send + Sync` for use in parallel systems. For resources
//! that are not thread-safe (e.g., window handles), use non-send resources
//! (future: `NonSend<T>`, `NonSendMut<T>`).

pub mod access;
pub mod non_send;
pub mod types;

#[cfg(test)]
mod tests_non_send;
#[cfg(test)]
mod tests_send;

// Re-export all public items so the public API is unchanged.
pub use access::{Res, ResMut};
pub use non_send::{
    NonSend, NonSendMarker, NonSendMut, NonSendResource, NonSendResourceId, NonSendResources,
};
pub use types::{Resource, ResourceId, Resources};
