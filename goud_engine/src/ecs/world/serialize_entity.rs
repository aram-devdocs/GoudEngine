//! Entity serialization and deserialization.
//!
//! Provides methods to serialize individual entities (with all their
//! serializable components) to JSON and to deserialize components back
//! onto entities.

use super::super::component::ComponentId;
use super::super::entity::Entity;
use super::super::Component;
use super::World;

impl World {
    /// Registers a component type as serializable.
    ///
    /// After registration, entities with this component can have it
    /// serialized/deserialized via [`serialize_entity`](Self::serialize_entity)
    /// and [`deserialize_entity_components`](Self::deserialize_entity_components).
    ///
    /// The component type must implement `Serialize` and `Deserialize`.
    ///
    /// # Type Name Fragility
    ///
    /// This method uses `std::any::type_name::<T>()` to key registered components.
    /// The returned type name includes full module paths (e.g., `my_game::components::Health`).
    /// Refactoring code structure or moving components to different modules will change
    /// these paths, breaking deserialization of existing scene files that reference the
    /// old type names. Plan component module stability accordingly.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    /// struct Health(f32);
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// world.register_serializable::<Health>();
    /// ```
    pub fn register_serializable<
        T: Component + serde::Serialize + for<'de> serde::Deserialize<'de>,
    >(
        &mut self,
    ) {
        let id = ComponentId::of::<T>();
        let type_name = std::any::type_name::<T>();

        self.storages
            .entry(id)
            .or_insert_with(super::storage_entry::ComponentStorageEntry::new::<T>)
            .set_serialize_fns::<T>();

        self.serializable_names.insert(type_name.to_string(), id);
    }

    /// Registers built-in engine components as serializable.
    ///
    /// Registers all standard engine components for serialization:
    /// - Transform2D, Transform (local transforms only)
    /// - Parent, Children, Name
    /// - Sprite, SpriteAnimator
    /// - RigidBody, Collider
    /// - AudioSource
    ///
    /// # Global Transforms Excluded
    ///
    /// GlobalTransform2D and GlobalTransform are **not** registered because they are
    /// system-computed values derived from local transforms and the hierarchy tree.
    /// They must be re-derived by the transform propagation system after deserialization,
    /// not loaded from scene files. Loading stale global transforms would violate
    /// the invariant that global transforms match parent-to-child propagation.
    ///
    /// Call this if you intend to serialize/deserialize entities with
    /// built-in components. This is opt-in.
    pub fn register_builtin_serializables(&mut self) {
        if self.builtins_serializable_registered {
            return;
        }
        use crate::ecs::components::hierarchy::{Children, Parent};
        use crate::ecs::components::{
            AudioSource, Collider, Name, RigidBody, Sprite, SpriteAnimator, Transform, Transform2D,
        };

        self.register_serializable::<Transform2D>();
        self.register_serializable::<Transform>();
        self.register_serializable::<Parent>();
        self.register_serializable::<Children>();
        self.register_serializable::<Name>();
        self.register_serializable::<Sprite>();
        self.register_serializable::<SpriteAnimator>();
        self.register_serializable::<RigidBody>();
        self.register_serializable::<Collider>();
        self.register_serializable::<AudioSource>();

        self.builtins_serializable_registered = true;
    }

    /// Serializes an entity and all its serializable components to JSON.
    ///
    /// Returns a JSON object with the entity ID and a map of component
    /// type names to their serialized values. Components not registered
    /// via [`register_serializable`](Self::register_serializable) are
    /// silently skipped.
    ///
    /// # Returns
    ///
    /// `Some(json)` if the entity is alive, `None` otherwise.
    ///
    /// # JSON Format
    ///
    /// ```json
    /// {
    ///     "entity": { "index": 0, "generation": 0 },
    ///     "components": {
    ///         "my_game::Position": { "x": 1.0, "y": 2.0 },
    ///         "my_game::Velocity": { "x": 0.0, "y": -9.8 }
    ///     }
    /// }
    /// ```
    pub fn serialize_entity(&self, entity: Entity) -> Option<serde_json::Value> {
        if !self.is_alive(entity) {
            return None;
        }

        // Get entity's archetype to find its component set
        let archetype_id = *self.entity_archetypes.get(&entity)?;
        let component_ids: Vec<ComponentId> = self
            .archetypes
            .get(archetype_id)
            .map(|arch| arch.components().iter().copied().collect())
            .unwrap_or_default();

        let mut components = serde_json::Map::new();

        for component_id in &component_ids {
            if let Some(entry) = self.storages.get(component_id) {
                if let Some(type_name) = entry.type_name() {
                    if let Some(value) = entry.serialize_component(entity) {
                        components.insert(type_name.to_string(), value);
                    }
                }
            }
        }

        let entity_json = serde_json::to_value(entity).ok()?;

        Some(serde_json::json!({
            "entity": entity_json,
            "components": components,
        }))
    }

