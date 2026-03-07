//! Scene serialization and deserialization.
//!
//! Converts an entire [`World`] (or a subset) to/from [`SceneData`] for
//! persistence, networking, or scene-switching.

use std::collections::HashMap;

use crate::core::error::GoudError;
use crate::ecs::components::hierarchy::{Children, Parent};
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::data::{EntityData, EntityRemap, SceneData, SerializedEntity};

// =============================================================================
// Serialize
// =============================================================================

/// Serializes all entities in the world into a [`SceneData`].
///
/// Iterates every archetype, collects all entities, and serializes each
/// entity's registered components. Entities with no serializable components
/// are skipped.
///
/// # Errors
///
/// Currently infallible in practice, but returns `Result` for forward
/// compatibility (e.g., custom serialization errors).
pub fn serialize_scene(world: &World, name: &str) -> Result<SceneData, GoudError> {
    let mut entities_data = Vec::new();

    // Collect all entities from all archetypes.
    let all_entities: Vec<Entity> = world
        .archetypes()
        .iter()
        .flat_map(|archetype| archetype.entities().iter().copied())
        .collect();

    for entity in all_entities {
        let json = match world.serialize_entity(entity) {
            Some(v) => v,
            None => continue,
        };

        // Extract the components map from the JSON.
        let components_map = match json.get("components").and_then(|v| v.as_object()) {
            Some(m) => m,
            None => continue,
        };

        // Skip entities with no serializable components.
        if components_map.is_empty() {
            continue;
        }

        let components: HashMap<String, serde_json::Value> = components_map
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        entities_data.push(EntityData {
            id: SerializedEntity::from_entity(entity),
            components,
        });
    }

    Ok(SceneData {
        name: name.to_string(),
        entities: entities_data,
    })
}

// =============================================================================
// Deserialize
// =============================================================================

/// Deserializes a [`SceneData`] into a [`World`], spawning new entities and
/// populating their components.
///
/// Returns an [`EntityRemap`] mapping old serialized entity IDs to the newly
/// spawned entities. After the first pass inserts all components, a second
/// pass fixes entity references in [`Parent`] and [`Children`] components.
///
/// # Errors
///
/// Returns an error if component deserialization fails critically (currently
/// all failures are per-component and non-fatal).
pub fn deserialize_scene(data: &SceneData, world: &mut World) -> Result<EntityRemap, GoudError> {
    let mut remap = HashMap::new();

    // First pass: spawn entities and deserialize components.
    for entity_data in &data.entities {
        let new_entity = world.spawn_empty();
        remap.insert(entity_data.id, new_entity);

        // Build the JSON value that deserialize_entity_components expects.
        let components_json = serde_json::json!({
            "components": &entity_data.components,
        });

        world.deserialize_entity_components(new_entity, &components_json);
    }

    // Second pass: remap entity references in Parent and Children.
    remap_entity_references(world, &remap);

    Ok(EntityRemap(remap))
}

// =============================================================================
// JSON helpers
// =============================================================================

/// Converts a [`SceneData`] to a pretty-printed JSON string.
///
/// # Errors
///
/// Returns [`GoudError::InternalError`] if serialization fails.
pub fn scene_to_json(data: &SceneData) -> Result<String, GoudError> {
    serde_json::to_string_pretty(data)
        .map_err(|e| GoudError::InternalError(format!("Failed to serialize scene to JSON: {}", e)))
}

/// Parses a JSON string into a [`SceneData`].
///
/// # Errors
///
/// Returns [`GoudError::InternalError`] if the JSON is invalid or does not
/// match the expected schema.
pub fn scene_from_json(json: &str) -> Result<SceneData, GoudError> {
    serde_json::from_str(json).map_err(|e| {
        GoudError::InternalError(format!("Failed to deserialize scene from JSON: {}", e))
    })
}

// =============================================================================
// Entity reference remapping
// =============================================================================

