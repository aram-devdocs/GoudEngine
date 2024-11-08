// ecs.rs

use crate::game::cgmath::Vector2;
use crate::types::{EntityId, Sprite};
use std::collections::HashMap;
use std::rc::Rc;

pub struct ECS {
    pub sprites: Vec<Sprite>,
}

impl ECS {
    pub fn new() -> Self {
        ECS {
            sprites: Vec::new(),
        }
    }

    /// Adds a sprite to be rendered.
    pub fn add_sprite(&mut self, sprite: Sprite) -> usize {
        self.sprites.push(sprite);
        self.sprites.len() - 1
    }

    /// Updates a sprite at a given index.
    pub fn update_sprite(&mut self, index: usize, sprite: Sprite) -> Result<(), String> {
        if index < self.sprites.len() {
            self.sprites[index] = sprite;
            Ok(())
        } else {
            Err("Sprite index out of bounds".into())
        }
    }
}
