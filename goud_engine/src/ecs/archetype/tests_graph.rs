//! Tests for [`ArchetypeGraph`].

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::ecs::archetype::{ArchetypeGraph, ArchetypeId};
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

    // ==================== Construction Tests ====================

    #[test]
    fn test_graph_new() {
        let graph = ArchetypeGraph::new();
        assert_eq!(graph.len(), 1);
        assert!(!graph.is_empty());

        let empty = graph.get(ArchetypeId::EMPTY).unwrap();
        assert_eq!(empty.id(), ArchetypeId::EMPTY);
        assert!(empty.has_no_components());
        assert!(empty.is_empty());
    }

    #[test]
    fn test_graph_default() {
        let graph = ArchetypeGraph::default();
        assert_eq!(graph.len(), 1);
        assert!(graph.get(ArchetypeId::EMPTY).is_some());
    }

    // ==================== Lookup Tests ====================

    #[test]
    fn test_graph_get_nonexistent() {
        let graph = ArchetypeGraph::new();
        assert!(graph.get(ArchetypeId::new(1)).is_none());
        assert!(graph.get(ArchetypeId::new(u32::MAX)).is_none());
    }

    #[test]
    fn test_graph_get_mut() {
        let mut graph = ArchetypeGraph::new();
        let entity = Entity::new(0, 1);
        graph
            .get_mut(ArchetypeId::EMPTY)
            .unwrap()
            .add_entity(entity);
        assert!(graph
            .get(ArchetypeId::EMPTY)
            .unwrap()
            .contains_entity(entity));
    }

    #[test]
    fn test_graph_get_mut_nonexistent() {
        let mut graph = ArchetypeGraph::new();
        assert!(graph.get_mut(ArchetypeId::new(1)).is_none());
    }

    #[test]
    fn test_graph_contains() {
        let mut graph = ArchetypeGraph::new();
        assert!(graph.contains(ArchetypeId::EMPTY));
        assert!(!graph.contains(ArchetypeId::new(1)));

        let mut components = BTreeSet::new();
        components.insert(ComponentId::of::<Position>());
        let id = graph.find_or_create(components);
        assert!(graph.contains(id));
    }

    // ==================== find_or_create Tests ====================

    #[test]
    fn test_graph_find_or_create_empty() {
        let mut graph = ArchetypeGraph::new();
        let id = graph.find_or_create(BTreeSet::new());
        assert_eq!(id, ArchetypeId::EMPTY);
        assert_eq!(graph.len(), 1);
    }

    #[test]
    fn test_graph_find_or_create_new() {
        let mut graph = ArchetypeGraph::new();
        let mut components = BTreeSet::new();
        components.insert(ComponentId::of::<Position>());

        let id = graph.find_or_create(components);
        assert_ne!(id, ArchetypeId::EMPTY);
        assert_eq!(id.index(), 1);
        assert_eq!(graph.len(), 2);
        assert!(graph
            .get(id)
            .unwrap()
            .has_component(ComponentId::of::<Position>()));
    }

    #[test]
    fn test_graph_find_or_create_idempotent() {
        let mut graph = ArchetypeGraph::new();
        let mut components = BTreeSet::new();
        components.insert(ComponentId::of::<Position>());

        let id1 = graph.find_or_create(components.clone());
        let id2 = graph.find_or_create(components.clone());
        let id3 = graph.find_or_create(components);
        assert_eq!(id1, id2);
        assert_eq!(id1, id3);
        assert_eq!(graph.len(), 2);
    }

    #[test]
    fn test_graph_find_or_create_multiple() {
        let mut graph = ArchetypeGraph::new();

        let pos_id = graph.find_or_create([ComponentId::of::<Position>()].into_iter().collect());
        let vel_id = graph.find_or_create([ComponentId::of::<Velocity>()].into_iter().collect());
        let both_id = graph.find_or_create(
            [ComponentId::of::<Position>(), ComponentId::of::<Velocity>()]
                .into_iter()
                .collect(),
        );

        assert_ne!(pos_id, vel_id);
        assert_ne!(pos_id, both_id);
        assert_ne!(vel_id, both_id);
        assert_eq!(graph.len(), 4);
    }

    #[test]
    fn test_graph_find() {
        let mut graph = ArchetypeGraph::new();
        assert_eq!(graph.find(&BTreeSet::new()), Some(ArchetypeId::EMPTY));

        let mut components = BTreeSet::new();
        components.insert(ComponentId::of::<Position>());
        assert_eq!(graph.find(&components), None);

        let id = graph.find_or_create(components.clone());
        assert_eq!(graph.find(&components), Some(id));
    }

    #[test]
    fn test_graph_component_order_independence() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        let mut set1: BTreeSet<ComponentId> = BTreeSet::new();
        set1.insert(pos_id);
        set1.insert(vel_id);
        set1.insert(health_id);

        let mut set2: BTreeSet<ComponentId> = BTreeSet::new();
        set2.insert(health_id);
        set2.insert(pos_id);
        set2.insert(vel_id);

        assert_eq!(graph.find_or_create(set1), graph.find_or_create(set2));
        assert_eq!(graph.len(), 2);
    }

    // ==================== Iterator Tests ====================

    #[test]
    fn test_graph_iter() {
        let mut graph = ArchetypeGraph::new();
        graph.find_or_create([ComponentId::of::<Position>()].into_iter().collect());
        graph.find_or_create([ComponentId::of::<Velocity>()].into_iter().collect());

        let archetypes: Vec<_> = graph.iter().collect();
        assert_eq!(archetypes.len(), 3);
        assert_eq!(archetypes[0].id(), ArchetypeId::EMPTY);
    }

    #[test]
    fn test_graph_archetype_ids() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = graph.find_or_create([ComponentId::of::<Position>()].into_iter().collect());

        let ids: Vec<_> = graph.archetype_ids().collect();
        assert_eq!(ids, vec![ArchetypeId::EMPTY, pos_id]);
    }

    // ==================== Entity Count Tests ====================

    #[test]
    fn test_graph_entity_count() {
        let mut graph = ArchetypeGraph::new();
        assert_eq!(graph.entity_count(), 0);

        graph
            .get_mut(ArchetypeId::EMPTY)
            .unwrap()
            .add_entity(Entity::new(0, 1));
        graph
            .get_mut(ArchetypeId::EMPTY)
            .unwrap()
            .add_entity(Entity::new(1, 1));
        assert_eq!(graph.entity_count(), 2);

        let pos_id = graph.find_or_create([ComponentId::of::<Position>()].into_iter().collect());
        let arch = graph.get_mut(pos_id).unwrap();
        arch.add_entity(Entity::new(2, 1));
        arch.add_entity(Entity::new(3, 1));
        arch.add_entity(Entity::new(4, 1));
        assert_eq!(graph.entity_count(), 5);
    }

    // ==================== Edge Cache Tests ====================

    #[test]
    fn test_graph_edge_count_initial() {
        let graph = ArchetypeGraph::new();
        assert_eq!(graph.add_edge_count(), 0);
        assert_eq!(graph.remove_edge_count(), 0);
    }

    // ==================== Transition Tests ====================

    #[test]
    fn test_get_add_edge_from_empty() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let target = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        assert_ne!(target, ArchetypeId::EMPTY);
        assert!(graph.get(target).unwrap().has_component(pos_id));
        assert_eq!(graph.add_edge_count(), 1);
    }

    #[test]
    fn test_get_add_edge_existing_component_is_noop() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        assert_eq!(graph.get_add_edge(pos_arch, pos_id), pos_arch);
        assert_eq!(graph.add_edge_count(), 2);
    }

    #[test]
    fn test_get_add_edge_multiple_components() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();

        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        let both_arch = graph.get_add_edge(pos_arch, vel_id);

        let both = graph.get(both_arch).unwrap();
        assert!(both.has_component(pos_id));
        assert!(both.has_component(vel_id));
        assert_eq!(graph.len(), 3);
    }

    #[test]
    fn test_get_add_edge_caching() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();

        let t1 = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        assert_eq!(graph.add_edge_count(), 1);

        let t2 = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        assert_eq!(t1, t2);
        assert_eq!(graph.add_edge_count(), 1); // cached
    }

    #[test]
    fn test_get_remove_edge_basic() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        assert_eq!(
            graph.get_remove_edge(pos_arch, pos_id),
            Some(ArchetypeId::EMPTY)
        );
        assert_eq!(graph.remove_edge_count(), 1);
    }

    #[test]
    fn test_get_remove_edge_component_not_present() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        assert_eq!(graph.get_remove_edge(pos_arch, vel_id), None);
        assert_eq!(graph.remove_edge_count(), 0); // not cached
    }

    #[test]
    fn test_get_remove_edge_from_empty() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        assert_eq!(graph.get_remove_edge(ArchetypeId::EMPTY, pos_id), None);
    }

    #[test]
    fn test_get_remove_edge_to_existing_archetype() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();

        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        let both_arch = graph.get_add_edge(pos_arch, vel_id);

        assert_eq!(graph.get_remove_edge(both_arch, vel_id), Some(pos_arch));
    }

    #[test]
    fn test_get_remove_edge_caching() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        let t1 = graph.get_remove_edge(pos_arch, pos_id);
        assert_eq!(graph.remove_edge_count(), 1);
        let t2 = graph.get_remove_edge(pos_arch, pos_id);
        assert_eq!(t1, t2);
        assert_eq!(graph.remove_edge_count(), 1); // cached
    }

    #[test]
    fn test_transition_roundtrip() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        let mut current = ArchetypeId::EMPTY;
        current = graph.get_add_edge(current, pos_id);
        current = graph.get_add_edge(current, vel_id);
        current = graph.get_add_edge(current, health_id);
        assert_eq!(graph.get(current).unwrap().component_count(), 3);

        current = graph.get_remove_edge(current, vel_id).unwrap();
        let arch = graph.get(current).unwrap();
        assert!(arch.has_component(pos_id));
        assert!(!arch.has_component(vel_id));
        assert!(arch.has_component(health_id));

        current = graph.get_remove_edge(current, pos_id).unwrap();
        current = graph.get_remove_edge(current, health_id).unwrap();
        assert_eq!(current, ArchetypeId::EMPTY);
    }

    #[test]
    fn test_transition_converging_paths() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();

        // Path 1: empty -> Position -> Position+Velocity
        let via_a = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        let via_a_then_b = graph.get_add_edge(via_a, vel_id);

        // Path 2: empty -> Velocity -> Position+Velocity
        let via_b = graph.get_add_edge(ArchetypeId::EMPTY, vel_id);
        let via_b_then_a = graph.get_add_edge(via_b, pos_id);

        assert_eq!(via_a_then_b, via_b_then_a);
        assert_eq!(graph.len(), 4);
    }

    #[test]
    fn test_transition_edge_count_after_clear() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();

        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        graph.get_add_edge(pos_arch, vel_id);
        graph.get_remove_edge(pos_arch, pos_id);

        assert!(graph.add_edge_count() > 0);
        assert!(graph.remove_edge_count() > 0);

        graph.clear_edge_cache();
        assert_eq!(graph.add_edge_count(), 0);
        assert_eq!(graph.remove_edge_count(), 0);

        // Edges can be rebuilt
        let pos_arch2 = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        assert_eq!(pos_arch, pos_arch2);
        assert_eq!(graph.add_edge_count(), 1);
    }

    #[test]
    fn test_transition_stress() {
        let mut graph = ArchetypeGraph::new();
        let components = [
            ComponentId::of::<Position>(),
            ComponentId::of::<Velocity>(),
            ComponentId::of::<Health>(),
            ComponentId::of::<Player>(),
        ];

        let mut current = ArchetypeId::EMPTY;
        for &comp in &components {
            current = graph.get_add_edge(current, comp);
        }
        assert_eq!(graph.get(current).unwrap().component_count(), 4);

        for &comp in &components {
            if let Some(next) = graph.get_remove_edge(current, comp) {
                current = next;
            }
        }
        assert_eq!(current, ArchetypeId::EMPTY);
        assert!(graph.add_edge_count() >= 4);
        assert!(graph.remove_edge_count() >= 4);
    }

    // ==================== Stress / Correctness Tests ====================

    #[test]
    fn test_graph_all_combinations() {
        let mut graph = ArchetypeGraph::new();
        let base_ids = [
            ComponentId::of::<Position>(),
            ComponentId::of::<Velocity>(),
            ComponentId::of::<Health>(),
            ComponentId::of::<Player>(),
        ];

        // Create all 2^4 = 16 combinations
        for mask in 0u32..16 {
            let components: BTreeSet<ComponentId> = base_ids
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, &id)| id)
                .collect();
            graph.find_or_create(components);
        }
        assert_eq!(graph.len(), 16);

        for mask in 0u32..16 {
            let components: BTreeSet<ComponentId> = base_ids
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, &id)| id)
                .collect();
            assert!(graph.find(&components).is_some());
        }
    }

    // ==================== Thread Safety Tests ====================

    #[test]
    fn test_graph_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ArchetypeGraph>();
        assert_sync::<ArchetypeGraph>();
    }

    // ==================== Debug Tests ====================

    #[test]
    fn test_graph_debug() {
        let graph = ArchetypeGraph::new();
        let debug_str = format!("{:?}", graph);
        assert!(debug_str.contains("ArchetypeGraph"));
        assert!(debug_str.contains("archetypes"));
        assert!(debug_str.contains("component_index"));
        assert!(debug_str.contains("edges"));
    }
}
