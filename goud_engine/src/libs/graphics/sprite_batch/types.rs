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
#[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(super) struct SpriteVertex {
    /// World-space position (x, y)
    pub(super) position: Vec2,
    /// Texture coordinates (u, v)
    pub(super) tex_coords: Vec2,
    /// Vertex color (r, g, b, a)
    pub(super) color: Color,
}

impl SpriteVertex {
    /// Returns the vertex layout descriptor for GPU.
    #[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
    pub(super) fn layout() -> VertexLayout {
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
#[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
#[derive(Debug, Clone)]
pub(super) struct SpriteInstance {
    /// Entity that owns this sprite
    pub(super) entity: Entity,
    /// Texture handle
    pub(super) texture: AssetHandle<TextureAsset>,
    /// World transform matrix
    pub(super) transform: Mat3x3,
    /// Color tint
    pub(super) color: Color,
    /// Source rectangle (UV coordinates)
    pub(super) source_rect: Option<Rect>,
    /// Sprite size
    pub(super) size: Vec2,
    /// Z-layer for sorting
    pub(super) z_layer: f32,
    /// Flip flags
    pub(super) flip_x: bool,
    pub(super) flip_y: bool,
}

impl SpriteInstance {
    /// Constructs a `SpriteInstance` from ECS components and a computed world transform.
    pub(super) fn from_components(
        entity: Entity,
        sprite: &Sprite,
        transform_matrix: Mat3x3,
        z_layer: f32,
        size: Vec2,
    ) -> Self {
        Self {
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
#[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
#[derive(Debug)]
pub(super) struct SpriteBatchEntry {
    /// Texture used by this batch
    pub(super) texture_handle: AssetHandle<TextureAsset>,
    /// GPU texture handle (resolved from asset handle)
    pub(super) gpu_texture: Option<TextureHandle>,
    /// Start index in vertex buffer
    pub(super) vertex_start: usize,
    /// Number of vertices in this batch
    pub(super) vertex_count: usize,
}
