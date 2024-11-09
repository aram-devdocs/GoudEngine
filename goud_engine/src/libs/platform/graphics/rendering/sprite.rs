// src/sprite.rs

use crate::{types::Rectangle, types::Sprite, types::Texture};
use std::rc::Rc;

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
        source_rect: Rectangle,
    ) -> Sprite {
        Sprite {
            x,
            y,
            scale_x: scale_x,
            scale_y: scale_y,
            dimension_x: dimension_x,
            dimension_y: dimension_y,
            rotation,
            texture,
            source_rect,
        }
    }
}
