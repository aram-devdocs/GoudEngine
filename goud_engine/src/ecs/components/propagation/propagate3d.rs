//! 3D transform propagation through entity hierarchies.

use crate::ecs::components::hierarchy::{Children, Parent};
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::super::global_transform::GlobalTransform;
use super::super::transform::Transform;

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
