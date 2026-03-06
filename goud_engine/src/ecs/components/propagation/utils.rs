//! Utility functions for transform propagation.

use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::super::global_transform::GlobalTransform;
use super::super::global_transform2d::GlobalTransform2D;
use super::super::transform::Transform;
use super::super::transform2d::Transform2D;

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
