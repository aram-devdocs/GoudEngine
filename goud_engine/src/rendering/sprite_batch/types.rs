//! Internal data types for the sprite batch renderer.

use crate::assets::{loaders::TextureAsset, AssetHandle};
use crate::core::math::{Color, Rect, Vec2};
use crate::ecs::components::{Mat3x3, Sprite};
use crate::ecs::Entity;
use crate::libs::graphics::backend::types::{
    TextureHandle, VertexAttribute, VertexAttributeType, VertexLayout,
};

/// Vertex data for a single sprite corner.
///
/// Each sprite is composed of 4 vertices forming a quad.
/// The vertex layout is optimized for cache coherency.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SpriteVertex {
    /// World-space position (x, y)
    pub position: Vec2,
    /// Texture coordinates (u, v)
    pub tex_coords: Vec2,
    /// Vertex color (r, g, b, a)
    pub color: Color,
}

impl SpriteVertex {
    /// Returns the vertex layout descriptor for GPU.
    pub fn layout() -> VertexLayout {
        VertexLayout::new(std::mem::size_of::<Self>() as u32)
            .with_attribute(VertexAttribute {
                location: 0,
                attribute_type: VertexAttributeType::Float2,
                offset: 0,
                normalized: false,
            })
            .with_attribute(VertexAttribute {
                location: 1,
                attribute_type: VertexAttributeType::Float2,
                offset: 8,
                normalized: false,
            })
            .with_attribute(VertexAttribute {
                location: 2,
                attribute_type: VertexAttributeType::Float4,
                offset: 16,
                normalized: false,
            })
    }
}

/// Internal representation of a sprite instance for batching.
#[derive(Debug, Clone)]
pub struct SpriteInstance {
    /// Entity that owns this sprite — available in tests for assertion purposes.
    #[cfg(test)]
    pub entity: Entity,
    /// Texture handle
    pub texture: AssetHandle<TextureAsset>,
    /// World transform matrix
    pub transform: Mat3x3,
    /// Color tint
    pub color: Color,
    /// Source rectangle (UV coordinates)
    pub source_rect: Option<Rect>,
    /// Sprite size
    pub size: Vec2,
    /// Z-layer for sorting
    pub z_layer: f32,
    /// Flip horizontally
    pub flip_x: bool,
    /// Flip vertically
    pub flip_y: bool,
}

impl SpriteInstance {
    /// Constructs a `SpriteInstance` from ECS components and a computed world transform.
    pub fn from_components(
        #[cfg(test)] entity: Entity,
        #[cfg(not(test))] _entity: Entity,
        sprite: &Sprite,
        transform_matrix: Mat3x3,
        z_layer: f32,
        size: Vec2,
    ) -> Self {
        Self {
            #[cfg(test)]
            entity,
            transform: transform_matrix,
            texture: sprite.texture,
            color: sprite.color,
            source_rect: sprite.source_rect,
            size,
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
            z_layer,
        }
    }
}

/// A single draw batch for sprites sharing the same texture.
#[derive(Debug)]
pub struct SpriteBatchEntry {
    /// Texture used by this batch (will be passed to the draw call when the render stage is wired up)
    pub _texture_handle: AssetHandle<TextureAsset>,
    /// GPU texture handle (resolved from asset handle)
    pub gpu_texture: Option<TextureHandle>,
    /// Start index in vertex buffer
    pub vertex_start: usize,
    /// Number of vertices in this batch
    pub vertex_count: usize,
}
