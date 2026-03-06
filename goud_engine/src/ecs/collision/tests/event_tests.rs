//! Tests for [`CollisionStarted`] and [`CollisionEnded`] event types.

use crate::core::event::Event;
use crate::core::math::Vec2;
use crate::ecs::collision::contact::Contact;
use crate::ecs::collision::events::{CollisionEnded, CollisionStarted};
use crate::ecs::Entity;

// =========================================================================
// CollisionStarted Tests
// =========================================================================

#[test]
fn test_collision_started_new() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.5);

    let event = CollisionStarted::new(entity_a, entity_b, contact);

    assert_eq!(event.entity_a, entity_a);
    assert_eq!(event.entity_b, entity_b);
    assert_eq!(event.contact, contact);
}

#[test]
fn test_collision_started_involves() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let entity_c = Entity::from_bits(3);
    let contact = Contact::default();

    let event = CollisionStarted::new(entity_a, entity_b, contact);

    assert!(event.involves(entity_a));
    assert!(event.involves(entity_b));
    assert!(!event.involves(entity_c));
}

#[test]
fn test_collision_started_other_entity() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let entity_c = Entity::from_bits(3);
    let contact = Contact::default();

    let event = CollisionStarted::new(entity_a, entity_b, contact);

    assert_eq!(event.other_entity(entity_a), Some(entity_b));
    assert_eq!(event.other_entity(entity_b), Some(entity_a));
    assert_eq!(event.other_entity(entity_c), None);
}

#[test]
fn test_collision_started_ordered_pair() {
    let entity_1 = Entity::from_bits(1);
    let entity_2 = Entity::from_bits(2);
    let contact = Contact::default();

    let event1 = CollisionStarted::new(entity_1, entity_2, contact);
    let event2 = CollisionStarted::new(entity_2, entity_1, contact);

    assert_eq!(event1.ordered_pair(), (entity_1, entity_2));
    assert_eq!(event2.ordered_pair(), (entity_1, entity_2));
}

#[test]
fn test_collision_started_implements_event() {
    fn accepts_event<E: Event>(_: E) {}

    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let contact = Contact::default();
    let event = CollisionStarted::new(entity_a, entity_b, contact);

    accepts_event(event);
}

#[test]
fn test_collision_started_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CollisionStarted>();
}

#[test]
fn test_collision_started_clone() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let contact = Contact::default();

    let event = CollisionStarted::new(entity_a, entity_b, contact);
    let cloned = event.clone();

    assert_eq!(event, cloned);
}

#[test]
fn test_collision_started_debug() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let contact = Contact::default();

    let event = CollisionStarted::new(entity_a, entity_b, contact);
    let debug_str = format!("{:?}", event);

    assert!(debug_str.contains("CollisionStarted"));
    assert!(debug_str.contains("entity_a"));
    assert!(debug_str.contains("entity_b"));
    assert!(debug_str.contains("contact"));
}

// =========================================================================
// CollisionEnded Tests
// =========================================================================

#[test]
fn test_collision_ended_new() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);

    let event = CollisionEnded::new(entity_a, entity_b);

    assert_eq!(event.entity_a, entity_a);
    assert_eq!(event.entity_b, entity_b);
}

#[test]
fn test_collision_ended_involves() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let entity_c = Entity::from_bits(3);

    let event = CollisionEnded::new(entity_a, entity_b);

    assert!(event.involves(entity_a));
    assert!(event.involves(entity_b));
    assert!(!event.involves(entity_c));
}

#[test]
fn test_collision_ended_other_entity() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let entity_c = Entity::from_bits(3);

    let event = CollisionEnded::new(entity_a, entity_b);

    assert_eq!(event.other_entity(entity_a), Some(entity_b));
    assert_eq!(event.other_entity(entity_b), Some(entity_a));
    assert_eq!(event.other_entity(entity_c), None);
}

#[test]
fn test_collision_ended_ordered_pair() {
    let entity_1 = Entity::from_bits(1);
    let entity_2 = Entity::from_bits(2);

    let event1 = CollisionEnded::new(entity_1, entity_2);
    let event2 = CollisionEnded::new(entity_2, entity_1);

    assert_eq!(event1.ordered_pair(), (entity_1, entity_2));
    assert_eq!(event2.ordered_pair(), (entity_1, entity_2));
}

#[test]
fn test_collision_ended_implements_event() {
    fn accepts_event<E: Event>(_: E) {}

    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let event = CollisionEnded::new(entity_a, entity_b);

    accepts_event(event);
}

#[test]
fn test_collision_ended_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CollisionEnded>();
}

#[test]
fn test_collision_ended_clone() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);

    let event = CollisionEnded::new(entity_a, entity_b);
    let cloned = event.clone();

    assert_eq!(event, cloned);
}

#[test]
fn test_collision_ended_debug() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);

    let event = CollisionEnded::new(entity_a, entity_b);
    let debug_str = format!("{:?}", event);

    assert!(debug_str.contains("CollisionEnded"));
    assert!(debug_str.contains("entity_a"));
    assert!(debug_str.contains("entity_b"));
}

#[test]
fn test_collision_ended_hash() {
    use std::collections::HashSet;

    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let entity_c = Entity::from_bits(3);

    let mut set: HashSet<CollisionEnded> = HashSet::new();
    set.insert(CollisionEnded::new(entity_a, entity_b));
    set.insert(CollisionEnded::new(entity_b, entity_c));
    set.insert(CollisionEnded::new(entity_a, entity_b)); // Duplicate

    assert_eq!(set.len(), 2);
}

// =========================================================================
// Integration Tests
// =========================================================================

#[test]
fn test_collision_pair_consistency() {
    let entity_a = Entity::from_bits(1);
    let entity_b = Entity::from_bits(2);
    let contact = Contact::default();

    let start1 = CollisionStarted::new(entity_a, entity_b, contact);
    let start2 = CollisionStarted::new(entity_b, entity_a, contact);
    let end1 = CollisionEnded::new(entity_a, entity_b);
    let end2 = CollisionEnded::new(entity_b, entity_a);

    assert_eq!(start1.ordered_pair(), start2.ordered_pair());
    assert_eq!(end1.ordered_pair(), end2.ordered_pair());
    assert_eq!(start1.ordered_pair(), end1.ordered_pair());
}

#[test]
fn test_collision_event_workflow() {
    let player = Entity::from_bits(1);
    let enemy = Entity::from_bits(2);
    let contact = Contact::new(Vec2::new(5.0, 5.0), Vec2::new(1.0, 0.0), 0.2);

    let start_event = CollisionStarted::new(player, enemy, contact);
    assert!(start_event.involves(player));
    assert_eq!(start_event.other_entity(player), Some(enemy));
    assert!(start_event.contact.is_colliding());

    let end_event = CollisionEnded::new(player, enemy);
    assert!(end_event.involves(player));
    assert_eq!(end_event.other_entity(player), Some(enemy));

    assert_eq!(start_event.ordered_pair(), end_event.ordered_pair());
}