/// Fixes entity references in [`Parent`] and [`Children`] components using
/// the remap table.
///
/// For each newly spawned entity:
/// - If it has a `Parent` component whose entity matches an old ID, replace
///   it with the corresponding new entity.
/// - If it has a `Children` component, replace each child entity that matches
///   an old ID.
fn remap_entity_references(world: &mut World, remap: &HashMap<SerializedEntity, Entity>) {
    // Collect the new entities that need remapping.
    let new_entities: Vec<Entity> = remap.values().copied().collect();

    for &entity in &new_entities {
        // Remap Parent.
        if let Some(parent) = world.get::<Parent>(entity) {
            let old_parent = parent.get();
            let serialized = SerializedEntity::from_entity(old_parent);
            if let Some(&new_parent) = remap.get(&serialized) {
                world.insert(entity, Parent::new(new_parent));
            }
        }

        // Remap Children.
        if let Some(children) = world.get::<Children>(entity) {
            let old_children: Vec<Entity> = children.as_slice().to_vec();
            let mut remapped = Vec::with_capacity(old_children.len());
            let mut changed = false;

            for child in &old_children {
                let serialized = SerializedEntity::from_entity(*child);
                if let Some(&new_child) = remap.get(&serialized) {
                    remapped.push(new_child);
                    changed = true;
                } else {
                    remapped.push(*child);
                }
            }

            if changed {
                world.insert(entity, Children::from(remapped));
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec2;
    use crate::ecs::components::hierarchy::Name;
    use crate::ecs::components::Transform2D;
    use crate::ecs::World;

    /// Helper: create a world with built-in serializables registered.
    fn test_world() -> World {
        let mut world = World::new();
        world.register_builtin_serializables();
        world
    }

    // ----- empty scene -------------------------------------------------------

    #[test]
    fn test_serialize_empty_scene() {
        let world = test_world();

        let scene_data = serialize_scene(&world, "empty").unwrap();

        assert_eq!(scene_data.name, "empty");
        assert!(scene_data.entities.is_empty());
    }

    #[test]
    fn test_roundtrip_empty_scene() {
        let world = test_world();

        let scene_data = serialize_scene(&world, "empty").unwrap();
        let mut world2 = test_world();
        let remap = deserialize_scene(&scene_data, &mut world2).unwrap();

        assert!(remap.0.is_empty());
        assert_eq!(world2.entity_count(), 0);
    }

    // ----- basic round-trip --------------------------------------------------

    #[test]
    fn test_roundtrip_with_transform_and_name() {
        let mut world = test_world();

        let entity = world.spawn_empty();
        world.insert(
            entity,
            Transform2D::new(Vec2::new(10.0, 20.0), 0.0, Vec2::one()),
        );
        world.insert(entity, Name::new("player"));

        let scene_data = serialize_scene(&world, "test_scene").unwrap();

        assert_eq!(scene_data.entities.len(), 1);
        assert_eq!(scene_data.name, "test_scene");

        // Deserialize into a fresh world.
        let mut world2 = test_world();
        let remap = deserialize_scene(&scene_data, &mut world2).unwrap();

        assert_eq!(remap.0.len(), 1);
        assert_eq!(world2.entity_count(), 1);

        // Verify the components on the new entity.
        let new_entity = *remap.0.values().next().unwrap();
        let name = world2.get::<Name>(new_entity).unwrap();
        assert_eq!(name.as_str(), "player");

        let transform = world2.get::<Transform2D>(new_entity).unwrap();
        assert!((transform.position.x - 10.0).abs() < f32::EPSILON);
        assert!((transform.position.y - 20.0).abs() < f32::EPSILON);
    }

    // ----- hierarchy round-trip ----------------------------------------------

    #[test]
    fn test_roundtrip_with_hierarchy() {
        let mut world = test_world();

        let parent = world.spawn_empty();
        world.insert(parent, Name::new("parent"));

        let child1 = world.spawn_empty();
        world.insert(child1, Name::new("child1"));
        world.insert(child1, Parent::new(parent));

        let child2 = world.spawn_empty();
        world.insert(child2, Name::new("child2"));
        world.insert(child2, Parent::new(parent));

        world.insert(parent, Children::from_slice(&[child1, child2]));

        let scene_data = serialize_scene(&world, "hierarchy").unwrap();

        assert_eq!(scene_data.entities.len(), 3);

        // Deserialize into a new world.
        let mut world2 = test_world();
        let remap = deserialize_scene(&scene_data, &mut world2).unwrap();

        assert_eq!(remap.0.len(), 3);
        assert_eq!(world2.entity_count(), 3);

        // Find the new parent entity.
        let old_parent_key = SerializedEntity::from_entity(parent);
        let new_parent = remap.0[&old_parent_key];

        // The new parent should have a Children component pointing
        // to the new child entities.
        let children = world2.get::<Children>(new_parent).unwrap();
        assert_eq!(children.len(), 2);

        // Both children should have Parent pointing to new_parent.
        let old_child1_key = SerializedEntity::from_entity(child1);
        let new_child1 = remap.0[&old_child1_key];
        let parent_comp = world2.get::<Parent>(new_child1).unwrap();
        assert_eq!(parent_comp.get(), new_parent);

        let old_child2_key = SerializedEntity::from_entity(child2);
        let new_child2 = remap.0[&old_child2_key];
        let parent_comp2 = world2.get::<Parent>(new_child2).unwrap();
        assert_eq!(parent_comp2.get(), new_parent);

        // Verify that the children in the Children component match
        // the remapped child entities.
        assert!(children.contains(new_child1));
        assert!(children.contains(new_child2));
    }

    // ----- JSON round-trip ---------------------------------------------------

    #[test]
    fn test_json_roundtrip() {
        let mut world = test_world();

        let entity = world.spawn_empty();
        world.insert(entity, Name::new("hero"));
        world.insert(
            entity,
            Transform2D::new(Vec2::new(5.0, 15.0), 0.0, Vec2::one()),
        );

        let scene_data = serialize_scene(&world, "json_test").unwrap();

        let json = scene_to_json(&scene_data).unwrap();

        // Verify JSON is valid and contains expected fields.
        assert!(json.contains("json_test"));
        assert!(json.contains("hero"));
        assert!(json.contains("entities"));
        assert!(json.contains("components"));

        // Parse back.
        let parsed = scene_from_json(&json).unwrap();
        assert_eq!(parsed.name, "json_test");
        assert_eq!(parsed.entities.len(), 1);
    }

    #[test]
    fn test_scene_from_invalid_json_returns_error() {
        let result = scene_from_json("not valid json {{{");
        assert!(result.is_err());
    }

    // ----- multiple entities, no hierarchy -----------------------------------

    #[test]
    fn test_roundtrip_multiple_entities() {
        let mut world = test_world();

        for i in 0..5 {
            let e = world.spawn_empty();
            world.insert(e, Name::new(format!("entity_{}", i)));
        }

        let scene_data = serialize_scene(&world, "multi").unwrap();
        assert_eq!(scene_data.entities.len(), 5);

        let mut world2 = test_world();
        let remap = deserialize_scene(&scene_data, &mut world2).unwrap();

        assert_eq!(remap.0.len(), 5);
        assert_eq!(world2.entity_count(), 5);
    }
}
