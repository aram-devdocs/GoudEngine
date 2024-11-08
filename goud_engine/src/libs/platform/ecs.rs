use crate::types::{EntityId, Sprite, SpriteMap};
use std::collections::HashMap;

pub struct ECS {
    pub sprites: SpriteMap,
    next_id: EntityId, // Tracks the next unique ID to assign
}

impl ECS {
    pub fn new() -> Self {
        ECS {
            sprites: HashMap::new(),
            next_id: 0, // Start ID at 0 or any other initial value
        }
    }

    /// Adds a sprite to the ECS and returns its unique EntityId.
    pub fn add_sprite(&mut self, sprite: Sprite) -> EntityId {
        let entity_id = self.next_id;
        self.sprites.insert(entity_id, sprite);
        self.next_id += 1; // Increment the ID for the next entity
        entity_id
    }

    /// Updates a sprite for a given EntityId.
    pub fn update_sprite(&mut self, entity_id: EntityId, sprite: Sprite) -> Result<(), String> {
        if self.sprites.contains_key(&entity_id) {
            self.sprites.insert(entity_id, sprite);
            Ok(())
        } else {
            Err("EntityId not found".into())
        }
    }

    /// Retrieves a sprite by EntityId, if it exists.
    pub fn get_sprite(&self, entity_id: EntityId) -> Option<&Sprite> {
        self.sprites.get(&entity_id)
    }
}
