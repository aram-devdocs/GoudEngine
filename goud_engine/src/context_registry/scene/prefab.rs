//! Prefab system for reusable entity templates.
//!
//! A [`PrefabData`] captures one or more entities (with their components)
//! as a template that can be instantiated multiple times into a [`World`].
//! Prefabs support nesting via the [`PrefabRef`] component.

use std::collections::{HashMap, HashSet};

use crate::core::error::GoudError;
use crate::ecs::components::hierarchy::Children;
use crate::ecs::entity::Entity;
use crate::ecs::Component;
use crate::ecs::World;

use super::data::{EntityData, SceneData, SerializedEntity};
use super::serialization::deserialize_scene;

// =============================================================================
// PrefabRef Component
// =============================================================================

/// A component that references another prefab by name.
///
/// When an entity carries this component, the prefab system can
/// resolve it during instantiation to recursively spawn nested
/// prefabs.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct PrefabRef {
    /// The name of the referenced prefab.
    pub name: String,
}

impl PrefabRef {
    /// Creates a new `PrefabRef` with the given prefab name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Component for PrefabRef {}

// =============================================================================
// PrefabData
// =============================================================================

/// A reusable entity template.
///
/// Contains the serialized form of one or more entities that can be
/// instantiated (cloned) into a [`World`] any number of times. Each
/// instantiation creates fresh entities with remapped IDs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrefabData {
    /// Human-readable name of the prefab.
    pub name: String,
    /// Serialized entities that make up the prefab.
    pub entities: Vec<EntityData>,
}

impl PrefabData {
    /// Creates a prefab from a root entity and its children.
    ///
    /// Serializes the root entity and recursively collects all
    /// descendant entities via [`Children`] components.
    ///
    /// # Errors
    ///
    /// Returns an error if the root entity cannot be serialized.
    pub fn from_entity(
        world: &World,
        root: Entity,
        name: &str,
    ) -> Result<Self, GoudError> {
        let mut entities = Vec::new();
        Self::collect_entity(world, root, &mut entities)?;

        Ok(Self {
            name: name.to_string(),
            entities,
        })
    }

    /// Instantiates the prefab into a world, spawning new entities.
    ///
    /// Returns the root entity (the first entity in the remap) and a
    /// list of all newly spawned entities.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn instantiate(
        &self,
        world: &mut World,
    ) -> Result<Entity, GoudError> {
        let (root, _) = self.instantiate_with_entities(world)?;
        Ok(root)
    }

    /// Like [`instantiate`](Self::instantiate) but also returns the
    /// list of newly spawned entities.
    fn instantiate_with_entities(
        &self,
        world: &mut World,
    ) -> Result<(Entity, Vec<Entity>), GoudError> {
        if self.entities.is_empty() {
            return Err(GoudError::InternalError(
                "Prefab has no entities to instantiate".to_string(),
            ));
        }

        let scene_data = SceneData {
            name: self.name.clone(),
            entities: self.entities.clone(),
        };

        let remap = deserialize_scene(&scene_data, world)?;

        let root_key = self.entities[0].id;
        let root =
            remap.0.get(&root_key).copied().ok_or_else(|| {
                GoudError::InternalError(
                    "Root entity not found in remap after instantiation"
                        .to_string(),
                )
            })?;

        let spawned: Vec<Entity> = remap.0.values().copied().collect();
        Ok((root, spawned))
    }

    /// Parses a prefab from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid.
    pub fn from_json(json: &str) -> Result<Self, GoudError> {
        serde_json::from_str(json).map_err(|e| {
            GoudError::InternalError(format!(
                "Failed to parse prefab JSON: {}",
                e
            ))
        })
    }

    /// Serializes the prefab to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, GoudError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            GoudError::InternalError(format!(
                "Failed to serialize prefab to JSON: {}",
                e
            ))
        })
    }

    /// Recursively collects entity data from a root entity.
    fn collect_entity(
        world: &World,
        entity: Entity,
        out: &mut Vec<EntityData>,
    ) -> Result<(), GoudError> {
        let json = world.serialize_entity(entity).ok_or_else(|| {
            GoudError::InternalError(format!(
                "Failed to serialize entity {:?}",
                entity
            ))
        })?;

        let components_map = json
            .get("components")
            .and_then(|v| v.as_object())
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<String, serde_json::Value>>()
            })
            .unwrap_or_default();

        out.push(EntityData {
            id: SerializedEntity::from_entity(entity),
            components: components_map,
        });

        // Recurse into children.
        if let Some(children) = world.get::<Children>(entity) {
            let child_entities: Vec<Entity> =
                children.as_slice().to_vec();
            for child in child_entities {
                Self::collect_entity(world, child, out)?;
            }
        }

        Ok(())
    }
}

