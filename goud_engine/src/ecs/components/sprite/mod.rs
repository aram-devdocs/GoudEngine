//! Sprite component for 2D rendering.
//!
//! This module provides the [`Sprite`] component for rendering 2D textured quads.
//! Sprites are the fundamental building block for 2D games, representing a
//! rectangular image that can be positioned, rotated, scaled, and tinted.
//!
//! # Component Structure
//!
//! A [`Sprite`] component contains:
//! - **Texture reference**: [`AssetHandle<TextureAsset>`](crate::assets::AssetHandle) pointing to the image data
//! - **Color tint**: RGBA color multiplied with texture colors
//! - **Source rectangle**: Optional UV rect for sprite sheets and atlases
//! - **Flip flags**: Horizontal and vertical mirroring
//! - **Anchor point**: Origin point for rotation and positioning (normalized 0-1)
//! - **Size override**: Optional custom size (defaults to texture size)
//!
//! # Usage with ECS
//!
//! ```
//! use goud_engine::ecs::{World, Component};
//! use goud_engine::ecs::components::{Sprite, Transform2D};
//! use goud_engine::assets::{AssetServer, loaders::TextureAsset};
//! use goud_engine::core::math::{Vec2, Color, Rect};
//!
//! let mut world = World::new();
//! let mut asset_server = AssetServer::new();
//!
//! // Load texture
//! let texture = asset_server.load::<TextureAsset>("player.png");
//!
//! // Create sprite entity
//! let entity = world.spawn_empty();
//! world.insert(entity, Transform2D::from_position(Vec2::new(100.0, 100.0)));
//! world.insert(entity, Sprite::new(texture));
//! ```
//!
//! # Sprite Sheets and Atlases
//!
//! Use `with_source_rect()` to render a portion of a texture:
//!
//! ```
//! use goud_engine::ecs::components::Sprite;
//! use goud_engine::core::math::Rect;
//! # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
//! # let mut asset_server = AssetServer::new();
//! # let texture = asset_server.load::<TextureAsset>("spritesheet.png");
//!
//! let sprite = Sprite::new(texture)
//!     .with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0)) // Top-left 32x32 tile
//!     .with_anchor(0.5, 0.5); // Center anchor
//! ```
//!
//! # Color Tinting
//!
//! Sprites can be tinted by multiplying the texture color with a color value:
//!
//! ```
//! use goud_engine::ecs::components::Sprite;
//! use goud_engine::core::math::Color;
//! # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
//! # let mut asset_server = AssetServer::new();
//! # let texture = asset_server.load::<TextureAsset>("player.png");
//!
//! // Red tint with 50% transparency
//! let sprite = Sprite::new(texture)
//!     .with_color(Color::rgba(1.0, 0.0, 0.0, 0.5));
//! ```
//!
//! # Flipping
//!
//! Sprites can be flipped horizontally or vertically:
//!
//! ```
//! use goud_engine::ecs::components::Sprite;
//! # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
//! # let mut asset_server = AssetServer::new();
//! # let texture = asset_server.load::<TextureAsset>("player.png");
//!
//! let sprite = Sprite::new(texture)
//!     .with_flip_x(true)  // Mirror horizontally
//!     .with_flip_y(false);
//! ```
//!
//! # Anchor Points
//!
//! The anchor point determines the origin for rotation and positioning.
//! Coordinates are normalized (0.0 - 1.0):
//!
//! - `(0.0, 0.0)` = Top-left corner
//! - `(0.5, 0.5)` = Center (default)
//! - `(1.0, 1.0)` = Bottom-right corner
//!
//! ```
//! use goud_engine::ecs::components::Sprite;
//! # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
//! # let mut asset_server = AssetServer::new();
//! # let texture = asset_server.load::<TextureAsset>("player.png");
//!
//! let sprite = Sprite::new(texture)
//!     .with_anchor(0.5, 1.0); // Bottom-center anchor
//! ```
//!
//! # Custom Size
//!
//! By default, sprites render at their texture's pixel size. You can override this:
//!
//! ```
//! use goud_engine::ecs::components::Sprite;
//! use goud_engine::core::math::Vec2;
//! # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
//! # let mut asset_server = AssetServer::new();
//! # let texture = asset_server.load::<TextureAsset>("player.png");
//!
//! let sprite = Sprite::new(texture)
//!     .with_custom_size(Vec2::new(64.0, 64.0)); // Force 64x64 size
//! ```

mod component;

#[cfg(test)]
mod tests;

pub use component::Sprite;