    /// Deserializes components from JSON and inserts them onto an entity.
    ///
    /// Reads the "components" map from the JSON and inserts each component
    /// whose type name is registered via
    /// [`register_serializable`](Self::register_serializable).
    ///
    /// # Returns
    ///
    /// The number of components successfully deserialized and inserted.
    pub fn deserialize_entity_components(
        &mut self,
        entity: Entity,
        json: &serde_json::Value,
    ) -> usize {
        if !self.is_alive(entity) {
            return 0;
        }

        let components = match json.get("components").and_then(|v| v.as_object()) {
            Some(map) => map,
            None => return 0,
        };

        let mut count = 0;

        // Use split borrows to avoid cloning serializable_names
        let name_to_id = &self.serializable_names;
        let storages = &mut self.storages;
        let archetypes = &mut self.archetypes;
        let entity_archetypes = &mut self.entity_archetypes;

        for (type_name, value) in components {
            let component_id = match name_to_id.get(type_name) {
                Some(id) => *id,
                None => continue,
            };

            // Deserialize the component
            let boxed = match storages.get(&component_id) {
                Some(entry) => entry.deserialize_component(value),
                None => continue,
            };

            let boxed = match boxed {
                Some(b) => b,
                None => continue,
            };

            // Insert the component
            if let Some(entry) = storages.get_mut(&component_id) {
                if entry.insert_any(entity, boxed) {
                    // Update archetype
                    let current_arch_id = entity_archetypes[&entity];
                    let target_arch_id = archetypes.get_add_edge(current_arch_id, component_id);

                    if current_arch_id != target_arch_id {
                        if let Some(old) = archetypes.get_mut(current_arch_id) {
                            old.remove_entity(entity);
                        }
                        if let Some(new) = archetypes.get_mut(target_arch_id) {
                            new.add_entity(entity);
                        }
                        entity_archetypes.insert(entity, target_arch_id);
                    }

                    count += 1;
                }
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::Component;
    use super::super::World;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Velocity {
        vx: f32,
        vy: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, PartialEq)]
    struct NonSerializable {
        _data: Vec<u8>,
    }
    impl Component for NonSerializable {}

    #[test]
    fn test_serialize_entity_single_component() {
        let mut world = World::new();
        world.register_serializable::<Position>();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        let json = world.serialize_entity(entity).unwrap();
        let components = json["components"].as_object().unwrap();

        assert_eq!(components.len(), 1);
        assert!(components.values().next().unwrap()["x"]
            .as_f64()
            .unwrap()
            .eq(&1.0));
        assert!(components.values().next().unwrap()["y"]
            .as_f64()
            .unwrap()
            .eq(&2.0));
    }

    #[test]
    fn test_serialize_entity_multiple_components() {
        let mut world = World::new();
        world.register_serializable::<Position>();
        world.register_serializable::<Velocity>();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 5.0, y: 10.0 });
        world.insert(entity, Velocity { vx: 1.0, vy: -1.0 });

        let json = world.serialize_entity(entity).unwrap();
        let components = json["components"].as_object().unwrap();

        assert_eq!(components.len(), 2);
    }

    #[test]
    fn test_serialize_nonexistent_entity_returns_none() {
        let world = World::new();
        let fake = super::super::super::entity::Entity::new(999, 1);

        assert!(world.serialize_entity(fake).is_none());
    }

    #[test]
    fn test_serialize_skips_unregistered_components() {
        let mut world = World::new();
        world.register_serializable::<Position>();
        // NonSerializable is NOT registered

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });
        world.insert(
            entity,
            NonSerializable {
                _data: vec![1, 2, 3],
            },
        );

        let json = world.serialize_entity(entity).unwrap();
        let components = json["components"].as_object().unwrap();

        // Only Position should be serialized
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn test_roundtrip_serialize_deserialize() {
        let mut world = World::new();
        world.register_serializable::<Position>();
        world.register_serializable::<Velocity>();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 42.0, y: 99.0 });
        world.insert(entity, Velocity { vx: 1.5, vy: -3.0 });

        // Serialize
        let json = world.serialize_entity(entity).unwrap();

        // Deserialize onto a new entity
        let new_entity = world.spawn_empty();
        let count = world.deserialize_entity_components(new_entity, &json);

        assert_eq!(count, 2);
        assert_eq!(
            world.get::<Position>(new_entity),
            Some(&Position { x: 42.0, y: 99.0 })
        );
        assert_eq!(
            world.get::<Velocity>(new_entity),
            Some(&Velocity { vx: 1.5, vy: -3.0 })
        );
    }

    #[test]
    fn test_deserialize_onto_dead_entity_returns_zero() {
        let mut world = World::new();
        world.register_serializable::<Position>();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });
        let json = world.serialize_entity(entity).unwrap();

        let fake = super::super::super::entity::Entity::new(999, 1);
        let count = world.deserialize_entity_components(fake, &json);
        assert_eq!(count, 0);
    }
}
