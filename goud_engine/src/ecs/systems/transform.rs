//! Transform propagation system.
//!
//! This module provides systems for propagating transform changes through entity hierarchies.
//! It handles both 2D and 3D transform propagation in a single system.
//!
//! # Example
//!
//! ```rust,ignore
//! use goud_engine::ecs::systems::TransformPropagationSystem;
//! use goud_engine::ecs::system::System;
//! use goud_engine::ecs::World;
//!
//! let mut system = TransformPropagationSystem::new();
//! system.run(&mut world);
//! ```

use crate::ecs::component::ComponentId;
use crate::ecs::components::global_transform::GlobalTransform;
use crate::ecs::components::global_transform2d::GlobalTransform2D;
use crate::ecs::components::hierarchy::{Children, Parent};
use crate::ecs::components::propagation::{propagate_transforms, propagate_transforms_2d};
use crate::ecs::components::transform::Transform;
use crate::ecs::components::transform2d::Transform2D;
use crate::ecs::query::Access;
use crate::ecs::system::System;
use crate::ecs::World;

/// System for propagating transform changes through entity hierarchies.
///
/// This system:
/// - Runs 2D transform propagation (Transform2D -> GlobalTransform2D)
/// - Runs 3D transform propagation (Transform -> GlobalTransform)
/// - Traverses parent-child relationships to compute world-space transforms
///
/// Both 2D and 3D propagation run independently in each execution.
#[derive(Debug, Default, Clone)]
pub struct TransformPropagationSystem {
    _marker: std::marker::PhantomData<()>,
}

impl TransformPropagationSystem {
    /// Creates a new transform propagation system.
    pub fn new() -> Self {
        Self::default()
    }
}

