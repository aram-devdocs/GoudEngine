use super::*;
use crate::ecs::components::hierarchy::{Children, Parent};

mod despawn_recursive {
    use super::*;

    #[test]
    fn test_despawn_recursive_no_children() {
        let mut world = World::new();
        let entity = world.spawn().id();
        assert!(world.despawn_recursive(entity));
        assert!(!world.is_alive(entity));
    }

    #[test]
    fn test_despawn_recursive_single_child() {
        let mut world = World::new();
        let parent = world.spawn().id();
        let child = world.spawn().id();
        world.insert(parent, Children::from_slice(&[child]));
        world.insert(child, Parent::new(parent));

        assert!(world.despawn_recursive(parent));
        assert!(!world.is_alive(parent));
        assert!(!world.is_alive(child));
    }

    #[test]
    fn test_despawn_recursive_multiple_children() {
        let mut world = World::new();
        let parent = world.spawn().id();
        let c1 = world.spawn().id();
        let c2 = world.spawn().id();
        let c3 = world.spawn().id();
        world.insert(parent, Children::from_slice(&[c1, c2, c3]));
        world.insert(c1, Parent::new(parent));
        world.insert(c2, Parent::new(parent));
        world.insert(c3, Parent::new(parent));

        assert!(world.despawn_recursive(parent));
        assert!(!world.is_alive(parent));
        assert!(!world.is_alive(c1));
        assert!(!world.is_alive(c2));
        assert!(!world.is_alive(c3));
    }

    #[test]
    fn test_despawn_recursive_grandchildren() {
        let mut world = World::new();
        let a = world.spawn().id();
        let b = world.spawn().id();
        let c = world.spawn().id();
        world.insert(a, Children::from_slice(&[b]));
        world.insert(b, Parent::new(a));
        world.insert(b, Children::from_slice(&[c]));
        world.insert(c, Parent::new(b));

        assert!(world.despawn_recursive(a));
        assert!(!world.is_alive(a));
        assert!(!world.is_alive(b));
        assert!(!world.is_alive(c));
    }

    #[test]
    fn test_despawn_recursive_deep_hierarchy() {
        let mut world = World::new();
        let mut entities = Vec::new();
        for _ in 0..5 {
            entities.push(world.spawn().id());
        }
        // Chain: 0 -> 1 -> 2 -> 3 -> 4
        for i in 0..4 {
            world.insert(entities[i], Children::from_slice(&[entities[i + 1]]));
            world.insert(entities[i + 1], Parent::new(entities[i]));
        }

        assert!(world.despawn_recursive(entities[0]));
        for e in &entities {
            assert!(!world.is_alive(*e));
        }
    }

    #[test]
    fn test_despawn_recursive_preserves_siblings() {
        let mut world = World::new();
        let root = world.spawn().id();
        let branch_a = world.spawn().id();
        let branch_b = world.spawn().id();
        let leaf_a = world.spawn().id();
        let leaf_b = world.spawn().id();

        world.insert(root, Children::from_slice(&[branch_a, branch_b]));
        world.insert(branch_a, Parent::new(root));
        world.insert(branch_b, Parent::new(root));
        world.insert(branch_a, Children::from_slice(&[leaf_a]));
        world.insert(leaf_a, Parent::new(branch_a));
        world.insert(branch_b, Children::from_slice(&[leaf_b]));
        world.insert(leaf_b, Parent::new(branch_b));

        // Despawn only branch_a subtree
        assert!(world.despawn_recursive(branch_a));
        assert!(!world.is_alive(branch_a));
        assert!(!world.is_alive(leaf_a));
        // branch_b subtree should still be alive
        assert!(world.is_alive(root));
        assert!(world.is_alive(branch_b));
        assert!(world.is_alive(leaf_b));
    }

    #[test]
    fn test_despawn_recursive_dead_entity() {
        let mut world = World::new();
        let entity = world.spawn().id();
        world.despawn(entity);
        assert!(!world.despawn_recursive(entity));
    }

    #[test]
    fn test_despawn_recursive_cleans_all_components() {
        let mut world = World::new();
        let parent = world.spawn().id();
        let child = world.spawn().id();
        world.insert(parent, Position { x: 1.0, y: 2.0 });
        world.insert(parent, Children::from_slice(&[child]));
        world.insert(child, Parent::new(parent));
        world.insert(child, Position { x: 3.0, y: 4.0 });

        assert!(world.despawn_recursive(parent));
        assert!(!world.is_alive(parent));
        assert!(!world.is_alive(child));
        // Verify components are gone
        assert!(world.get::<Position>(parent).is_none());
        assert!(world.get::<Position>(child).is_none());
        assert!(world.get::<Children>(parent).is_none());
        assert!(world.get::<Parent>(child).is_none());
    }
}
