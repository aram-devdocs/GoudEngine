//! Tests for the [`Children`] component.

use crate::ecs::component::Component;
use crate::ecs::components::hierarchy::Children;
use crate::ecs::entity::Entity;

#[test]
fn test_children_new() {
    let children = Children::new();
    assert!(children.is_empty());
    assert_eq!(children.len(), 0);
}

#[test]
fn test_children_with_capacity() {
    let children = Children::with_capacity(100);
    assert!(children.is_empty());
}

#[test]
fn test_children_from_slice() {
    let entities = vec![Entity::new(1, 1), Entity::new(2, 1)];
    let children = Children::from_slice(&entities);
    assert_eq!(children.len(), 2);
}

#[test]
fn test_children_push() {
    let mut children = Children::new();
    children.push(Entity::new(1, 1));
    children.push(Entity::new(2, 1));
    assert_eq!(children.len(), 2);
    assert_eq!(children.get(0), Some(Entity::new(1, 1)));
    assert_eq!(children.get(1), Some(Entity::new(2, 1)));
}

#[test]
fn test_children_insert() {
    let mut children = Children::new();
    children.push(Entity::new(1, 1));
    children.push(Entity::new(3, 1));
    children.insert(1, Entity::new(2, 1));
    assert_eq!(children.get(1), Some(Entity::new(2, 1)));
    assert_eq!(children.len(), 3);
}

#[test]
fn test_children_remove() {
    let mut children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let removed = children.remove(0);
    assert_eq!(removed, Entity::new(1, 1));
    assert_eq!(children.len(), 1);
}

#[test]
fn test_children_remove_child() {
    let mut children =
        Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1), Entity::new(3, 1)]);

    assert!(children.remove_child(Entity::new(2, 1)));
    assert_eq!(children.len(), 2);
    assert!(!children.contains(Entity::new(2, 1)));

    // Order preserved
    assert_eq!(children.get(0), Some(Entity::new(1, 1)));
    assert_eq!(children.get(1), Some(Entity::new(3, 1)));

    // Already removed
    assert!(!children.remove_child(Entity::new(2, 1)));
}

#[test]
fn test_children_swap_remove_child() {
    let mut children =
        Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1), Entity::new(3, 1)]);

    assert!(children.swap_remove_child(Entity::new(1, 1)));
    assert_eq!(children.len(), 2);
    assert!(!children.contains(Entity::new(1, 1)));

    // Order NOT preserved (last element moved to removed position)
    assert!(children.contains(Entity::new(2, 1)));
    assert!(children.contains(Entity::new(3, 1)));
}

#[test]
fn test_children_contains() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    assert!(children.contains(Entity::new(1, 1)));
    assert!(children.contains(Entity::new(2, 1)));
    assert!(!children.contains(Entity::new(3, 1)));
}

#[test]
fn test_children_get() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    assert_eq!(children.get(0), Some(Entity::new(1, 1)));
    assert_eq!(children.get(1), Some(Entity::new(2, 1)));
    assert_eq!(children.get(2), None);
}

#[test]
fn test_children_first_last() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1), Entity::new(3, 1)]);
    assert_eq!(children.first(), Some(Entity::new(1, 1)));
    assert_eq!(children.last(), Some(Entity::new(3, 1)));

    let empty = Children::new();
    assert_eq!(empty.first(), None);
    assert_eq!(empty.last(), None);
}

#[test]
fn test_children_iter() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let collected: Vec<_> = children.iter().copied().collect();
    assert_eq!(collected, vec![Entity::new(1, 1), Entity::new(2, 1)]);
}

#[test]
fn test_children_index_of() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1), Entity::new(3, 1)]);
    assert_eq!(children.index_of(Entity::new(1, 1)), Some(0));
    assert_eq!(children.index_of(Entity::new(2, 1)), Some(1));
    assert_eq!(children.index_of(Entity::new(3, 1)), Some(2));
    assert_eq!(children.index_of(Entity::new(99, 1)), None);
}

