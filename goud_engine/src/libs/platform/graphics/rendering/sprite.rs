// src/sprite.rs

use crate::{
    types::Rectangle,
    types::{Sprite, SpriteCreateDto, SpriteUpdateDto},
};
use std::ffi::c_uint;

impl Sprite {
    /// Creates a new Sprite.
    pub fn new(
        id: c_uint,
        x: f32,
        y: f32,
        z_layer: i32,
        scale_x: f32,
        scale_y: f32,
        dimension_x: f32,
        dimension_y: f32,
        rotation: f32,
        source_rect: Rectangle,
        texture_id: c_uint,
        debug: bool,
    ) -> Sprite {
        Sprite {
            id,
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

impl SpriteCreateDto {
    /// Creates a new SpriteCreateDto.
    pub fn new(
        x: f32,
        y: f32,
        z_layer: i32,
        scale_x: f32,
        scale_y: f32,
        dimension_x: f32,
        dimension_y: f32,
        rotation: f32,
        source_rect: Rectangle,
        texture_id: c_uint,
        debug: bool,
    ) -> SpriteCreateDto {
        SpriteCreateDto {
            x,
            y,
            z_layer,
            scale_x,
            scale_y,
            dimension_x,
            dimension_y,
            rotation,
            source_rect,
            texture_id,
            debug,
        }
    }
}

impl SpriteUpdateDto {
    /// Creates a new SpriteUpdateDto.
    pub fn new(
        id: c_uint,
        x: f32,
        y: f32,
        z_layer: i32,
        scale_x: f32,
        scale_y: f32,
        dimension_x: f32,
        dimension_y: f32,
        rotation: f32,
        source_rect: Rectangle,
        texture_id: c_uint,
        debug: bool,
    ) -> SpriteUpdateDto {
        SpriteUpdateDto {
            id,
            x,
            y,
            z_layer,
            scale_x,
            scale_y,
            dimension_x,
            dimension_y,
            rotation,
            source_rect,
            texture_id,
            debug,
        }
    }
}
