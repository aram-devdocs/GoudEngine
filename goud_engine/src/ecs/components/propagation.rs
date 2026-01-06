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

use crate::ecs::components::hierarchy::{Children, Parent};
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::global_transform::GlobalTransform;
use super::global_transform2d::GlobalTransform2D;
use super::transform::Transform;
use super::transform2d::Transform2D;

// =============================================================================
// 3D Transform Propagation
// =============================================================================

/// Propagates transforms through the entity hierarchy (3D).
///
/// This function updates `GlobalTransform` for all entities with `Transform`:
/// - Root entities: `GlobalTransform` = `Transform` (direct copy)
/// - Child entities: `GlobalTransform` = parent's `GlobalTransform` * local `Transform`
///
/// # Requirements
///
/// For correct propagation:
/// - Parent entities must have both `Transform` and `GlobalTransform`
/// - Parent entities must have a `Children` component listing their children
/// - Child entities must have both `Transform`, `GlobalTransform`, and `Parent`
///
/// # Example
///
/// ```
/// use goud_engine::ecs::World;
/// use goud_engine::ecs::components::{Transform, GlobalTransform, Parent, Children};
/// use goud_engine::ecs::components::propagation::propagate_transforms;
/// use goud_engine::core::math::Vec3;
///
/// let mut world = World::new();
///
/// // Create parent at (10, 0, 0)
/// let parent = world.spawn_empty();
/// world.insert(parent, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
/// world.insert(parent, GlobalTransform::IDENTITY);
///
/// // Create child at local (5, 0, 0)
/// let child = world.spawn_empty();
/// world.insert(child, Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
/// world.insert(child, GlobalTransform::IDENTITY);
/// world.insert(child, Parent::new(parent));
///
/// // Set up parent's children list
/// let mut children = Children::new();
/// children.push(child);
/// world.insert(parent, children);
///
/// // Propagate transforms
/// propagate_transforms(&mut world);
///
/// // Child's global position should be (15, 0, 0)
/// if let Some(global) = world.get::<GlobalTransform>(child) {
///     let pos = global.translation();
///     assert!((pos.x - 15.0).abs() < 0.001);
/// }
/// ```
pub fn propagate_transforms(world: &mut World) {
    // Step 1: Find all root entities (have Transform but no Parent)
    let mut roots = Vec::new();
    let mut entities_with_transform = Vec::new();

    // Collect entities with Transform component
    // We need to iterate through archetypes to find entities
    // For now, use a simpler approach: track via archetype iteration
    for archetype in world.archetypes().iter() {
        for &entity in archetype.entities() {
            if world.has::<Transform>(entity) {
                entities_with_transform.push(entity);
            }
        }
    }

    // Identify roots vs children
    for entity in &entities_with_transform {
        if !world.has::<Parent>(*entity) {
            roots.push(*entity);
        }
    }

    // Step 2: Update root entities (GlobalTransform = Transform)
    for &root in &roots {
        if let Some(transform) = world.get::<Transform>(root) {
            let global = GlobalTransform::from(*transform);
            world.insert(root, global);
        }
    }

    // Step 3: Process hierarchy depth-first using explicit stack
    let mut stack: Vec<Entity> = Vec::new();

    // Push children of all roots onto stack
    for &root in &roots {
        if let Some(children) = world.get::<Children>(root) {
            // Push in reverse order so first child is processed first
            for &child in children.as_slice().iter().rev() {
                stack.push(child);
            }
        }
    }

    // Process stack
    while let Some(entity) = stack.pop() {
        // Get parent's global transform and local transform
        let parent_global = if let Some(parent_comp) = world.get::<Parent>(entity) {
            let parent_entity = parent_comp.get();
            world.get::<GlobalTransform>(parent_entity).copied()
        } else {
            None
        };

        let local = world.get::<Transform>(entity).copied();

        // Compute and update global transform
        if let (Some(parent_global), Some(local)) = (parent_global, local) {
            let global = parent_global.transform_by(&local);
            world.insert(entity, global);
        } else if let Some(local) = local {
            // No parent, treat as root
            world.insert(entity, GlobalTransform::from(local));
        }

        // Push children onto stack
        if let Some(children) = world.get::<Children>(entity) {
            for &child in children.as_slice().iter().rev() {
                stack.push(child);
            }
        }
    }
}

