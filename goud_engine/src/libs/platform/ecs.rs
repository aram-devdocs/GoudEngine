use crate::types::{EntityId, Sprite};

pub struct ECS {
    pub sprites: Vec<Option<Sprite>>, // Vec for storing sprites with optional entries
    next_id: EntityId,                // Tracks the next unused ID
}

impl ECS {
    pub fn new() -> Self {
        ECS {
            sprites: Vec::new(),
            next_id: 0,
        }
    }

    /// Adds a sprite to the ECS and returns its unique EntityId.
    pub fn add_sprite(&mut self, sprite: Sprite) -> EntityId {
        // Check if there's a reusable slot
        for (id, slot) in self.sprites.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(sprite);
                return id as EntityId;
            }
        }

        // If no reusable slot, push to the end
        self.sprites.push(Some(sprite));
        self.next_id += 1;
        self.next_id - 1
    }

    /// Updates a sprite for a given EntityId.
    pub fn update_sprite(&mut self, entity_id: EntityId, sprite: Sprite) -> Result<(), String> {
        if let Some(slot) = self.sprites.get_mut(entity_id as usize) {
            if slot.is_some() {
                *slot = Some(sprite);
                Ok(())
            } else {
                Err("EntityId not found".into())
            }
        } else {
            Err("EntityId not found".into())
        }
    }

    /// Retrieves a sprite by EntityId, if it exists.
    pub fn get_sprite(&self, entity_id: EntityId) -> Option<&Sprite> {
        self.sprites.get(entity_id as usize).and_then(|slot| slot.as_ref())
    }

    /// Removes a sprite by EntityId.
    pub fn remove_sprite(&mut self, entity_id: EntityId) -> Result<Sprite, String> {
        if let Some(slot) = self.sprites.get_mut(entity_id as usize) {
            if let Some(sprite) = slot.take() {
                return Ok(sprite);
            }
        }
        Err("EntityId not found".into())
    }

    pub fn terminate(&mut self) {
        self.sprites.clear();
        self.next_id = 0;
    }
}