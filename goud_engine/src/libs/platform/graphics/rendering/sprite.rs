// src/sprite.rs

use crate::{types::Rectangle, types::Sprite, types::Texture};
use cgmath::Vector2;
use std::rc::Rc;

use super::texture;

impl Sprite {
    /// Creates a new Sprite.
    pub fn new(
        texture: Rc<Texture>,
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
        dimension_x: f32,
        dimension_y: f32,
        rotation: f32,
        source_rect: Option<Rectangle>,
    ) -> Sprite {
        Sprite {
            x,
            y,
            scale_x: Some(scale_x),
            scale_y: Some(scale_y),
            dimension_x: Some(dimension_x),
            dimension_y: Some(dimension_y),
            rotation,
            texture,
            source_rect,
        }
    }
}