/// Updates GlobalTransform for a single entity and its descendants.
///
/// This is useful when you need to update a specific subtree rather than
/// the entire world.
///
/// # Arguments
///
/// * `world` - The ECS world
/// * `entity` - The root of the subtree to update
/// * `parent_global` - The parent's global transform (or None for root)
pub fn propagate_transform_subtree(
    world: &mut World,
    entity: Entity,
    parent_global: Option<&GlobalTransform>,
) {
    // Get local transform
    let local = match world.get::<Transform>(entity).copied() {
        Some(t) => t,
        None => return, // No transform, nothing to do
    };

    // Compute global transform
    let global = match parent_global {
        Some(pg) => pg.transform_by(&local),
        None => GlobalTransform::from(local),
    };

    // Update entity's global transform
    world.insert(entity, global);

    // Process children
    let children: Vec<Entity> = world
        .get::<Children>(entity)
        .map(|c| c.as_slice().to_vec())
        .unwrap_or_default();

    for child in children {
        propagate_transform_subtree(world, child, Some(&global));
    }
}

// =============================================================================
// 2D Transform Propagation
// =============================================================================

/// Propagates 2D transforms through the entity hierarchy.
///
/// This function updates `GlobalTransform2D` for all entities with `Transform2D`:
/// - Root entities: `GlobalTransform2D` = `Transform2D` (direct copy)
/// - Child entities: `GlobalTransform2D` = parent's `GlobalTransform2D` * local `Transform2D`
///
/// # Requirements
///
/// For correct propagation:
/// - Parent entities must have both `Transform2D` and `GlobalTransform2D`
/// - Parent entities must have a `Children` component listing their children
/// - Child entities must have both `Transform2D`, `GlobalTransform2D`, and `Parent`
///
/// # Example
///
/// ```
/// use goud_engine::ecs::World;
/// use goud_engine::ecs::components::{Transform2D, GlobalTransform2D, Parent, Children};
/// use goud_engine::ecs::components::propagation::propagate_transforms_2d;
/// use goud_engine::core::math::Vec2;
///
/// let mut world = World::new();
///
/// // Create parent at (100, 0)
/// let parent = world.spawn_empty();
/// world.insert(parent, Transform2D::from_position(Vec2::new(100.0, 0.0)));
/// world.insert(parent, GlobalTransform2D::IDENTITY);
///
/// // Create child at local (50, 0)
/// let child = world.spawn_empty();
/// world.insert(child, Transform2D::from_position(Vec2::new(50.0, 0.0)));
/// world.insert(child, GlobalTransform2D::IDENTITY);
/// world.insert(child, Parent::new(parent));
///
/// // Set up parent's children list
/// let mut children = Children::new();
/// children.push(child);
/// world.insert(parent, children);
///
/// // Propagate transforms
/// propagate_transforms_2d(&mut world);
///
/// // Child's global position should be (150, 0)
/// if let Some(global) = world.get::<GlobalTransform2D>(child) {
///     let pos = global.translation();
///     assert!((pos.x - 150.0).abs() < 0.001);
/// }
/// ```
pub fn propagate_transforms_2d(world: &mut World) {
    // Step 1: Find all root entities (have Transform2D but no Parent)
    let mut roots = Vec::new();
    let mut entities_with_transform = Vec::new();

    // Collect entities with Transform2D component
    for archetype in world.archetypes().iter() {
        for &entity in archetype.entities() {
            if world.has::<Transform2D>(entity) {
                entities_with_transform.push(entity);
            }
        }
    }

    // Identify roots vs children
    for entity in &entities_with_transform {
        if !world.has::<Parent>(*entity) {
            roots.push(*entity);
        }
    }

    // Step 2: Update root entities (GlobalTransform2D = Transform2D)
    for &root in &roots {
        if let Some(transform) = world.get::<Transform2D>(root) {
            let global = GlobalTransform2D::from(*transform);
            world.insert(root, global);
        }
    }

    // Step 3: Process hierarchy depth-first using explicit stack
    let mut stack: Vec<Entity> = Vec::new();

    // Push children of all roots onto stack
    for &root in &roots {
        if let Some(children) = world.get::<Children>(root) {
            for &child in children.as_slice().iter().rev() {
                stack.push(child);
            }
        }
    }

    // Process stack
    while let Some(entity) = stack.pop() {
        // Get parent's global transform and local transform
        let parent_global = if let Some(parent_comp) = world.get::<Parent>(entity) {
            let parent_entity = parent_comp.get();
            world.get::<GlobalTransform2D>(parent_entity).copied()
        } else {
            None
        };

        let local = world.get::<Transform2D>(entity).copied();

        // Compute and update global transform
        if let (Some(parent_global), Some(local)) = (parent_global, local) {
            let global = parent_global.transform_by(&local);
            world.insert(entity, global);
        } else if let Some(local) = local {
            // No parent, treat as root
            world.insert(entity, GlobalTransform2D::from(local));
        }

        // Push children onto stack
        if let Some(children) = world.get::<Children>(entity) {
            for &child in children.as_slice().iter().rev() {
                stack.push(child);
            }
        }
    }
}

