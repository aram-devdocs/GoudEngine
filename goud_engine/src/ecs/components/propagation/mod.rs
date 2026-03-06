//! Transform propagation functions for hierarchical transforms.
//!
//! This module provides functions to propagate transforms through entity hierarchies.
//! When entities have parent-child relationships, child transforms are relative to
//! their parents. These functions compute the world-space [`GlobalTransform`] and
//! [`GlobalTransform2D`] from local transforms by traversing the hierarchy.
//!
//! # How Propagation Works
//!
//! 1. **Root entities** (no `Parent` component): `GlobalTransform = Transform`
//! 2. **Child entities**: `GlobalTransform = Parent's GlobalTransform * Local Transform`
//!
//! The propagation must be done in the correct order:
//! - Parents must be processed before their children
//! - This is a depth-first traversal from root nodes
//!
//! # Usage
//!
//! These functions are typically called each frame to update world transforms:
//!
//! ```
//! use goud_engine::ecs::World;
//! use goud_engine::ecs::components::{Transform, GlobalTransform, Parent, Children};
//! use goud_engine::ecs::components::propagation;
//!
//! let mut world = World::new();
//!
//! // ... spawn entities with Transform, Parent, Children components ...
//!
//! // Update all global transforms
//! propagation::propagate_transforms(&mut world);
//! ```
//!
//! # Performance Considerations
//!
//! - Propagation is O(n) where n is the number of entities with transforms
//! - Uses iterative traversal with explicit stack (no recursion)
//! - Root entities are updated first, then children in depth-first order
//!
//! # System Integration
//!
//! The propagation functions can be integrated into the scheduler as systems
//! that run in `PostUpdate` stage:
//!
//! ```ignore
//! schedule.add_system_to_stage(
//!     CoreStage::PostUpdate,
//!     propagate_transforms_system,
//! );
//! ```

mod propagate2d;
mod propagate3d;
mod utils;

pub use propagate2d::{propagate_transform_2d_subtree, propagate_transforms_2d};
pub use propagate3d::{propagate_transform_subtree, propagate_transforms};
pub use utils::{
    compute_local_transform, compute_local_transform_2d, ensure_global_transforms,
    ensure_global_transforms_2d,
};

// Keep tests in a separate file to keep mod.rs lean
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