impl System for TransformPropagationSystem {
    fn name(&self) -> &'static str {
        "TransformPropagationSystem"
    }

    fn component_access(&self) -> Access {
        let mut access = Access::new();

        // Reads: local transforms and hierarchy
        access.add_read(ComponentId::of::<Transform>());
        access.add_read(ComponentId::of::<Transform2D>());
        access.add_read(ComponentId::of::<Parent>());
        access.add_read(ComponentId::of::<Children>());

        // Writes: global transforms
        access.add_write(ComponentId::of::<GlobalTransform>());
        access.add_write(ComponentId::of::<GlobalTransform2D>());

        access
    }

    fn run(&mut self, world: &mut World) {
        propagate_transforms_2d(world);
        propagate_transforms(world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::{Vec2, Vec3};

    /// Helper: set up a 3D entity with Transform + GlobalTransform.
    fn spawn_3d(world: &mut World, pos: Vec3) -> crate::ecs::entity::Entity {
        let entity = world.spawn_empty();
        world.insert(entity, Transform::from_position(pos));
        world.insert(entity, GlobalTransform::IDENTITY);
        entity
    }

    /// Helper: set up a 2D entity with Transform2D + GlobalTransform2D.
    fn spawn_2d(world: &mut World, pos: Vec2) -> crate::ecs::entity::Entity {
        let entity = world.spawn_empty();
        world.insert(entity, Transform2D::from_position(pos));
        world.insert(entity, GlobalTransform2D::IDENTITY);
        entity
    }

    /// Helper: establish parent-child relationship.
    fn set_parent(
        world: &mut World,
        parent: crate::ecs::entity::Entity,
        child: crate::ecs::entity::Entity,
    ) {
        world.insert(child, Parent::new(parent));
        if let Some(existing) = world.get::<Children>(parent) {
            let mut children = existing.clone();
            children.push(child);
            world.insert(parent, children);
        } else {
            let mut children = Children::new();
            children.push(child);
            world.insert(parent, children);
        }
    }

    #[test]
    fn test_system_name() {
        let system = TransformPropagationSystem::new();
        assert_eq!(system.name(), "TransformPropagationSystem");
    }

    #[test]
    fn test_component_access_declares_reads() {
        let system = TransformPropagationSystem::new();
        let access = system.component_access();

        // Verify we're not read-only (we write global transforms)
        assert!(
            !access.is_read_only(),
            "System writes GlobalTransform/GlobalTransform2D, should not be read-only"
        );
    }

    #[test]
    fn test_component_access_is_not_empty() {
        let system = TransformPropagationSystem::new();
        let access = system.component_access();
        assert!(
            !access.is_empty(),
            "System should declare non-empty component access"
        );
    }

    #[test]
    fn test_run_on_empty_world() {
        let mut world = World::new();
        let mut system = TransformPropagationSystem::new();
        // Should not panic
        system.run(&mut world);
    }

    #[test]
    fn test_3d_propagation_multi_level_hierarchy() {
        let mut world = World::new();

        // grandparent at (10, 0, 0)
        let grandparent = spawn_3d(&mut world, Vec3::new(10.0, 0.0, 0.0));

        // parent at local (5, 0, 0) -> global (15, 0, 0)
        let parent = spawn_3d(&mut world, Vec3::new(5.0, 0.0, 0.0));
        set_parent(&mut world, grandparent, parent);

        // child at local (3, 0, 0) -> global (18, 0, 0)
        let child = spawn_3d(&mut world, Vec3::new(3.0, 0.0, 0.0));
        set_parent(&mut world, parent, child);

        let mut system = TransformPropagationSystem::new();
        system.run(&mut world);

        // Verify grandparent global = local
        let gp_global = world.get::<GlobalTransform>(grandparent).unwrap();
        assert!(
            (gp_global.translation().x - 10.0).abs() < 0.001,
            "Grandparent global x should be 10.0, got {}",
            gp_global.translation().x
        );

        // Verify parent global = grandparent + local
        let p_global = world.get::<GlobalTransform>(parent).unwrap();
        assert!(
            (p_global.translation().x - 15.0).abs() < 0.001,
            "Parent global x should be 15.0, got {}",
            p_global.translation().x
        );

        // Verify child global = grandparent + parent + local
        let c_global = world.get::<GlobalTransform>(child).unwrap();
        assert!(
            (c_global.translation().x - 18.0).abs() < 0.001,
            "Child global x should be 18.0, got {}",
            c_global.translation().x
        );
    }

    #[test]
    fn test_2d_propagation_parent_child() {
        let mut world = World::new();

        // parent at (100, 50)
        let parent = spawn_2d(&mut world, Vec2::new(100.0, 50.0));

        // child at local (20, 10) -> global (120, 60)
        let child = spawn_2d(&mut world, Vec2::new(20.0, 10.0));
        set_parent(&mut world, parent, child);

        let mut system = TransformPropagationSystem::new();
        system.run(&mut world);

        let parent_global = world.get::<GlobalTransform2D>(parent).unwrap();
        assert!((parent_global.translation().x - 100.0).abs() < 0.001);
        assert!((parent_global.translation().y - 50.0).abs() < 0.001);

        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!(
            (child_global.translation().x - 120.0).abs() < 0.001,
            "Child 2D global x should be 120.0, got {}",
            child_global.translation().x
        );
        assert!(
            (child_global.translation().y - 60.0).abs() < 0.001,
            "Child 2D global y should be 60.0, got {}",
            child_global.translation().y
        );
    }

    #[test]
    fn test_both_2d_and_3d_propagate_in_same_run() {
        let mut world = World::new();

        // 2D hierarchy
        let parent_2d = spawn_2d(&mut world, Vec2::new(10.0, 0.0));
        let child_2d = spawn_2d(&mut world, Vec2::new(5.0, 0.0));
        set_parent(&mut world, parent_2d, child_2d);

        // 3D hierarchy
        let parent_3d = spawn_3d(&mut world, Vec3::new(20.0, 0.0, 0.0));
        let child_3d = spawn_3d(&mut world, Vec3::new(7.0, 0.0, 0.0));
        set_parent(&mut world, parent_3d, child_3d);

        let mut system = TransformPropagationSystem::new();
        system.run(&mut world);

        // 2D child should be (15, 0)
        let g2d = world.get::<GlobalTransform2D>(child_2d).unwrap();
        assert!((g2d.translation().x - 15.0).abs() < 0.001);

        // 3D child should be (27, 0, 0)
        let g3d = world.get::<GlobalTransform>(child_3d).unwrap();
        assert!((g3d.translation().x - 27.0).abs() < 0.001);
    }

    #[test]
    fn test_should_run_returns_true() {
        let system = TransformPropagationSystem::new();
        let world = World::new();
        assert!(system.should_run(&world));
    }

    #[test]
    fn test_system_is_not_read_only() {
        let system = TransformPropagationSystem::new();
        assert!(
            !system.is_read_only(),
            "TransformPropagationSystem writes global transforms"
        );
    }

    #[test]
    fn test_3d_root_entity_global_equals_local() {
        let mut world = World::new();
        let root = spawn_3d(&mut world, Vec3::new(42.0, 13.0, 7.0));

        let mut system = TransformPropagationSystem::new();
        system.run(&mut world);

        let global = world.get::<GlobalTransform>(root).unwrap();
        assert!((global.translation().x - 42.0).abs() < 0.001);
        assert!((global.translation().y - 13.0).abs() < 0.001);
        assert!((global.translation().z - 7.0).abs() < 0.001);
    }
}
