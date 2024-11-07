// src/sprite.rs

use crate::libs::platform::graphics::rendering::texture::Texture;
use cgmath::Vector2;
use std::rc::Rc;

/// Rectangle
///
/// Represents a rectangle, typically used for spritesheet source rectangles.
#[derive(Debug, Copy, Clone)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Sprite
///
/// Represents a 2D sprite with position, scale, rotation, and optional source rectangle.
#[derive(Debug, Clone)]
pub struct Sprite {
    pub position: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub dimensions: Vector2<f32>,
    pub rotation: f32,
    pub texture: Rc<Texture>,
    pub source_rect: Option<Rectangle>,
}

impl Sprite {
    /// Creates a new Sprite.
    pub fn new(
        texture: Rc<Texture>,
        position: Vector2<f32>,
        scale: Vector2<f32>,
        dimensions: Vector2<f32>,
        rotation: f32,
        source_rect: Option<Rectangle>,
    ) -> Sprite {
        Sprite {
            position,
            scale,
            dimensions,
            rotation,
            texture,
            source_rect,
        }
    }
}
