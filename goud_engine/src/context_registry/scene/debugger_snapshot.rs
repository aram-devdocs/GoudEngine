use std::collections::BTreeMap;

use crate::core::debugger::EntityStateV1;
use crate::ecs::{Entity, World};

fn entity_name(components: &BTreeMap<String, serde_json::Value>) -> Option<String> {
    components
        .get("Name")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

fn entity_components(world: &World, entity: Entity) -> BTreeMap<String, serde_json::Value> {
    world
        .serialize_entity(entity)
        .and_then(|value| value.get("components").cloned())
        .and_then(|value| value.as_object().cloned())
        .map(|components| components.into_iter().collect())
        .unwrap_or_default()
}

/// Builds debugger-friendly entity summaries for one scene.
pub fn collect_debugger_entities(
    world: &World,
    scene_id: impl Into<String>,
    selected_entity: Option<(&str, u64)>,
) -> Vec<EntityStateV1> {
    let scene_id = scene_id.into();
    let mut entities = Vec::new();

    for archetype in world.archetypes().iter() {
        for entity in archetype.entities() {
            let entity_id = entity.to_bits();
            let components = entity_components(world, *entity);
            let mut component_types: Vec<String> = components.keys().cloned().collect();
            component_types.sort();

            let mut entity_state = EntityStateV1::summary_only(
                entity_id,
                scene_id.clone(),
                entity_name(&components),
                component_types,
            );
            if selected_entity
                .as_ref()
                .is_some_and(|(selected_scene_id, selected_entity_id)| {
                    selected_scene_id == &scene_id && *selected_entity_id == entity_id
                })
            {
                entity_state.components = components;
            }
            entities.push(entity_state);
        }
    }

    entities.sort_by_key(|entity| entity.entity_id);
    entities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec2;
    use crate::ecs::components::hierarchy::Name;
    use crate::ecs::components::Transform2D;

    #[test]
    fn test_collect_debugger_entities_only_expands_selected_entity() {
        let mut world = World::new();
        world.register_builtin_serializables();

        let hero = world.spawn_empty();
        world.insert(hero, Name::new("hero"));
        world.insert(
            hero,
            Transform2D::new(Vec2::new(1.0, 2.0), 0.0, Vec2::one()),
        );

        let npc = world.spawn_empty();
        world.insert(npc, Name::new("npc"));

        let entities =
            collect_debugger_entities(&world, "default", Some(("default", hero.to_bits())));
        assert_eq!(entities.len(), 2);
        assert!(!entities[0].component_types.is_empty());
        assert!(entities
            .iter()
            .any(|entity| entity.entity_id == hero.to_bits() && !entity.components.is_empty()));
        assert!(entities
            .iter()
            .any(|entity| entity.entity_id == npc.to_bits() && entity.components.is_empty()));
    }

    #[test]
    fn test_collect_debugger_entities_does_not_expand_selected_entity_from_other_scene() {
        let mut world = World::new();
        world.register_builtin_serializables();

        let hero = world.spawn_empty();
        world.insert(hero, Name::new("hero"));
        world.insert(
            hero,
            Transform2D::new(Vec2::new(1.0, 2.0), 0.0, Vec2::one()),
        );

        let entities =
            collect_debugger_entities(&world, "default", Some(("other-scene", hero.to_bits())));

        assert_eq!(entities.len(), 1);
        assert!(entities[0].components.is_empty());
    }
}
