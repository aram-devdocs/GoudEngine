//! Tests for transform propagation functions.

use crate::core::math::{Vec2, Vec3};
use crate::ecs::components::hierarchy::{Children, Parent};
use crate::ecs::components::propagation::{
    compute_local_transform, compute_local_transform_2d, ensure_global_transforms,
    ensure_global_transforms_2d, propagate_transform_subtree, propagate_transforms,
    propagate_transforms_2d,
};
use crate::ecs::components::transform::Quat;
use crate::ecs::components::{GlobalTransform, GlobalTransform2D, Transform, Transform2D};
use crate::ecs::World;
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
        let parent_transform =
            Transform::from_rotation(Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2));
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
        world.insert(
            grandparent,
            Transform::from_position(Vec3::new(10.0, 0.0, 0.0)),
        );
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
