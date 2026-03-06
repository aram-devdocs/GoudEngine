//! Integration tests exercising all three hierarchy components together.

use crate::ecs::components::hierarchy::{Children, Name, Parent};
use crate::ecs::entity::Entity;
use crate::ecs::ComponentId;

#[test]
fn test_hierarchy_components_work_together() {
    // Simulate a simple hierarchy
    let parent_entity = Entity::new(0, 1);
    let child1 = Entity::new(1, 1);
    let child2 = Entity::new(2, 1);

    // Parent with children
    let children = Children::from_slice(&[child1, child2]);
    assert_eq!(children.len(), 2);

    // Children with parent
    let parent1 = Parent::new(parent_entity);
    let parent2 = Parent::new(parent_entity);
    assert_eq!(parent1.get(), parent_entity);
    assert_eq!(parent2.get(), parent_entity);

    // Names for debugging
    let parent_name = Name::new("Root");
    let child1_name = Name::new("Child_A");
    let child2_name = Name::new("Child_B");

    assert_eq!(parent_name.as_str(), "Root");
    assert!(child1_name.starts_with("Child"));
    assert!(child2_name.starts_with("Child"));
}

#[test]
fn test_hierarchy_mutation() {
    let parent_entity = Entity::new(0, 1);
    let child1 = Entity::new(1, 1);
    let child2 = Entity::new(2, 1);
    let new_parent = Entity::new(3, 1);

    // Start with one child
    let mut children = Children::new();
    children.push(child1);

    // Child has parent
    let mut parent_comp = Parent::new(parent_entity);

    // Add another child
    children.push(child2);
    assert_eq!(children.len(), 2);

    // Reparent the child
    parent_comp.set(new_parent);
    assert_eq!(parent_comp.get(), new_parent);

    // Remove from old parent's children
    children.remove_child(child1);
    assert_eq!(children.len(), 1);
    assert!(!children.contains(child1));
}

#[test]
fn test_components_are_distinct() {
    // Verify these are distinct component types
    let parent_id = ComponentId::of::<Parent>();
    let children_id = ComponentId::of::<Children>();
    let name_id = ComponentId::of::<Name>();

    assert_ne!(parent_id, children_id);
    assert_ne!(parent_id, name_id);
    assert_ne!(children_id, name_id);
}
