//! Tests for [`Archetype`].

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::ecs::archetype::{Archetype, ArchetypeId};
    use crate::ecs::component::{Component, ComponentId};
    use crate::ecs::entity::Entity;

    // Component types used across tests
    #[derive(Debug, Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, Copy)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, Copy)]
    struct Health(f32);
    impl Component for Health {}

    #[derive(Debug, Clone, Copy)]
    struct Player;
    impl Component for Player {}

    fn make_components(ids: &[ComponentId]) -> BTreeSet<ComponentId> {
        ids.iter().cloned().collect()
    }

    // ==================== Construction Tests ====================

    #[test]
    fn test_archetype_new_empty() {
        let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        assert_eq!(archetype.id(), ArchetypeId::EMPTY);
        assert!(archetype.components().is_empty());
        assert!(archetype.entities().is_empty());
        assert_eq!(archetype.len(), 0);
        assert!(archetype.is_empty());
        assert_eq!(archetype.component_count(), 0);
        assert!(archetype.has_no_components());
    }

    #[test]
    fn test_archetype_new_with_components() {
        let components =
            make_components(&[ComponentId::of::<Position>(), ComponentId::of::<Velocity>()]);
        let archetype = Archetype::new(ArchetypeId::new(1), components.clone());

        assert_eq!(archetype.id().index(), 1);
        assert_eq!(archetype.components(), &components);
        assert_eq!(archetype.component_count(), 2);
        assert!(!archetype.has_no_components());
    }

    #[test]
    fn test_archetype_with_capacity() {
        let archetype = Archetype::with_capacity(ArchetypeId::new(5), BTreeSet::new(), 1000);
        assert_eq!(archetype.id().index(), 5);
        assert!(archetype.is_empty());
    }

    #[test]
    fn test_archetype_default() {
        let archetype = Archetype::default();
        assert_eq!(archetype.id(), ArchetypeId::EMPTY);
        assert!(archetype.has_no_components());
        assert!(archetype.is_empty());
    }

    #[test]
    fn test_archetype_clone() {
        let components =
            make_components(&[ComponentId::of::<Position>(), ComponentId::of::<Velocity>()]);
        let archetype = Archetype::new(ArchetypeId::new(5), components);
        let cloned = archetype.clone();

        assert_eq!(cloned.id(), archetype.id());
        assert_eq!(cloned.components(), archetype.components());
    }

    // ==================== Component Query Tests ====================

    #[test]
    fn test_archetype_has_component() {
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        let archetype = Archetype::new(ArchetypeId::new(1), make_components(&[pos_id, vel_id]));
        assert!(archetype.has_component(pos_id));
        assert!(archetype.has_component(vel_id));
        assert!(!archetype.has_component(health_id));
    }

    #[test]
    fn test_archetype_has_all() {
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        let archetype = Archetype::new(ArchetypeId::new(1), make_components(&[pos_id, vel_id]));
        assert!(archetype.has_all(&[pos_id, vel_id]));
        assert!(archetype.has_all(&[pos_id]));
        assert!(archetype.has_all(&[])); // vacuous truth
        assert!(!archetype.has_all(&[health_id]));
        assert!(!archetype.has_all(&[pos_id, health_id]));
    }

    #[test]
    fn test_archetype_has_none() {
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();
        let player_id = ComponentId::of::<Player>();

        let archetype = Archetype::new(ArchetypeId::new(1), make_components(&[pos_id, vel_id]));
        assert!(archetype.has_none(&[health_id, player_id]));
        assert!(archetype.has_none(&[])); // vacuous truth
        assert!(!archetype.has_none(&[pos_id]));
        assert!(!archetype.has_none(&[pos_id, health_id]));
    }

    #[test]
    fn test_archetype_component_count() {
        let empty = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        assert_eq!(empty.component_count(), 0);

        let four = Archetype::new(
            ArchetypeId::new(1),
            make_components(&[
                ComponentId::of::<Position>(),
                ComponentId::of::<Velocity>(),
                ComponentId::of::<Health>(),
                ComponentId::of::<Player>(),
            ]),
        );
        assert_eq!(four.component_count(), 4);
    }

    #[test]
    fn test_archetype_component_set_insertion_order_independent() {
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        let mut set1 = BTreeSet::new();
        set1.insert(pos_id);
        set1.insert(vel_id);
        set1.insert(health_id);

        let mut set2 = BTreeSet::new();
        set2.insert(health_id);
        set2.insert(pos_id);
        set2.insert(vel_id);

        assert_eq!(set1, set2); // BTreeSet is order-independent
    }

    #[test]
    fn test_archetype_many_components() {
        macro_rules! define_marker_components {
            ($($name:ident),*) => {
                $(
                    #[derive(Debug)]
                    struct $name;
                    impl Component for $name {}
                )*
            };
        }
        define_marker_components!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);

        let components = make_components(&[
            ComponentId::of::<C1>(),
            ComponentId::of::<C2>(),
            ComponentId::of::<C3>(),
            ComponentId::of::<C4>(),
            ComponentId::of::<C5>(),
            ComponentId::of::<C6>(),
            ComponentId::of::<C7>(),
            ComponentId::of::<C8>(),
            ComponentId::of::<C9>(),
            ComponentId::of::<C10>(),
        ]);
        let archetype = Archetype::new(ArchetypeId::new(1), components);
        assert_eq!(archetype.component_count(), 10);
        assert!(archetype.has_component(ComponentId::of::<C1>()));
        assert!(archetype.has_component(ComponentId::of::<C10>()));
    }

    // ==================== Entity Management Tests ====================

    #[test]
    fn test_add_entity_basic() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        assert_eq!(archetype.add_entity(Entity::new(0, 1)), 0);
        assert_eq!(archetype.add_entity(Entity::new(1, 1)), 1);
        assert_eq!(archetype.add_entity(Entity::new(2, 1)), 2);
        assert_eq!(archetype.len(), 3);
    }

    #[test]
    fn test_add_entity_idempotent() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let e1 = Entity::new(0, 1);
        assert_eq!(archetype.add_entity(e1), 0);
        assert_eq!(archetype.add_entity(e1), 0); // same index
        assert_eq!(archetype.add_entity(e1), 0);
        assert_eq!(archetype.len(), 1);
    }

    #[test]
    fn test_contains_entity() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);

        assert!(archetype.contains_entity(e1));
        assert!(archetype.contains_entity(e2));
        assert!(!archetype.contains_entity(e3));
    }

    #[test]
    fn test_entity_index() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);

        assert_eq!(archetype.entity_index(e1), Some(0));
        assert_eq!(archetype.entity_index(e2), Some(1));
        assert_eq!(archetype.entity_index(e3), None);
    }

    #[test]
    fn test_remove_entity_last() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        archetype.add_entity(e1);
        archetype.add_entity(e2);

        let (idx, swapped) = archetype.remove_entity(e2).unwrap();
        assert_eq!(idx, 1);
        assert!(swapped.is_none());
        assert_eq!(archetype.len(), 1);
        assert!(archetype.contains_entity(e1));
    }

    #[test]
    fn test_remove_entity_swap_remove() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);
        archetype.add_entity(e1);
        archetype.add_entity(e2);
        archetype.add_entity(e3);

        let (idx, swapped) = archetype.remove_entity(e1).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(swapped, Some(e3));

        assert_eq!(archetype.len(), 2);
        assert!(!archetype.contains_entity(e1));
        assert_eq!(archetype.entity_index(e3), Some(0));
        assert_eq!(archetype.entity_index(e2), Some(1));
        assert_eq!(archetype.entities()[0], e3);
        assert_eq!(archetype.entities()[1], e2);
    }

    #[test]
    fn test_remove_entity_not_found() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        archetype.add_entity(e1);

        assert!(archetype.remove_entity(e2).is_none());
        assert_eq!(archetype.len(), 1);
    }

    #[test]
    fn test_remove_entity_single() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let e1 = Entity::new(0, 1);
        archetype.add_entity(e1);

        let (idx, swapped) = archetype.remove_entity(e1).unwrap();
        assert_eq!(idx, 0);
        assert!(swapped.is_none());
        assert!(archetype.is_empty());
    }

    #[test]
    fn test_remove_entity_middle() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);
        let e4 = Entity::new(3, 1);
        archetype.add_entity(e1);
        archetype.add_entity(e2);
        archetype.add_entity(e3);
        archetype.add_entity(e4);

        let (idx, swapped) = archetype.remove_entity(e2).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(swapped, Some(e4));
        assert_eq!(archetype.entity_index(e4), Some(1));
        assert_eq!(archetype.entity_index(e2), None);
    }

    #[test]
    fn test_clear_entities() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        archetype.add_entity(Entity::new(0, 1));
        archetype.add_entity(Entity::new(1, 1));
        archetype.clear_entities();
        assert!(archetype.is_empty());
        assert_eq!(archetype.len(), 0);
    }

    #[test]
    fn test_reserve_entities() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        archetype.reserve_entities(1000);
        assert!(archetype.is_empty());

        for i in 0..1000 {
            archetype.add_entity(Entity::new(i, 1));
        }
        assert_eq!(archetype.len(), 1000);
    }

    #[test]
    fn test_entity_index_consistency_after_removals() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let entities: Vec<Entity> = (0..5).map(|i| Entity::new(i, 1)).collect();
        for &e in &entities {
            archetype.add_entity(e);
        }

        archetype.remove_entity(entities[1]);
        verify_index_consistency(&archetype);

        archetype.remove_entity(entities[3]);
        verify_index_consistency(&archetype);

        archetype.remove_entity(entities[0]);
        verify_index_consistency(&archetype);
    }

    fn verify_index_consistency(archetype: &Archetype) {
        for (actual_idx, &entity) in archetype.entities().iter().enumerate() {
            let stored_idx = archetype.entity_index(entity);
            assert_eq!(
                stored_idx,
                Some(actual_idx),
                "Entity {:?} at index {} but entity_index returned {:?}",
                entity,
                actual_idx,
                stored_idx
            );
        }
    }

    #[test]
    fn test_entity_management_stress() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let entities: Vec<Entity> = (0..1000).map(|i| Entity::new(i, 1)).collect();
        for &e in &entities {
            archetype.add_entity(e);
        }
        assert_eq!(archetype.len(), 1000);

        for (i, &e) in entities.iter().enumerate() {
            if i % 2 == 0 {
                assert!(archetype.remove_entity(e).is_some());
            }
        }
        assert_eq!(archetype.len(), 500);

        for (i, &e) in entities.iter().enumerate() {
            if i % 2 == 0 {
                assert!(!archetype.contains_entity(e));
            } else {
                assert!(archetype.contains_entity(e));
            }
        }
    }

    // ==================== Thread Safety Tests ====================

    #[test]
    fn test_archetype_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Archetype>();
    }

    #[test]
    fn test_archetype_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Archetype>();
    }

    // ==================== Debug Tests ====================

    #[test]
    fn test_archetype_debug() {
        let archetype = Archetype::new(
            ArchetypeId::new(1),
            make_components(&[ComponentId::of::<Position>()]),
        );
        let debug_str = format!("{:?}", archetype);
        assert!(debug_str.contains("Archetype"));
        assert!(debug_str.contains("id"));
        assert!(debug_str.contains("components"));
        assert!(debug_str.contains("entities"));
    }

    #[test]
    fn test_debug_with_entities() {
        let mut archetype = Archetype::new(ArchetypeId::new(1), BTreeSet::new());
        archetype.add_entity(Entity::new(0, 1));
        let debug_str = format!("{:?}", archetype);
        assert!(debug_str.contains("entity_indices"));
    }
}
