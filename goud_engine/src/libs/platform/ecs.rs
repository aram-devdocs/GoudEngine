use crate::types::{EntityId, Sprite};
use std::collections::{HashMap, VecDeque};

pub struct ECS {
    pub sprites: HashMap<EntityId, Sprite>, // HashMap for constant-time access by EntityId
    free_ids: VecDeque<EntityId>,           // Queue for reusable entity IDs
    next_id: EntityId,                      // Tracks the next unused ID
}

impl ECS {
    pub fn new() -> Self {
        ECS {
            sprites: HashMap::new(),
            free_ids: VecDeque::new(),
            next_id: 0,
        }
    }

    /// Adds a sprite to the ECS and returns its unique EntityId.
    pub fn add_sprite(&mut self, sprite: Sprite) -> EntityId {
        // Check for a reusable ID
        let entity_id = if let Some(reused_id) = self.free_ids.pop_front() {
            reused_id
        } else {
            // If no reusable ID, use the next available ID
            let new_id = self.next_id;
            self.next_id += 1;
            new_id
        };

        // Insert sprite into the HashMap
        self.sprites.insert(entity_id, sprite);
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

    /// Removes a sprite by EntityId.
    pub fn remove_sprite(&mut self, entity_id: EntityId) -> Result<Sprite, String> {
        if let Some(sprite) = self.sprites.remove(&entity_id) {
            self.free_ids.push_back(entity_id); // Recycle the ID
            Ok(sprite)
        } else {
            Err("EntityId not found".into())
        }
    }

    pub fn terminate(&mut self) {
        self.sprites.clear();
        self.free_ids.clear();
        self.next_id = 0;
    }
}
