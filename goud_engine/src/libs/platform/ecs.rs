use std::{collections::BTreeMap, ffi::c_uint};

use crate::types::{Sprite, SpriteCreateDto, SpriteUpdateDto};

pub struct ECS {
    pub sprites: BTreeMap<i32, Vec<Sprite>>,
    next_id: c_uint,        // Tracks the next unused ID
    free_list: Vec<c_uint>, // List of reusable indices
}

impl ECS {
    pub fn new() -> Self {
        ECS {
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

    // refactor using BTreeMap
    pub fn get_sprite(&self, sprite_id: c_uint) -> Option<&Sprite> {
        for sprites in self.sprites.values() {
            if let Some(sprite) = sprites.iter().find(|s| s.id == sprite_id) {
                return Some(sprite);
            }
        }
        None
    }

    // refactor using BTreeMap
    pub fn remove_sprite(&mut self, sprite_id: c_uint) -> Result<Sprite, String> {
        for sprites in self.sprites.values_mut() {
            if let Some(index) = sprites.iter().position(|s| s.id == sprite_id) {
                return Ok(sprites.remove(index));
            }
        }
        Err("EntityId not found".into())
    }

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
