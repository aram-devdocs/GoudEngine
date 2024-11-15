use std::{collections::BTreeMap, ffi::c_uint};

use crate::types::{Sprite, SpriteCreateDto, SpriteUpdateDto, Text, TextCreateDto, TextUpdateDto};

pub struct ECS {
    pub sprites: BTreeMap<i32, Vec<Sprite>>,
    pub texts: BTreeMap<i32, Vec<Text>>,
    next_id: c_uint,
    free_list: Vec<c_uint>,
}

impl ECS {
    pub fn new() -> Self {
        ECS {
            sprites: BTreeMap::new(),
            texts: BTreeMap::new(),
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

    /// Adds a text block to the ECS and returns its unique EntityId.
    pub fn add_text(&mut self, text_input: TextCreateDto) -> c_uint {
        // let id = if let Some(id) = self.free_list.pop() {
        //     id
        // } else {
        //     let id = self.next_id;
        //     self.next_id += 1;
        //     id
        // };


        let id = 0;
        let text = Text {
            id,
            content: unsafe { std::ffi::CStr::from_ptr(text_input.content).to_string_lossy().into_owned() },
            x: text_input.x,
            y: text_input.y,
            scale: text_input.scale,
            color: (text_input.color.r, text_input.color.g, text_input.color.b),
            font_id: text_input.font_id,
            z_layer: text_input.z_layer,
        };
        self.texts
            .entry(text.z_layer)
            .or_insert_with(Vec::new)
            .push(text);
        id
    }

    pub fn update_text(&mut self, text_input: TextUpdateDto) -> Result<(), String> {
        let text = Text {
            id: text_input.id,
            content: unsafe { std::ffi::CStr::from_ptr(text_input.content).to_string_lossy().into_owned() },
            x: text_input.x,
            y: text_input.y,
            scale: text_input.scale,
            color: (text_input.color.r, text_input.color.g, text_input.color.b),
            font_id: text_input.font_id,
            z_layer: text_input.z_layer,
        };
        for texts in self.texts.values_mut() {
            if let Some(existing_text) = texts.iter_mut().find(|t| t.id == text.id) {
                *existing_text = text;
                return Ok(());
            }
        }
        Err("Text EntityId not found".into())
    }

    pub fn get_text(&self, text_id: c_uint) -> Option<&Text> {
        for texts in self.texts.values() {
            if let Some(text) = texts.iter().find(|t| t.id == text_id) {
                return Some(text);
            }
        }
        None
    }

    pub fn remove_text(&mut self, text_id: c_uint) -> Result<Text, String> {
        for texts in self.texts.values_mut() {
            if let Some(index) = texts.iter().position(|t| t.id == text_id) {
                return Ok(texts.remove(index));
            }
        }
        Err("Text EntityId not found".into())
    }

    pub fn terminate(&mut self) {
        self.sprites.clear();
        self.free_list.clear();
        self.next_id = 0;
        self.texts.clear();
    }
}
