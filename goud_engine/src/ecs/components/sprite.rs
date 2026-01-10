//! Sprite component for 2D rendering.
//!
//! This module provides the [`Sprite`] component for rendering 2D textured quads.
//! Sprites are the fundamental building block for 2D games, representing a
//! rectangular image that can be positioned, rotated, scaled, and tinted.
//!
//! # Component Structure
//!
//! A [`Sprite`] component contains:
//! - **Texture reference**: [`AssetHandle<TextureAsset>`] pointing to the image data
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

use crate::assets::{loaders::TextureAsset, AssetHandle};
use crate::core::math::{Color, Rect, Vec2};
use crate::ecs::Component;

// =============================================================================
// Sprite Component
// =============================================================================

/// A 2D sprite component for rendering textured quads.
///
/// The `Sprite` component defines how a texture should be rendered in 2D space.
/// It must be paired with a [`Transform2D`](crate::ecs::components::Transform2D)
/// or [`GlobalTransform2D`](crate::ecs::components::GlobalTransform2D) component
/// to define the sprite's position, rotation, and scale.
///
/// # Fields
///
/// - `texture`: Handle to the texture asset to render
/// - `color`: Color tint multiplied with texture pixels (default: white)
/// - `source_rect`: Optional UV rectangle for sprite sheets (default: full texture)
/// - `flip_x`: Flip the sprite horizontally (default: false)
/// - `flip_y`: Flip the sprite vertically (default: false)
/// - `anchor`: Normalized anchor point for rotation/positioning (default: center)
/// - `custom_size`: Optional override for sprite size (default: texture size)
///
/// # Examples
///
/// ## Basic Sprite
///
/// ```
/// use goud_engine::ecs::components::Sprite;
/// use goud_engine::assets::{AssetServer, loaders::TextureAsset};
///
/// let mut asset_server = AssetServer::new();
/// let texture = asset_server.load::<TextureAsset>("player.png");
///
/// let sprite = Sprite::new(texture);
/// ```
///
/// ## Sprite with Custom Properties
///
/// ```
/// use goud_engine::ecs::components::Sprite;
/// use goud_engine::core::math::{Color, Vec2};
/// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
/// # let mut asset_server = AssetServer::new();
/// # let texture = asset_server.load::<TextureAsset>("player.png");
///
/// let sprite = Sprite::new(texture)
///     .with_color(Color::rgba(1.0, 0.5, 0.5, 0.8))
///     .with_flip_x(true)
///     .with_anchor(0.5, 1.0)
///     .with_custom_size(Vec2::new(64.0, 64.0));
/// ```
///
/// ## Sprite Sheet Frame
///
/// ```
/// use goud_engine::ecs::components::Sprite;
/// use goud_engine::core::math::Rect;
/// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
/// # let mut asset_server = AssetServer::new();
/// # let texture = asset_server.load::<TextureAsset>("spritesheet.png");
///
/// // Extract a 32x32 tile from the sprite sheet
/// let sprite = Sprite::new(texture)
///     .with_source_rect(Rect::new(64.0, 32.0, 32.0, 32.0));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Sprite {
    /// Handle to the texture asset to render.
    pub texture: AssetHandle<TextureAsset>,

    /// Color tint multiplied with texture pixels.
    ///
    /// Each component is in range [0.0, 1.0]. White (1, 1, 1, 1) renders
    /// the texture unmodified. RGB values tint the color, alpha controls
    /// transparency.
    pub color: Color,

    /// Optional source rectangle for sprite sheets and atlases.
    ///
    /// If `None`, the entire texture is rendered. If `Some`, only the
    /// specified rectangle (in pixel coordinates) is rendered.
    ///
    /// For normalized UV coordinates, multiply by texture dimensions.
    pub source_rect: Option<Rect>,

    /// Flip the sprite horizontally.
    ///
    /// When true, the texture is mirrored along the Y-axis.
    pub flip_x: bool,

    /// Flip the sprite vertically.
    ///
    /// When true, the texture is mirrored along the X-axis.
    pub flip_y: bool,

    /// Normalized anchor point for rotation and positioning.
    ///
    /// Coordinates are in range [0.0, 1.0]:
    /// - `(0.0, 0.0)` = Top-left corner
    /// - `(0.5, 0.5)` = Center (default)
    /// - `(1.0, 1.0)` = Bottom-right corner
    ///
    /// The anchor point is the origin for rotation and the point that aligns
    /// with the entity's Transform2D position.
    pub anchor: Vec2,

    /// Optional custom size override.
    ///
    /// If `None`, the sprite renders at the texture's pixel dimensions
    /// (or source_rect dimensions if specified). If `Some`, the sprite
    /// is scaled to this size.
    pub custom_size: Option<Vec2>,
}

