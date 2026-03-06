//! Hierarchy components for parent-child entity relationships.
//!
//! This module provides components for building entity hierarchies, enabling
//! parent-child relationships between entities. This is essential for:
//!
//! - Scene graphs where child transforms are relative to parent transforms
//! - UI layouts where child widgets are positioned within parent containers
//! - Game object grouping where destroying a parent also destroys children
//!
//! # Components
//!
//! - [`Parent`]: Points to the parent entity (stored on child entities)
//! - [`Children`]: Lists all child entities (stored on parent entities)
//! - [`Name`]: Human-readable name for debugging and editor integration
//!
//! # Design Philosophy
//!
//! The hierarchy system uses a bidirectional pointer approach:
//! - Children point up to their parent via [`Parent`]
//! - Parents point down to their children via [`Children`]
//!
//! This redundancy enables efficient traversal in both directions and
//! matches the design of major engines like Bevy, Unity, and Godot.
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::{World, Entity};
//! use goud_engine::ecs::components::{Parent, Children, Name};
//!
//! let mut world = World::new();
//!
//! // Create parent entity
//! let parent = world.spawn_empty();
//! world.insert(parent, Name::new("Player"));
//! world.insert(parent, Children::new());
//!
//! // Create child entity
//! let child = world.spawn_empty();
//! world.insert(child, Name::new("Weapon"));
//! world.insert(child, Parent::new(parent));
//!
//! // Update parent's children list (normally done by a hierarchy system)
//! if let Some(children) = world.get_mut::<Children>(parent) {
//!     children.push(child);
//! }
//! ```
//!
//! # FFI Safety
//!
//! All hierarchy components are `#[repr(C)]` where applicable for FFI
//! compatibility. String data uses Rust's `String` type internally but
//! provides FFI-safe accessor methods.
//!
//! # Consistency
//!
//! **Important**: The hierarchy must be kept consistent. When adding a child:
//! 1. Add `Parent` component to the child pointing to parent
//! 2. Add/update `Children` component on parent to include child
//!
//! When removing a child:
//! 1. Remove `Parent` component from child (or update to new parent)
//! 2. Remove child from parent's `Children` list
//!
//! A hierarchy maintenance system should be used to ensure consistency.

mod children;
mod name;
mod parent;

#[cfg(test)]
mod tests;

pub use children::Children;
pub use name::Name;
pub use parent::Parent;
