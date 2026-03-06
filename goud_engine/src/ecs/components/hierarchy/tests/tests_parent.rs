//! Tests for the [`Parent`] component.

use crate::ecs::component::Component;
use crate::ecs::components::hierarchy::Parent;
use crate::ecs::entity::Entity;

#[test]
fn test_parent_new() {
    let entity = Entity::new(42, 1);
    let parent = Parent::new(entity);
    assert_eq!(parent.get(), entity);
}

#[test]
fn test_parent_get() {
    let entity = Entity::new(10, 2);
    let parent = Parent::new(entity);
    assert_eq!(parent.get().index(), 10);
    assert_eq!(parent.get().generation(), 2);
}

#[test]
fn test_parent_set() {
    let mut parent = Parent::new(Entity::new(0, 1));
    parent.set(Entity::new(5, 3));
    assert_eq!(parent.get(), Entity::new(5, 3));
}

#[test]
fn test_parent_default() {
    let parent = Parent::default();
    assert!(parent.get().is_placeholder());
}

#[test]
fn test_parent_from_entity() {
    let entity = Entity::new(7, 2);
    let parent: Parent = entity.into();
    assert_eq!(parent.get(), entity);
}

#[test]
fn test_entity_from_parent() {
    let parent = Parent::new(Entity::new(7, 2));
    let entity: Entity = parent.into();
    assert_eq!(entity, Entity::new(7, 2));
}

#[test]
fn test_parent_clone_copy() {
    let parent = Parent::new(Entity::new(1, 1));
    let cloned = parent.clone();
    let copied = parent;
    assert_eq!(parent, cloned);
    assert_eq!(parent, copied);
}

#[test]
fn test_parent_eq() {
    let p1 = Parent::new(Entity::new(1, 1));
    let p2 = Parent::new(Entity::new(1, 1));
    let p3 = Parent::new(Entity::new(2, 1));
    assert_eq!(p1, p2);
    assert_ne!(p1, p3);
}

#[test]
fn test_parent_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Parent::new(Entity::new(1, 1)));
    assert!(set.contains(&Parent::new(Entity::new(1, 1))));
    assert!(!set.contains(&Parent::new(Entity::new(2, 1))));
}

#[test]
fn test_parent_debug() {
    let parent = Parent::new(Entity::new(42, 3));
    let debug = format!("{:?}", parent);
    assert!(debug.contains("Parent"));
    assert!(debug.contains("42"));
}

#[test]
fn test_parent_display() {
    let parent = Parent::new(Entity::new(42, 3));
    let display = format!("{}", parent);
    assert!(display.contains("Parent"));
}

#[test]
fn test_parent_is_component() {
    fn assert_component<T: Component>() {}
    assert_component::<Parent>();
}

#[test]
fn test_parent_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Parent>();
}

#[test]
fn test_parent_size() {
    // Parent wraps Entity, so should be same size
    assert_eq!(std::mem::size_of::<Parent>(), std::mem::size_of::<Entity>());
    assert_eq!(std::mem::size_of::<Parent>(), 8);
}
