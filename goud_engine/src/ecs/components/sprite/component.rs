//! Core [`Sprite`] component type and its builder methods.

use crate::assets::{
    loaders::{SpriteSheetAsset, TextureAsset},
    AssetHandle,
};
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
/// - `z_layer`: Explicit render-order layer (default: 0)
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    /// Handle to the texture asset to render.
    // TODO(#219): Resolve texture_path back to a handle on deserialization
    #[serde(skip)]
    pub texture: AssetHandle<TextureAsset>,

    /// Optional path to the texture asset for serialization.
    ///
    /// When a scene is serialized the handle cannot be persisted, but
    /// this path string can. At load time a higher-level system
    /// resolves the path back to an [`AssetHandle`].
    #[serde(default)]
    pub texture_path: Option<String>,

    /// Optional sprite-sheet descriptor handle for atlas-backed sprites.
    #[serde(skip)]
    pub sprite_sheet: AssetHandle<SpriteSheetAsset>,

    /// Optional path to the sprite-sheet descriptor for serialization.
    #[serde(default)]
    pub sprite_sheet_path: Option<String>,

    /// Optional named frame inside the sprite sheet.
    #[serde(default)]
    pub sprite_frame: Option<String>,

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

    /// Explicit render-order layer for 2D batching.
    ///
    /// Lower values draw first, higher values draw later.
    pub z_layer: i32,

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
            texture_path: None,
            sprite_sheet: AssetHandle::INVALID,
            sprite_sheet_path: None,
            sprite_frame: None,
            color: Color::WHITE,
            source_rect: None,
            flip_x: false,
            flip_y: false,
            z_layer: 0,
            anchor: Vec2::new(0.5, 0.5),
            custom_size: None,
        }
    }

    /// Sets the texture asset path for serialization.
    ///
    /// The path is stored alongside the sprite so it survives
    /// serialization. A higher-level system is responsible for
    /// resolving it back to an [`AssetHandle`] after deserialization.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Sprite;
    /// # use goud_engine::assets::{AssetServer, loaders::TextureAsset};
    /// # let mut asset_server = AssetServer::new();
    /// # let texture = asset_server.load::<TextureAsset>("player.png");
    ///
    /// let sprite = Sprite::new(texture)
    ///     .with_texture_path("player.png");
    /// assert_eq!(sprite.texture_path.as_deref(), Some("player.png"));
    /// ```
    #[inline]
    pub fn with_texture_path(mut self, path: impl Into<String>) -> Self {
        self.texture_path = Some(path.into());
        self
    }

    /// Creates a sprite whose texture and source rectangle are resolved from a
    /// sprite-sheet descriptor at render time.
    #[inline]
    pub fn from_sprite_sheet(
        sheet: AssetHandle<SpriteSheetAsset>,
        frame: impl Into<String>,
    ) -> Self {
        Self::default().with_sprite_sheet(sheet, frame)
    }

    /// Sets the sprite-sheet handle and named frame used for atlas resolution.
    #[inline]
    pub fn with_sprite_sheet(
        mut self,
        sheet: AssetHandle<SpriteSheetAsset>,
        frame: impl Into<String>,
    ) -> Self {
        self.sprite_sheet = sheet;
        self.sprite_frame = Some(frame.into());
        self
    }

    /// Sets the sprite-sheet path and named frame used for serialized scenes.
    #[inline]
    pub fn with_sprite_sheet_path(
        mut self,
        path: impl Into<String>,
        frame: impl Into<String>,
    ) -> Self {
        self.sprite_sheet_path = Some(path.into());
        self.sprite_frame = Some(frame.into());
        self
    }

    /// Sets or replaces only the named frame for a sprite-sheet-backed sprite.
    #[inline]
    pub fn with_sprite_frame(mut self, frame: impl Into<String>) -> Self {
        self.sprite_frame = Some(frame.into());
        self
    }

    /// Removes sprite-sheet-backed atlas resolution from this sprite.
    #[inline]
    pub fn without_sprite_sheet(mut self) -> Self {
        self.sprite_sheet = AssetHandle::INVALID;
        self.sprite_sheet_path = None;
        self.sprite_frame = None;
        self
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

    /// Sets the explicit render-order layer.
    #[inline]
    pub fn with_z_layer(mut self, z_layer: i32) -> Self {
        self.z_layer = z_layer;
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

    /// Returns true if the sprite is configured to resolve from a sprite sheet.
    #[inline]
    pub fn has_sprite_sheet(&self) -> bool {
        (self.sprite_sheet.is_valid() || self.sprite_sheet_path.is_some())
            && self.sprite_frame.is_some()
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
            texture_path: None,
            sprite_sheet: AssetHandle::INVALID,
            sprite_sheet_path: None,
            sprite_frame: None,
            color: Color::WHITE,
            source_rect: None,
            flip_x: false,
            flip_y: false,
            z_layer: 0,
            anchor: Vec2::new(0.5, 0.5),
            custom_size: None,
        }
    }
}