// =============================================================================
// Nested Prefab Instantiation
// =============================================================================

/// Instantiates a prefab and recursively resolves any [`PrefabRef`]
/// components by looking up prefabs in the provided registry.
///
/// # Cycle Detection
///
/// Tracks which prefab names are currently being instantiated. If a
/// cycle is detected (e.g., prefab A references B, B references A),
/// returns an error.
///
/// # Errors
///
/// Returns an error if:
/// - Instantiation of any prefab fails
/// - A referenced prefab is not found in the registry
/// - A cycle is detected in prefab references
pub fn instantiate_with_prefabs(
    data: &PrefabData,
    world: &mut World,
    prefab_registry: &HashMap<String, PrefabData>,
) -> Result<Entity, GoudError> {
    let mut visited = HashSet::new();
    instantiate_recursive(data, world, prefab_registry, &mut visited)
}

/// Internal recursive instantiation with cycle detection.
fn instantiate_recursive(
    data: &PrefabData,
    world: &mut World,
    prefab_registry: &HashMap<String, PrefabData>,
    visited: &mut HashSet<String>,
) -> Result<Entity, GoudError> {
    if !visited.insert(data.name.clone()) {
        return Err(GoudError::InternalError(format!(
            "Cycle detected in prefab references: '{}'",
            data.name
        )));
    }

    let (root, spawned) = data.instantiate_with_entities(world)?;

    // Only check newly spawned entities for PrefabRef components.
    let prefab_refs: Vec<(Entity, String)> = spawned
        .iter()
        .filter_map(|&e| {
            world
                .get::<PrefabRef>(e)
                .map(|pr| (e, pr.name.clone()))
        })
        .collect();

    for (entity, ref_name) in prefab_refs {
        // Remove the PrefabRef before recursing to avoid
        // re-processing it in nested calls.
        world.remove::<PrefabRef>(entity);

        if let Some(child_prefab) = prefab_registry.get(&ref_name) {
            instantiate_recursive(
                child_prefab,
                world,
                prefab_registry,
                visited,
            )?;
        } else {
            return Err(GoudError::ResourceNotFound(format!(
                "Referenced prefab '{}' not found in registry",
                ref_name
            )));
        }
    }

    visited.remove(&data.name);

    Ok(root)
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

    /// Helper: create a world with built-in serializables + PrefabRef.
    fn test_world() -> World {
        let mut world = World::new();
        world.register_builtin_serializables();
        world.register_serializable::<PrefabRef>();
        world
    }

    // ----- from_entity + instantiate -----------------------------------------

    #[test]
    fn test_prefab_from_entity_and_instantiate() {
        let mut world = test_world();

        let entity = world.spawn_empty();
        world.insert(entity, Name::new("soldier"));
        world.insert(
            entity,
            Transform2D::new(Vec2::new(1.0, 2.0), 0.0, Vec2::one()),
        );

        let prefab =
            PrefabData::from_entity(&world, entity, "soldier_prefab")
                .unwrap();

        // Instantiate into same world.
        let new_root = prefab.instantiate(&mut world).unwrap();

        assert_ne!(new_root, entity, "new entity should differ");
        let name = world.get::<Name>(new_root).unwrap();
        assert_eq!(name.as_str(), "soldier");

        let transform =
            world.get::<Transform2D>(new_root).unwrap();
        assert!(
            (transform.position.x - 1.0).abs() < f32::EPSILON
        );
    }

    // ----- instantiate 100 times ---------------------------------------------

    #[test]
    fn test_instantiate_100_times_isolation() {
        let mut world = test_world();

        let entity = world.spawn_empty();
        world.insert(entity, Name::new("unit"));

        let prefab =
            PrefabData::from_entity(&world, entity, "unit_prefab")
                .unwrap();

        let before = world.entity_count();

        for _ in 0..100 {
            prefab.instantiate(&mut world).unwrap();
        }

        assert_eq!(
            world.entity_count(),
            before + 100,
            "should have 100 more entities"
        );
    }

    // ----- nested prefab -----------------------------------------------------

    #[test]
    fn test_nested_prefab_instantiation() {
        let mut world = test_world();

        // Build prefab B (a simple entity).
        let b_entity = world.spawn_empty();
        world.insert(b_entity, Name::new("turret"));
        let prefab_b =
            PrefabData::from_entity(&world, b_entity, "turret")
                .unwrap();

        // Build prefab A that references B via PrefabRef.
        let a_entity = world.spawn_empty();
        world.insert(a_entity, Name::new("tank"));
        world.insert(a_entity, PrefabRef::new("turret"));
        let prefab_a =
            PrefabData::from_entity(&world, a_entity, "tank").unwrap();

        let mut registry = HashMap::new();
        registry.insert("turret".to_string(), prefab_b);

        let before = world.entity_count();

        let root = instantiate_with_prefabs(
            &prefab_a,
            &mut world,
            &registry,
        )
        .unwrap();

        // We should have spawned at least 2 new entities (tank + turret).
        assert!(
            world.entity_count() >= before + 2,
            "both prefab entities should exist"
        );

        let name = world.get::<Name>(root).unwrap();
        assert_eq!(name.as_str(), "tank");
    }

    // ----- cycle detection ---------------------------------------------------

    #[test]
    fn test_cycle_detection_errors() {
        let mut world = test_world();

        // Prefab A refs B.
        let a_entity = world.spawn_empty();
        world.insert(a_entity, Name::new("A"));
        world.insert(a_entity, PrefabRef::new("B"));
        let prefab_a =
            PrefabData::from_entity(&world, a_entity, "A").unwrap();

        // Prefab B refs A.
        let b_entity = world.spawn_empty();
        world.insert(b_entity, Name::new("B"));
        world.insert(b_entity, PrefabRef::new("A"));
        let prefab_b =
            PrefabData::from_entity(&world, b_entity, "B").unwrap();

        let mut registry = HashMap::new();
        registry.insert("A".to_string(), prefab_a.clone());
        registry.insert("B".to_string(), prefab_b);

        let result = instantiate_with_prefabs(
            &prefab_a,
            &mut world,
            &registry,
        );

        assert!(result.is_err(), "should detect cycle");
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("Cycle") || err_msg.contains("cycle"),
            "error should mention cycle"
        );
    }

    // ----- JSON roundtrip ----------------------------------------------------

    #[test]
    fn test_prefab_json_roundtrip() {
        let mut world = test_world();

        let entity = world.spawn_empty();
        world.insert(entity, Name::new("archer"));
        world.insert(
            entity,
            Transform2D::new(Vec2::new(5.0, 10.0), 0.0, Vec2::one()),
        );

        let prefab =
            PrefabData::from_entity(&world, entity, "archer_prefab")
                .unwrap();

        let json = prefab.to_json().unwrap();
        assert!(json.contains("archer_prefab"));
        assert!(json.contains("archer"));

        let parsed = PrefabData::from_json(&json).unwrap();
        assert_eq!(parsed.name, "archer_prefab");
        assert_eq!(parsed.entities.len(), prefab.entities.len());
    }

    #[test]
    fn test_prefab_from_invalid_json_errors() {
        let result = PrefabData::from_json("{{invalid}}}");
        assert!(result.is_err());
    }
}
