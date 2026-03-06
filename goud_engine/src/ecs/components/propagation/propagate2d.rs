//! 2D transform propagation through entity hierarchies.

use crate::ecs::components::hierarchy::{Children, Parent};
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::super::global_transform2d::GlobalTransform2D;
use super::super::transform2d::Transform2D;

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
