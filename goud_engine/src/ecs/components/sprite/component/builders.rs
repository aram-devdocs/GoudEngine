use super::Sprite;
use crate::assets::{loaders::SpriteSheetAsset, AssetHandle};
use crate::core::math::{Color, Rect, Vec2};

impl Sprite {
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
    ///     .with_color(Color::rgba(1.0, 0.0, 0.0, 0.5));
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
    #[inline]
    pub fn with_flip_x(mut self, flip: bool) -> Self {
        self.flip_x = flip;
        self
    }

    /// Sets the vertical flip flag.
    ///
    /// When true, the sprite is mirrored along the X-axis.
    #[inline]
    pub fn with_flip_y(mut self, flip: bool) -> Self {
        self.flip_y = flip;
        self
    }

    /// Sets both flip flags at once.
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
    #[inline]
    pub fn with_anchor(mut self, x: f32, y: f32) -> Self {
        self.anchor = Vec2::new(x, y);
        self
    }

    /// Sets the anchor point from a Vec2.
    #[inline]
    pub fn with_anchor_vec(mut self, anchor: Vec2) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets a custom size for the sprite.
    #[inline]
    pub fn with_custom_size(mut self, size: Vec2) -> Self {
        self.custom_size = Some(size);
        self
    }

    /// Removes the custom size, using texture dimensions.
    #[inline]
    pub fn without_custom_size(mut self) -> Self {
        self.custom_size = None;
        self
    }

    /// Gets the effective size of the sprite.
    #[inline]
    pub fn size_or_rect(&self) -> Vec2 {
        if let Some(size) = self.custom_size {
            size
        } else if let Some(rect) = self.source_rect {
            Vec2::new(rect.width, rect.height)
        } else {
            Vec2::zero()
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