impl Sprite {
    /// Creates a new sprite with default settings.
    ///
    /// The sprite will render the entire texture with:
    /// - White color tint (no modification)
    /// - No source rectangle (full texture)
    /// - No flipping
    /// - Center anchor point (0.5, 0.5)
    /// - No custom size (uses texture dimensions)
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// let sprite = Sprite::new(texture);
    /// ```
    #[inline]
    pub fn new(texture: AssetHandle<TextureAsset>) -> Self {
        Self {
            texture,
            color: Color::WHITE,
            source_rect: None,
            flip_x: false,
            flip_y: false,
            anchor: Vec2::new(0.5, 0.5),
            custom_size: None,
        }
    }

    /// Sets the color tint for this sprite.
    ///
    /// The color is multiplied with each texture pixel. Use white (1, 1, 1, 1)
    /// for no tinting.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// use goud_engine::core::math::Color;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// let sprite = Sprite::new(texture)
    ///     .with_color(Color::rgba(1.0, 0.0, 0.0, 0.5)); // Red, 50% transparent
    /// ```
    #[inline]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the source rectangle for sprite sheet rendering.
    ///
    /// The rectangle is specified in pixel coordinates relative to the
    /// top-left corner of the texture.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// use goud_engine::core::math::Rect;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("spritesheet.png");
    ///
    /// // Extract a 32x32 tile at position (64, 32)
    /// let sprite = Sprite::new(texture)
    ///     .with_source_rect(Rect::new(64.0, 32.0, 32.0, 32.0));
    /// ```
    #[inline]
    pub fn with_source_rect(mut self, rect: Rect) -> Self {
        self.source_rect = Some(rect);
        self
    }

    /// Removes the source rectangle, rendering the full texture.
    ///
    /// # Example
    ///
    /// ```
    /// # use goud_engine::ecs::components::Sprite;
    /// # use goud_engine::core::math::Rect;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("spritesheet.png");
    /// let mut sprite = Sprite::new(texture)
    ///     .with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0));
    ///
    /// sprite = sprite.without_source_rect();
    /// assert!(sprite.source_rect.is_none());
    /// ```
    #[inline]
    pub fn without_source_rect(mut self) -> Self {
        self.source_rect = None;
        self
    }

    /// Sets the horizontal flip flag.
    ///
    /// When true, the sprite is mirrored along the Y-axis.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// let sprite = Sprite::new(texture).with_flip_x(true);
    /// ```
    #[inline]
    pub fn with_flip_x(mut self, flip: bool) -> Self {
        self.flip_x = flip;
        self
    }

    /// Sets the vertical flip flag.
    ///
    /// When true, the sprite is mirrored along the X-axis.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// let sprite = Sprite::new(texture).with_flip_y(true);
    /// ```
    #[inline]
    pub fn with_flip_y(mut self, flip: bool) -> Self {
        self.flip_y = flip;
        self
    }

    /// Sets both flip flags at once.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// let sprite = Sprite::new(texture).with_flip(true, true);
    /// ```
    #[inline]
    pub fn with_flip(mut self, flip_x: bool, flip_y: bool) -> Self {
        self.flip_x = flip_x;
        self.flip_y = flip_y;
        self
    }

    /// Sets the anchor point with individual coordinates.
    ///
    /// Coordinates are normalized in range [0.0, 1.0]:
    /// - `(0.0, 0.0)` = Top-left
    /// - `(0.5, 0.5)` = Center
    /// - `(1.0, 1.0)` = Bottom-right
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// // Bottom-center anchor for ground-aligned sprites
    /// let sprite = Sprite::new(texture).with_anchor(0.5, 1.0);
    /// ```
    #[inline]
    pub fn with_anchor(mut self, x: f32, y: f32) -> Self {
        self.anchor = Vec2::new(x, y);
        self
    }

    /// Sets the anchor point from a Vec2.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// use goud_engine::core::math::Vec2;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// let sprite = Sprite::new(texture)
    ///     .with_anchor_vec(Vec2::new(0.5, 1.0));
    /// ```
    #[inline]
    pub fn with_anchor_vec(mut self, anchor: Vec2) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets a custom size for the sprite.
    ///
    /// When set, the sprite is scaled to this size regardless of the
    /// texture dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// use goud_engine::core::math::Vec2;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// // Force sprite to 64x64 size
    /// let sprite = Sprite::new(texture)
    ///     .with_custom_size(Vec2::new(64.0, 64.0));
    /// ```
    #[inline]
    pub fn with_custom_size(mut self, size: Vec2) -> Self {
        self.custom_size = Some(size);
        self
    }

