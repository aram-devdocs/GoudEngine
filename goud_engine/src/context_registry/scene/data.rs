//! Data types for scene serialization.
//!
//! These types represent the serialized form of a scene: entities and their
//! components as JSON-friendly structures that can be saved/loaded.

use std::collections::HashMap;

use crate::ecs::entity::Entity;

// =============================================================================
// SerializedEntity
// =============================================================================

/// A serialized entity identifier.
///
/// Captures the index and generation of an [`Entity`] so it can be
/// reconstructed or used as a key in a remap table.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct SerializedEntity {
    /// The entity slot index.
    pub index: u32,
    /// The entity generation.
    pub generation: u32,
}

impl SerializedEntity {
    /// Creates a `SerializedEntity` from a live [`Entity`].
    #[inline]
    pub fn from_entity(entity: Entity) -> Self {
        Self {
            index: entity.index(),
            generation: entity.generation(),
        }
    }
}

// =============================================================================
// EntityData
// =============================================================================

/// Serialized representation of a single entity and its components.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityData {
    /// The original entity identifier.
    pub id: SerializedEntity,
    /// Map of component type name to serialized component value.
    pub components: HashMap<String, serde_json::Value>,
}

// =============================================================================
// SceneData
// =============================================================================

/// The complete serialized form of a scene.
///
/// Contains the scene name and a list of all serialized entities with
/// their components.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneData {
    /// The name of the scene.
    pub name: String,
    /// All entities in the scene.
    pub entities: Vec<EntityData>,
}

// =============================================================================
// EntityRemap
// =============================================================================

/// Maps old (serialized) entity IDs to newly spawned entities.
///
/// After deserializing a scene, components that reference other entities
/// (e.g., [`Parent`](crate::ecs::components::Parent),
/// [`Children`](crate::ecs::components::Children)) need their entity
/// references updated to point to the newly created entities.
#[derive(Debug, Clone)]
pub struct EntityRemap(pub HashMap<SerializedEntity, Entity>);