#[test]
fn test_children_clear() {
    let mut children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    children.clear();
    assert!(children.is_empty());
}

#[test]
fn test_children_as_slice() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let slice = children.as_slice();
    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0], Entity::new(1, 1));
}

#[test]
fn test_children_retain() {
    let mut children = Children::from_slice(&[
        Entity::new(1, 1),
        Entity::new(2, 1),
        Entity::new(3, 1),
        Entity::new(4, 1),
    ]);

    children.retain(|e| e.index() % 2 == 0);

    assert_eq!(children.len(), 2);
    assert!(children.contains(Entity::new(2, 1)));
    assert!(children.contains(Entity::new(4, 1)));
}

#[test]
fn test_children_sort_by_index() {
    let mut children =
        Children::from_slice(&[Entity::new(3, 1), Entity::new(1, 1), Entity::new(2, 1)]);

    children.sort_by_index();

    assert_eq!(children.get(0), Some(Entity::new(1, 1)));
    assert_eq!(children.get(1), Some(Entity::new(2, 1)));
    assert_eq!(children.get(2), Some(Entity::new(3, 1)));
}

#[test]
fn test_children_sort_by() {
    let mut children =
        Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1), Entity::new(3, 1)]);

    // Reverse sort
    children.sort_by(|a, b| b.index().cmp(&a.index()));

    assert_eq!(children.get(0), Some(Entity::new(3, 1)));
    assert_eq!(children.get(1), Some(Entity::new(2, 1)));
    assert_eq!(children.get(2), Some(Entity::new(1, 1)));
}

#[test]
fn test_children_default() {
    let children: Children = Default::default();
    assert!(children.is_empty());
}

#[test]
fn test_children_debug() {
    let children = Children::from_slice(&[Entity::new(1, 1)]);
    let debug = format!("{:?}", children);
    assert!(debug.contains("Children"));
    assert!(debug.contains("count"));
}

#[test]
fn test_children_display() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let display = format!("{}", children);
    assert!(display.contains("Children(2)"));
}

#[test]
fn test_children_into_iter_ref() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let collected: Vec<_> = (&children).into_iter().copied().collect();
    assert_eq!(collected, vec![Entity::new(1, 1), Entity::new(2, 1)]);
}

#[test]
fn test_children_into_iter_owned() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let collected: Vec<_> = children.into_iter().collect();
    assert_eq!(collected, vec![Entity::new(1, 1), Entity::new(2, 1)]);
}

#[test]
fn test_children_from_vec() {
    let vec = vec![Entity::new(1, 1), Entity::new(2, 1)];
    let children: Children = vec.into();
    assert_eq!(children.len(), 2);
}

#[test]
fn test_children_from_slice_trait() {
    let slice = &[Entity::new(1, 1), Entity::new(2, 1)][..];
    let children: Children = slice.into();
    assert_eq!(children.len(), 2);
}

#[test]
fn test_vec_from_children() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let vec: Vec<Entity> = children.into();
    assert_eq!(vec.len(), 2);
}

#[test]
fn test_children_is_component() {
    fn assert_component<T: Component>() {}
    assert_component::<Children>();
}

#[test]
fn test_children_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Children>();
}

#[test]
fn test_children_clone() {
    let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let cloned = children.clone();
    assert_eq!(children, cloned);
}

#[test]
fn test_children_eq() {
    let c1 = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let c2 = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    let c3 = Children::from_slice(&[Entity::new(2, 1), Entity::new(1, 1)]); // Different order

    assert_eq!(c1, c2);
    assert_ne!(c1, c3); // Order matters
}

#[test]
fn test_children_many() {
    let mut children = Children::new();
    for i in 0..1000 {
        children.push(Entity::new(i, 1));
    }
    assert_eq!(children.len(), 1000);

    for i in 0..1000 {
        assert!(children.contains(Entity::new(i, 1)));
    }
}
