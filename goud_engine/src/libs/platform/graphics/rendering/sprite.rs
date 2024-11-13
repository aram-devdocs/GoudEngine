// src/sprite.rs

use crate::{types::Rectangle, types::Sprite};
use std::ffi::c_uint;

impl Sprite {
    /// Creates a new Sprite.
    pub fn new(
        // texture: Rc<Texture>,
        texture_id: c_uint,
        x: f32,
        y: f32,
        z_layer: f32,
        scale_x: f32,
        scale_y: f32,
        dimension_x: f32,
        dimension_y: f32,
        rotation: f32,
        source_rect: Rectangle,
        debug: bool,
    ) -> Sprite {
        Sprite {
            x,
            y,
            z_layer: z_layer,
            scale_x: scale_x,
            scale_y: scale_y,
            dimension_x: dimension_x,
            dimension_y: dimension_y,
            rotation,
            texture_id,
            source_rect,
            debug,
        }
    }

    pub fn check_collision(&self, other: &Sprite) -> bool {
        let self_left = self.x;
        let self_right = self.x + self.dimension_x;
        let self_top = self.y;
        let self_bottom = self.y + self.dimension_y;

        let other_left = other.x;
        let other_right = other.x + other.dimension_x;
        let other_top = other.y;
        let other_bottom = other.y + other.dimension_y;

        self_left < other_right
            && self_right > other_left
            && self_top < other_bottom
            && self_bottom > other_top
    }
}
