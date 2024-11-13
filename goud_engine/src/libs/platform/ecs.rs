use std::{collections::BTreeMap, ffi::c_uint};

use crate::types::{Sprite, SpriteCreateDto, SpriteUpdateDto};

pub struct ECS {
    // pub sprites: Vec<Option<Sprite>>, // Vec for storing sprites with optional entries
    // store as BTree to allow for ordering by z_layer
    // should we use a BTreeMap or a BTreeSet?
    // BTreeMap allows for duplicate z_layer values
    // BTreeSet does not allow for duplicate z_layer values
    // well we need to allow for duplicate z_layer values
    // so we should use a BTreeMap
    pub sprites: BTreeMap<i32, Vec<Sprite>>,
    next_id: c_uint,        // Tracks the next unused ID
    free_list: Vec<c_uint>, // List of reusable indices
}

impl ECS {
    pub fn new() -> Self {
        ECS {
            // sprites: Vec::new(),
            sprites: BTreeMap::new(),
            next_id: 0,
            free_list: Vec::new(),
        }
    }

    /// Adds a sprite to the ECS and returns its unique EntityId.
    pub fn add_sprite(&mut self, sprite_input: SpriteCreateDto) -> c_uint {
        if let Some(id) = self.free_list.pop() {
            // Reuse an index from the free list
            let sprite = Sprite::new(
                id,
                sprite_input.x,
                sprite_input.y,
                sprite_input.z_layer,
                sprite_input.scale_x,
                sprite_input.scale_y,
                sprite_input.dimension_x,
                sprite_input.dimension_y,
                sprite_input.rotation,
                sprite_input.source_rect,
                sprite_input.texture_id,
                sprite_input.debug,
            );
            self.sprites
                .entry(sprite.z_layer)
                .or_insert_with(Vec::new)
                .push(sprite);
            id
        } else {
            // No reusable slots; push to the end
            // self.sprites.push(Some(sprite));

            let sprite = Sprite::new(
                self.next_id,
                sprite_input.x,
                sprite_input.y,
                sprite_input.z_layer,
                sprite_input.scale_x,
                sprite_input.scale_y,
                sprite_input.dimension_x,
                sprite_input.dimension_y,
                sprite_input.rotation,
                sprite_input.source_rect,
                sprite_input.texture_id,
                sprite_input.debug,
            );
            self.sprites
                .entry(sprite.z_layer)
                .or_insert_with(Vec::new)
                .push(sprite);
            self.next_id += 1;
            self.next_id - 1
        }
    }

    /// Updates a sprite for a given EntityId.
    // pub fn update_sprite(&mut self, entity_id: EntityId, sprite: Sprite) -> Result<(), String> {
    //     if let Some(slot) = self.sprites.get_mut(entity_id as usize) {
    //         if slot.is_some() {
    //             *slot = Some(sprite);
    //             Ok(())
    //         } else {
    //             Err("EntityId not found".into())
    //         }
    //     } else {
    //         Err("EntityId not found".into())
    //     }
    // }

    // refactor using BTreeMap
    pub fn update_sprite(&mut self, sprite_input: SpriteUpdateDto) -> Result<(), String> {
        let sprite = Sprite::new(
            sprite_input.id,
            sprite_input.x,
            sprite_input.y,
            sprite_input.z_layer,
            sprite_input.scale_x,
            sprite_input.scale_y,
            sprite_input.dimension_x,
            sprite_input.dimension_y,
            sprite_input.rotation,
            sprite_input.source_rect,
            sprite_input.texture_id,
            sprite_input.debug,
        );
        for sprites in self.sprites.values_mut() {
            if let Some(existing_sprite) = sprites.iter_mut().find(|s| s.id == sprite.id) {
                *existing_sprite = sprite;
                return Ok(());
            }
        }
        Err("EntityId not found".into())
    }

    /// Retrieves a sprite by EntityId, if it exists.
    // pub fn get_sprite(&self, entity_id: EntityId) -> Option<&Sprite> {
    //     self.sprites
    //         .get(entity_id as usize)
    //         .and_then(|slot| slot.as_ref())
    // }

    // refactor using BTreeMap
    pub fn get_sprite(&self, sprite_id: c_uint) -> Option<&Sprite> {
        for sprites in self.sprites.values() {
            if let Some(sprite) = sprites.iter().find(|s| s.id == sprite_id) {
                return Some(sprite);
            }
        }
        None
    }

    /// Removes a sprite by EntityId.
    // pub fn remove_sprite(&mut self, entity_id: EntityId) -> Result<Sprite, String> {
    //     if let Some(slot) = self.sprites.get_mut(entity_id as usize) {
    //         if let Some(sprite) = slot.take() {
    //             self.free_list.push(entity_id); // Mark index as reusable
    //             return Ok(sprite);
    //         }
    //     }
    //     Err("EntityId not found".into())
    // }

    // refactor using BTreeMap
    pub fn remove_sprite(&mut self, sprite_id: c_uint) -> Result<Sprite, String> {
        for sprites in self.sprites.values_mut() {
            if let Some(index) = sprites.iter().position(|s| s.id == sprite_id) {
                return Ok(sprites.remove(index));
            }
        }
        Err("EntityId not found".into())
    }

    // pub fn check_collision_between_sprites(
    //     &self,
    //     entity_id1: EntityId,
    //     entity_id2: EntityId,
    // ) -> bool {
    //     match (
    //         self.sprites
    //             .get(entity_id1 as usize)
    //             .and_then(|s| s.as_ref()),
    //         self.sprites
    //             .get(entity_id2 as usize)
    //             .and_then(|s| s.as_ref()),
    //     ) {
    //         (Some(sprite1), Some(sprite2)) => sprite1.check_collision(sprite2),
    //         _ => false,
    //     }
    // }

    // refactor using BTreeMap
    pub fn check_collision_between_sprites(&self, sprite_id1: c_uint, sprite_id2: c_uint) -> bool {
        match (self.get_sprite(sprite_id1), self.get_sprite(sprite_id2)) {
            (Some(sprite1), Some(sprite2)) => sprite1.check_collision(sprite2),
            _ => false,
        }
    }

    pub fn terminate(&mut self) {
        self.sprites.clear();
        self.free_list.clear();
        self.next_id = 0;
    }
}
