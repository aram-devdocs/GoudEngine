//! Built-in ECS components for GoudEngine.
//!
//! This module provides commonly used components that are part of the engine's
//! core functionality. These components handle spatial transformations, hierarchy
//! relationships, and other fundamental game object properties.
//!
//! # Available Components
//!
//! ## Spatial Transformations
//!
//! - [`Transform`]: 3D local-space transformation (position, rotation, scale)
//! - [`Transform2D`]: 2D local-space transformation (position, rotation angle, scale)
//! - [`GlobalTransform`]: 3D world-space transformation cache (computed from hierarchy)
//! - [`GlobalTransform2D`]: 2D world-space transformation cache (computed from hierarchy)
//!
//! ## Rendering Components
//!
//! - [`Sprite`]: 2D sprite component for rendering textured quads
//!
//! ## Audio Components
//!
//! - [`AudioSource`]: Audio playback component with spatial audio support
//! - [`AudioChannel`]: Audio channel enumeration for mixing and grouping
//! - [`AttenuationModel`]: Distance-based volume falloff models
//!
//! ## Physics Components
//!
//! - [`RigidBody`]: Physics body component (dynamic, kinematic, or static)
//! - [`Collider`]: Collision shape component (circle, box, capsule, polygon)
//! - [`ColliderShape`]: Geometric shape types for colliders
//!
//! ## Hierarchy Components
//!
//! - [`Parent`]: Points to the parent entity (stored on child entities)
//! - [`Children`]: Lists all child entities (stored on parent entities)
//! - [`Name`]: Human-readable name for debugging and editor integration
//!
//! ## Transform Propagation
//!
//! The [`propagation`] module provides functions to compute world-space transforms
//! from local transforms in a hierarchy:
//!
//! - [`propagation::propagate_transforms`]: Update 3D GlobalTransforms
//! - [`propagation::propagate_transforms_2d`]: Update 2D GlobalTransform2Ds
//!
//! # 2D vs 3D Transforms
//!
//! For 2D games, use [`Transform2D`] and [`GlobalTransform2D`] which are more
//! memory efficient (20/36 bytes vs 40/64) and simpler to work with (rotation
//! is a single angle instead of a quaternion).
//!
//! For 3D games or when you need full 3D rotation, use [`Transform`] and
//! [`GlobalTransform`].
//!
//! # Local vs Global Transforms
//!
//! - **Local Transform** (`Transform`/`Transform2D`): Position, rotation, scale
//!   relative to the parent entity. Modify this to move/rotate/scale entities.
//!
//! - **Global Transform** (`GlobalTransform`/`GlobalTransform2D`): Computed
//!   world-space transformation. Read this for rendering, physics, etc.
//!   Never modify directly - use local transform instead.
//!
//! # Hierarchy System
//!
//! The hierarchy system uses a bidirectional pointer approach for efficient traversal:
//!
//! - Children point up to their parent via [`Parent`]
//! - Parents point down to their children via [`Children`]
//! - [`Name`] provides human-readable identification
//!
//! **Important**: When modifying hierarchies, maintain consistency:
//! 1. Add `Parent` component to child pointing to parent
//! 2. Add/update `Children` component on parent to include child
//!
//! # Examples
//!
//! ## 3D Transform with Hierarchy
//!
//! ```
//! use goud_engine::ecs::{World, Entity};
//! use goud_engine::ecs::components::{Transform, GlobalTransform, Parent, Children};
//! use goud_engine::ecs::components::propagation::propagate_transforms;
//! use goud_engine::core::math::Vec3;
//!
//! let mut world = World::new();
//!
//! // Create parent at (10, 0, 0)
//! let parent = world.spawn_empty();
//! world.insert(parent, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
//! world.insert(parent, GlobalTransform::IDENTITY);
//!
//! // Create child at local (5, 0, 0)
//! let child = world.spawn_empty();
//! world.insert(child, Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
//! world.insert(child, GlobalTransform::IDENTITY);
//! world.insert(child, Parent::new(parent));
//!
//! // Set up parent's children list
//! let mut children = Children::new();
//! children.push(child);
//! world.insert(parent, children);
//!
//! // Propagate transforms
//! propagate_transforms(&mut world);
//!
//! // Child's global position is now (15, 0, 0)
//! if let Some(global) = world.get::<GlobalTransform>(child) {
//!     let pos = global.translation();
//!     assert!((pos.x - 15.0).abs() < 0.001);
//! }
//! ```
//!
//! ## 2D Transform with Hierarchy
//!
//! ```
//! use goud_engine::ecs::{World, Entity};
//! use goud_engine::ecs::components::{Transform2D, GlobalTransform2D, Parent, Children};
//! use goud_engine::ecs::components::propagation::propagate_transforms_2d;
//! use goud_engine::core::math::Vec2;
//!
//! let mut world = World::new();
//!
//! // Create parent at (100, 0)
//! let parent = world.spawn_empty();
//! world.insert(parent, Transform2D::from_position(Vec2::new(100.0, 0.0)));
//! world.insert(parent, GlobalTransform2D::IDENTITY);
//!
//! // Create child at local (50, 0)
//! let child = world.spawn_empty();
//! world.insert(child, Transform2D::from_position(Vec2::new(50.0, 0.0)));
//! world.insert(child, GlobalTransform2D::IDENTITY);
//! world.insert(child, Parent::new(parent));
//!
//! // Set up parent's children list
//! let mut children = Children::new();
//! children.push(child);
//! world.insert(parent, children);
//!
//! // Propagate transforms
//! propagate_transforms_2d(&mut world);
//!
//! // Child's global position is now (150, 0)
//! if let Some(global) = world.get::<GlobalTransform2D>(child) {
//!     let pos = global.translation();
//!     assert!((pos.x - 150.0).abs() < 0.001);
//! }
//! ```

pub mod audiosource;
pub mod collider;
pub mod global_transform;
pub mod global_transform2d;
pub mod hierarchy;
pub mod propagation;
pub mod rigidbody;
pub mod sprite;
pub mod transform;
pub mod transform2d;

pub use audiosource::{AttenuationModel, AudioChannel, AudioSource};
pub use collider::{Collider, ColliderShape};
pub use global_transform::GlobalTransform;
pub use global_transform2d::GlobalTransform2D;
pub use hierarchy::{Children, Name, Parent};
pub use rigidbody::{RigidBody, RigidBodyType};
pub use sprite::Sprite;
pub use transform::Transform;
pub use transform2d::{Mat3x3, Transform2D};