/// Updates GlobalTransform2D for a single entity and its descendants.
///
/// This is useful when you need to update a specific subtree rather than
/// the entire world.
///
/// # Arguments
///
/// * `world` - The ECS world
/// * `entity` - The root of the subtree to update
/// * `parent_global` - The parent's global transform (or None for root)
pub fn propagate_transform_2d_subtree(
    world: &mut World,
    entity: Entity,
    parent_global: Option<&GlobalTransform2D>,
) {
    // Get local transform
    let local = match world.get::<Transform2D>(entity).copied() {
        Some(t) => t,
        None => return, // No transform, nothing to do
    };

    // Compute global transform
    let global = match parent_global {
        Some(pg) => pg.transform_by(&local),
        None => GlobalTransform2D::from(local),
    };

    // Update entity's global transform
    world.insert(entity, global);

    // Process children
    let children: Vec<Entity> = world
        .get::<Children>(entity)
        .map(|c| c.as_slice().to_vec())
        .unwrap_or_default();

    for child in children {
        propagate_transform_2d_subtree(world, child, Some(&global));
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Computes the local transform that would produce the given global transform
/// when combined with the parent's global transform.
///
/// This is useful when you want to set an entity's world position directly
/// but need to store it as a local transform.
///
/// # Arguments
///
/// * `desired_global` - The desired world-space transform
/// * `parent_global` - The parent's world-space transform (or None for root)
///
/// # Returns
///
/// The local transform that, when combined with the parent, produces the desired global.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::components::{Transform, GlobalTransform};
/// use goud_engine::ecs::components::propagation::compute_local_transform;
/// use goud_engine::core::math::Vec3;
///
/// // Parent at (10, 0, 0)
/// let parent_global = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
///
/// // We want child to be at world position (25, 0, 0)
/// let desired_global = GlobalTransform::from_translation(Vec3::new(25.0, 0.0, 0.0));
///
/// // Compute local transform: should be (15, 0, 0)
/// let local = compute_local_transform(&desired_global, Some(&parent_global));
/// let expected_local = local.expect("Should be computable");
///
/// assert!((expected_local.position.x - 15.0).abs() < 0.001);
/// ```
pub fn compute_local_transform(
    desired_global: &GlobalTransform,
    parent_global: Option<&GlobalTransform>,
) -> Option<Transform> {
    match parent_global {
        Some(pg) => {
            let parent_inv = pg.inverse()?;
            let local_global = parent_inv.mul_transform(desired_global);
            Some(local_global.to_transform())
        }
        None => Some(desired_global.to_transform()),
    }
}

/// Computes the local 2D transform that would produce the given global transform.
///
/// See [`compute_local_transform`] for 3D version and detailed documentation.
pub fn compute_local_transform_2d(
    desired_global: &GlobalTransform2D,
    parent_global: Option<&GlobalTransform2D>,
) -> Option<Transform2D> {
    match parent_global {
        Some(pg) => {
            let parent_inv = pg.inverse()?;
            let local_global = parent_inv.mul_transform(desired_global);
            Some(local_global.to_transform())
        }
        None => Some(desired_global.to_transform()),
    }
}

/// Ensures all entities with Transform also have GlobalTransform.
///
/// This is useful as a setup step before propagation to ensure all
/// necessary components are present.
///
/// # Arguments
///
/// * `world` - The ECS world
///
/// # Returns
///
/// The number of GlobalTransform components added.
pub fn ensure_global_transforms(world: &mut World) -> usize {
    let mut count = 0;
    let mut entities_needing_global: Vec<Entity> = Vec::new();

    // Find entities with Transform but no GlobalTransform
    for archetype in world.archetypes().iter() {
        for &entity in archetype.entities() {
            if world.has::<Transform>(entity) && !world.has::<GlobalTransform>(entity) {
                entities_needing_global.push(entity);
            }
        }
    }

    // Add GlobalTransform to entities that need it
    for entity in entities_needing_global {
        world.insert(entity, GlobalTransform::IDENTITY);
        count += 1;
    }

    count
}

/// Ensures all entities with Transform2D also have GlobalTransform2D.
///
/// See [`ensure_global_transforms`] for 3D version and detailed documentation.
pub fn ensure_global_transforms_2d(world: &mut World) -> usize {
    let mut count = 0;
    let mut entities_needing_global: Vec<Entity> = Vec::new();

    // Find entities with Transform2D but no GlobalTransform2D
    for archetype in world.archetypes().iter() {
        for &entity in archetype.entities() {
            if world.has::<Transform2D>(entity) && !world.has::<GlobalTransform2D>(entity) {
                entities_needing_global.push(entity);
            }
        }
    }

    // Add GlobalTransform2D to entities that need it
    for entity in entities_needing_global {
        world.insert(entity, GlobalTransform2D::IDENTITY);
        count += 1;
    }

    count
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::{Vec2, Vec3};
    use crate::ecs::components::transform::Quat;
    use std::f32::consts::FRAC_PI_2;

    mod propagate_3d_tests {
        use super::*;

        #[test]
        fn test_root_entity_propagation() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            world.insert(entity, Transform::from_position(Vec3::new(10.0, 5.0, 3.0)));
            world.insert(entity, GlobalTransform::IDENTITY);

            propagate_transforms(&mut world);

            let global = world.get::<GlobalTransform>(entity).unwrap();
            let pos = global.translation();
            assert!((pos.x - 10.0).abs() < 0.001);
            assert!((pos.y - 5.0).abs() < 0.001);
            assert!((pos.z - 3.0).abs() < 0.001);
        }

        #[test]
        fn test_parent_child_propagation() {
            let mut world = World::new();

            // Parent at (10, 0, 0)
            let parent = world.spawn_empty();
            world.insert(parent, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
            world.insert(parent, GlobalTransform::IDENTITY);

            // Child at local (5, 0, 0)
            let child = world.spawn_empty();
            world.insert(child, Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
            world.insert(child, GlobalTransform::IDENTITY);
            world.insert(child, Parent::new(parent));

            // Set up parent's children
            let mut children = Children::new();
            children.push(child);
            world.insert(parent, children);

            propagate_transforms(&mut world);

            let child_global = world.get::<GlobalTransform>(child).unwrap();
            let pos = child_global.translation();
            assert!((pos.x - 15.0).abs() < 0.001);
        }

        #[test]
        fn test_rotation_propagation() {
            let mut world = World::new();

            // Parent rotated 90 degrees around Y
            let parent = world.spawn_empty();
            let parent_transform = Transform::from_rotation(Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2));
            world.insert(parent, parent_transform);
            world.insert(parent, GlobalTransform::IDENTITY);

            // Child at local (0, 0, -10) - in front of parent
            let child = world.spawn_empty();
            world.insert(child, Transform::from_position(Vec3::new(0.0, 0.0, -10.0)));
            world.insert(child, GlobalTransform::IDENTITY);
            world.insert(child, Parent::new(parent));

            let mut children = Children::new();
            children.push(child);
            world.insert(parent, children);

            propagate_transforms(&mut world);

            // After parent's 90-degree Y rotation, child at (0,0,-10) should be at (-10, 0, 0)
            let child_global = world.get::<GlobalTransform>(child).unwrap();
            let pos = child_global.translation();
            assert!((pos.x - (-10.0)).abs() < 0.1);
            assert!(pos.y.abs() < 0.1);
            assert!(pos.z.abs() < 0.1);
        }

        #[test]
        fn test_scale_propagation() {
            let mut world = World::new();

            // Parent scaled 2x
            let parent = world.spawn_empty();
            world.insert(parent, Transform::from_scale(Vec3::new(2.0, 2.0, 2.0)));
            world.insert(parent, GlobalTransform::IDENTITY);

            // Child at local (5, 0, 0)
            let child = world.spawn_empty();
            world.insert(child, Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
            world.insert(child, GlobalTransform::IDENTITY);
            world.insert(child, Parent::new(parent));

            let mut children = Children::new();
            children.push(child);
            world.insert(parent, children);

            propagate_transforms(&mut world);

            // Child's position should be (10, 0, 0) due to parent scale
            let child_global = world.get::<GlobalTransform>(child).unwrap();
            let pos = child_global.translation();
            assert!((pos.x - 10.0).abs() < 0.001);
        }

        #[test]
        fn test_three_level_hierarchy() {
            let mut world = World::new();

            // Grandparent at (10, 0, 0)
            let grandparent = world.spawn_empty();
            world.insert(grandparent, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
            world.insert(grandparent, GlobalTransform::IDENTITY);

            // Parent at local (5, 0, 0)
            let parent = world.spawn_empty();
            world.insert(parent, Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
            world.insert(parent, GlobalTransform::IDENTITY);
            world.insert(parent, Parent::new(grandparent));

            // Child at local (3, 0, 0)
            let child = world.spawn_empty();
            world.insert(child, Transform::from_position(Vec3::new(3.0, 0.0, 0.0)));
            world.insert(child, GlobalTransform::IDENTITY);
            world.insert(child, Parent::new(parent));

            // Set up children lists
            let mut gp_children = Children::new();
            gp_children.push(parent);
            world.insert(grandparent, gp_children);

            let mut p_children = Children::new();
            p_children.push(child);
            world.insert(parent, p_children);

            propagate_transforms(&mut world);

            // Child's global should be (18, 0, 0)
            let child_global = world.get::<GlobalTransform>(child).unwrap();
            let pos = child_global.translation();
            assert!((pos.x - 18.0).abs() < 0.001);
        }

        #[test]
        fn test_multiple_children() {
            let mut world = World::new();

            // Parent at (10, 0, 0)
            let parent = world.spawn_empty();
            world.insert(parent, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
            world.insert(parent, GlobalTransform::IDENTITY);

            // Child 1 at local (1, 0, 0)
            let child1 = world.spawn_empty();
            world.insert(child1, Transform::from_position(Vec3::new(1.0, 0.0, 0.0)));
            world.insert(child1, GlobalTransform::IDENTITY);
            world.insert(child1, Parent::new(parent));

            // Child 2 at local (2, 0, 0)
            let child2 = world.spawn_empty();
            world.insert(child2, Transform::from_position(Vec3::new(2.0, 0.0, 0.0)));
            world.insert(child2, GlobalTransform::IDENTITY);
            world.insert(child2, Parent::new(parent));

            let mut children = Children::new();
            children.push(child1);
            children.push(child2);
            world.insert(parent, children);

            propagate_transforms(&mut world);

            let child1_global = world.get::<GlobalTransform>(child1).unwrap();
            assert!((child1_global.translation().x - 11.0).abs() < 0.001);

            let child2_global = world.get::<GlobalTransform>(child2).unwrap();
            assert!((child2_global.translation().x - 12.0).abs() < 0.001);
        }

        #[test]
        fn test_multiple_roots() {
            let mut world = World::new();

            // Root 1 at (10, 0, 0)
            let root1 = world.spawn_empty();
            world.insert(root1, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
            world.insert(root1, GlobalTransform::IDENTITY);

            // Root 2 at (20, 0, 0)
            let root2 = world.spawn_empty();
            world.insert(root2, Transform::from_position(Vec3::new(20.0, 0.0, 0.0)));
            world.insert(root2, GlobalTransform::IDENTITY);

            propagate_transforms(&mut world);

            let root1_global = world.get::<GlobalTransform>(root1).unwrap();
            assert!((root1_global.translation().x - 10.0).abs() < 0.001);

            let root2_global = world.get::<GlobalTransform>(root2).unwrap();
            assert!((root2_global.translation().x - 20.0).abs() < 0.001);
        }
    }

    mod propagate_2d_tests {
        use super::*;

        #[test]
        fn test_root_entity_propagation_2d() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            world.insert(entity, Transform2D::from_position(Vec2::new(100.0, 50.0)));
            world.insert(entity, GlobalTransform2D::IDENTITY);

            propagate_transforms_2d(&mut world);

            let global = world.get::<GlobalTransform2D>(entity).unwrap();
            let pos = global.translation();
            assert!((pos.x - 100.0).abs() < 0.001);
            assert!((pos.y - 50.0).abs() < 0.001);
        }

        #[test]
        fn test_parent_child_propagation_2d() {
            let mut world = World::new();

            // Parent at (100, 0)
            let parent = world.spawn_empty();
            world.insert(parent, Transform2D::from_position(Vec2::new(100.0, 0.0)));
            world.insert(parent, GlobalTransform2D::IDENTITY);

            // Child at local (50, 0)
            let child = world.spawn_empty();
            world.insert(child, Transform2D::from_position(Vec2::new(50.0, 0.0)));
            world.insert(child, GlobalTransform2D::IDENTITY);
            world.insert(child, Parent::new(parent));

            let mut children = Children::new();
            children.push(child);
            world.insert(parent, children);

            propagate_transforms_2d(&mut world);

            let child_global = world.get::<GlobalTransform2D>(child).unwrap();
            let pos = child_global.translation();
            assert!((pos.x - 150.0).abs() < 0.001);
        }

        #[test]
        fn test_rotation_propagation_2d() {
            let mut world = World::new();

            // Parent rotated 90 degrees
            let parent = world.spawn_empty();
            world.insert(parent, Transform2D::from_rotation(FRAC_PI_2));
            world.insert(parent, GlobalTransform2D::IDENTITY);

            // Child at local (0, 100) - above parent
            let child = world.spawn_empty();
            world.insert(child, Transform2D::from_position(Vec2::new(0.0, 100.0)));
            world.insert(child, GlobalTransform2D::IDENTITY);
            world.insert(child, Parent::new(parent));

            let mut children = Children::new();
            children.push(child);
            world.insert(parent, children);

            propagate_transforms_2d(&mut world);

            // After 90-degree rotation, child at (0, 100) should be at (-100, 0)
            let child_global = world.get::<GlobalTransform2D>(child).unwrap();
            let pos = child_global.translation();
            assert!((pos.x - (-100.0)).abs() < 0.1);
            assert!(pos.y.abs() < 0.1);
        }

        #[test]
        fn test_scale_propagation_2d() {
            let mut world = World::new();

            // Parent scaled 2x
            let parent = world.spawn_empty();
            world.insert(parent, Transform2D::from_scale(Vec2::new(2.0, 2.0)));
            world.insert(parent, GlobalTransform2D::IDENTITY);

            // Child at local (50, 0)
            let child = world.spawn_empty();
            world.insert(child, Transform2D::from_position(Vec2::new(50.0, 0.0)));
            world.insert(child, GlobalTransform2D::IDENTITY);
            world.insert(child, Parent::new(parent));

            let mut children = Children::new();
            children.push(child);
            world.insert(parent, children);

            propagate_transforms_2d(&mut world);

            // Child's position should be (100, 0) due to parent scale
            let child_global = world.get::<GlobalTransform2D>(child).unwrap();
            let pos = child_global.translation();
            assert!((pos.x - 100.0).abs() < 0.001);
        }
    }

    mod utility_tests {
        use super::*;

        #[test]
        fn test_compute_local_transform() {
            let parent_global = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
            let desired_global = GlobalTransform::from_translation(Vec3::new(25.0, 0.0, 0.0));

            let local = compute_local_transform(&desired_global, Some(&parent_global)).unwrap();
            assert!((local.position.x - 15.0).abs() < 0.001);
        }

        #[test]
        fn test_compute_local_transform_no_parent() {
            let desired_global = GlobalTransform::from_translation(Vec3::new(25.0, 0.0, 0.0));

            let local = compute_local_transform(&desired_global, None).unwrap();
            assert!((local.position.x - 25.0).abs() < 0.001);
        }

        #[test]
        fn test_compute_local_transform_2d() {
            let parent_global = GlobalTransform2D::from_translation(Vec2::new(100.0, 0.0));
            let desired_global = GlobalTransform2D::from_translation(Vec2::new(250.0, 0.0));

            let local = compute_local_transform_2d(&desired_global, Some(&parent_global)).unwrap();
            assert!((local.position.x - 150.0).abs() < 0.001);
        }

        #[test]
        fn test_ensure_global_transforms() {
            let mut world = World::new();

            // Entity with Transform but no GlobalTransform
            let entity = world.spawn_empty();
            world.insert(entity, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));

            // Should not have GlobalTransform yet
            assert!(!world.has::<GlobalTransform>(entity));

            let count = ensure_global_transforms(&mut world);

            assert_eq!(count, 1);
            assert!(world.has::<GlobalTransform>(entity));
        }

        #[test]
        fn test_ensure_global_transforms_2d() {
            let mut world = World::new();

            // Entity with Transform2D but no GlobalTransform2D
            let entity = world.spawn_empty();
            world.insert(entity, Transform2D::from_position(Vec2::new(100.0, 0.0)));

            assert!(!world.has::<GlobalTransform2D>(entity));

            let count = ensure_global_transforms_2d(&mut world);

            assert_eq!(count, 1);
            assert!(world.has::<GlobalTransform2D>(entity));
        }

        #[test]
        fn test_ensure_skips_existing() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            world.insert(entity, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
            world.insert(entity, GlobalTransform::IDENTITY);

            let count = ensure_global_transforms(&mut world);
            assert_eq!(count, 0);
        }
    }

    mod subtree_tests {
        use super::*;

        #[test]
        fn test_propagate_subtree() {
            let mut world = World::new();

            // Root not in hierarchy
            let root = world.spawn_empty();
            world.insert(root, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
            world.insert(root, GlobalTransform::IDENTITY);

            // Child
            let child = world.spawn_empty();
            world.insert(child, Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
            world.insert(child, GlobalTransform::IDENTITY);

            let mut children = Children::new();
            children.push(child);
            world.insert(root, children);

            // Propagate subtree from root
            propagate_transform_subtree(&mut world, root, None);

            // Root should be updated
            let root_global = world.get::<GlobalTransform>(root).unwrap();
            assert!((root_global.translation().x - 10.0).abs() < 0.001);

            // Child should be updated relative to root
            let child_global = world.get::<GlobalTransform>(child).unwrap();
            assert!((child_global.translation().x - 15.0).abs() < 0.001);
        }

        #[test]
        fn test_propagate_subtree_with_parent_global() {
            let mut world = World::new();

            let root = world.spawn_empty();
            world.insert(root, Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
            world.insert(root, GlobalTransform::IDENTITY);

            let parent_global = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));

            propagate_transform_subtree(&mut world, root, Some(&parent_global));

            let root_global = world.get::<GlobalTransform>(root).unwrap();
            assert!((root_global.translation().x - 15.0).abs() < 0.001);
        }
    }
}