    /// Removes the custom size, using texture dimensions.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// use goud_engine::core::math::Vec2;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// let mut sprite = Sprite::new(texture)
    ///     .with_custom_size(Vec2::new(64.0, 64.0));
    ///
    /// sprite = sprite.without_custom_size();
    /// assert!(sprite.custom_size.is_none());
    /// ```
    #[inline]
    pub fn without_custom_size(mut self) -> Self {
        self.custom_size = None;
        self
    }

    /// Gets the effective size of the sprite.
    ///
    /// Returns the custom size if set, otherwise the source rect size if set,
    /// otherwise falls back to a default size (requires texture dimensions).
    ///
    /// For actual rendering, you'll need to query the texture asset to get
    /// its dimensions when custom_size and source_rect are both None.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// use goud_engine::core::math::{Vec2, Rect};
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// let sprite = Sprite::new(texture)
    ///     .with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0));
    ///
    /// let size = sprite.size_or_rect();
    /// assert_eq!(size, Vec2::new(32.0, 32.0));
    /// ```
    #[inline]
    pub fn size_or_rect(&self) -> Vec2 {
        if let Some(size) = self.custom_size {
            size
        } else if let Some(rect) = self.source_rect {
            Vec2::new(rect.width, rect.height)
        } else {
            Vec2::zero() // Caller must query texture dimensions
        }
    }

    /// Returns true if the sprite has a source rectangle set.
    #[inline]
    pub fn has_source_rect(&self) -> bool {
        self.source_rect.is_some()
    }

    /// Returns true if the sprite has a custom size set.
    #[inline]
    pub fn has_custom_size(&self) -> bool {
        self.custom_size.is_some()
    }

    /// Returns true if the sprite is flipped on either axis.
    #[inline]
    pub fn is_flipped(&self) -> bool {
        self.flip_x || self.flip_y
    }
}

// Implement Component trait so Sprite can be used in the ECS
impl Component for Sprite {}

// =============================================================================
// Default Implementation
// =============================================================================

impl Default for Sprite {
    /// Creates a sprite with an invalid texture handle.
    ///
    /// This is primarily useful for deserialization or when the texture
    /// will be set later. The sprite will not render correctly until a
    /// valid texture handle is assigned.
    fn default() -> Self {
        Self {
            texture: AssetHandle::INVALID,
            color: Color::WHITE,
            source_rect: None,
            flip_x: false,
            flip_y: false,
            anchor: Vec2::new(0.5, 0.5),
            custom_size: None,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a valid handle for testing
    fn dummy_handle() -> AssetHandle<TextureAsset> {
        AssetHandle::new(1, 1)
    }

    #[test]
    fn test_sprite_new() {
        let handle = dummy_handle();
        let sprite = Sprite::new(handle);

        assert_eq!(sprite.texture, handle);
        assert_eq!(sprite.color, Color::WHITE);
        assert_eq!(sprite.source_rect, None);
        assert_eq!(sprite.flip_x, false);
        assert_eq!(sprite.flip_y, false);
        assert_eq!(sprite.anchor, Vec2::new(0.5, 0.5));
        assert_eq!(sprite.custom_size, None);
    }

    #[test]
    fn test_sprite_default() {
        let sprite = Sprite::default();

        assert_eq!(sprite.texture, AssetHandle::INVALID);
        assert_eq!(sprite.color, Color::WHITE);
        assert_eq!(sprite.anchor, Vec2::new(0.5, 0.5));
    }

    #[test]
    fn test_sprite_with_color() {
        let handle = dummy_handle();
        let red = Color::rgba(1.0, 0.0, 0.0, 0.5);
        let sprite = Sprite::new(handle).with_color(red);

        assert_eq!(sprite.color, red);
    }

    #[test]
    fn test_sprite_with_source_rect() {
        let handle = dummy_handle();
        let rect = Rect::new(10.0, 20.0, 32.0, 32.0);
        let sprite = Sprite::new(handle).with_source_rect(rect);

        assert_eq!(sprite.source_rect, Some(rect));
        assert!(sprite.has_source_rect());
    }

    #[test]
    fn test_sprite_without_source_rect() {
        let handle = dummy_handle();
        let rect = Rect::new(10.0, 20.0, 32.0, 32.0);
        let sprite = Sprite::new(handle)
            .with_source_rect(rect)
            .without_source_rect();

        assert_eq!(sprite.source_rect, None);
        assert!(!sprite.has_source_rect());
    }

    #[test]
    fn test_sprite_with_flip_x() {
        let handle = dummy_handle();
        let sprite = Sprite::new(handle).with_flip_x(true);

        assert_eq!(sprite.flip_x, true);
        assert_eq!(sprite.flip_y, false);
        assert!(sprite.is_flipped());
    }

    #[test]
    fn test_sprite_with_flip_y() {
        let handle = dummy_handle();
        let sprite = Sprite::new(handle).with_flip_y(true);

        assert_eq!(sprite.flip_x, false);
        assert_eq!(sprite.flip_y, true);
        assert!(sprite.is_flipped());
    }

    #[test]
    fn test_sprite_with_flip() {
        let handle = dummy_handle();
        let sprite = Sprite::new(handle).with_flip(true, true);

        assert_eq!(sprite.flip_x, true);
        assert_eq!(sprite.flip_y, true);
        assert!(sprite.is_flipped());
    }

    #[test]
    fn test_sprite_with_anchor() {
        let handle = dummy_handle();
        let sprite = Sprite::new(handle).with_anchor(0.0, 1.0);

        assert_eq!(sprite.anchor, Vec2::new(0.0, 1.0));
    }

    #[test]
    fn test_sprite_with_anchor_vec() {
        let handle = dummy_handle();
        let anchor = Vec2::new(0.25, 0.75);
        let sprite = Sprite::new(handle).with_anchor_vec(anchor);

        assert_eq!(sprite.anchor, anchor);
    }

    #[test]
    fn test_sprite_with_custom_size() {
        let handle = dummy_handle();
        let size = Vec2::new(64.0, 64.0);
        let sprite = Sprite::new(handle).with_custom_size(size);

        assert_eq!(sprite.custom_size, Some(size));
        assert!(sprite.has_custom_size());
    }

    #[test]
    fn test_sprite_without_custom_size() {
        let handle = dummy_handle();
        let size = Vec2::new(64.0, 64.0);
        let sprite = Sprite::new(handle)
            .with_custom_size(size)
            .without_custom_size();

        assert_eq!(sprite.custom_size, None);
        assert!(!sprite.has_custom_size());
    }

    #[test]
    fn test_sprite_size_or_rect_custom() {
        let handle = dummy_handle();
        let custom_size = Vec2::new(100.0, 100.0);
        let sprite = Sprite::new(handle)
            .with_custom_size(custom_size)
            .with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0));

        // Custom size takes precedence
        assert_eq!(sprite.size_or_rect(), custom_size);
    }

    #[test]
    fn test_sprite_size_or_rect_source() {
        let handle = dummy_handle();
        let sprite = Sprite::new(handle).with_source_rect(Rect::new(0.0, 0.0, 32.0, 48.0));

        // Source rect size is used when no custom size
        assert_eq!(sprite.size_or_rect(), Vec2::new(32.0, 48.0));
    }

    #[test]
    fn test_sprite_size_or_rect_none() {
        let handle = dummy_handle();
        let sprite = Sprite::new(handle);

        // Returns zero when neither is set (caller should query texture)
        assert_eq!(sprite.size_or_rect(), Vec2::zero());
    }

    #[test]
    fn test_sprite_is_flipped() {
        let handle = dummy_handle();

        let sprite1 = Sprite::new(handle);
        assert!(!sprite1.is_flipped());

        let sprite2 = Sprite::new(handle).with_flip_x(true);
        assert!(sprite2.is_flipped());

        let sprite3 = Sprite::new(handle).with_flip_y(true);
        assert!(sprite3.is_flipped());

        let sprite4 = Sprite::new(handle).with_flip(true, true);
        assert!(sprite4.is_flipped());
    }

    #[test]
    fn test_sprite_builder_chain() {
        let handle = dummy_handle();
        let sprite = Sprite::new(handle)
            .with_color(Color::RED)
            .with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0))
            .with_flip(true, false)
            .with_anchor(0.5, 1.0)
            .with_custom_size(Vec2::new(64.0, 64.0));

        assert_eq!(sprite.color, Color::RED);
        assert_eq!(sprite.source_rect, Some(Rect::new(0.0, 0.0, 32.0, 32.0)));
        assert_eq!(sprite.flip_x, true);
        assert_eq!(sprite.flip_y, false);
        assert_eq!(sprite.anchor, Vec2::new(0.5, 1.0));
        assert_eq!(sprite.custom_size, Some(Vec2::new(64.0, 64.0)));
    }

    #[test]
    fn test_sprite_clone() {
        let handle = dummy_handle();
        let sprite1 = Sprite::new(handle).with_color(Color::BLUE);
        let sprite2 = sprite1.clone();

        assert_eq!(sprite1, sprite2);
    }

    #[test]
    fn test_sprite_is_component() {
        // Compile-time check that Sprite implements Component
        fn assert_component<T: Component>() {}
        assert_component::<Sprite>();
    }

    #[test]
    fn test_sprite_debug() {
        let handle = dummy_handle();
        let sprite = Sprite::new(handle);
        let debug_str = format!("{sprite:?}");

        assert!(debug_str.contains("Sprite"));
    }

    #[test]
    fn test_sprite_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Sprite>();
    }
}
