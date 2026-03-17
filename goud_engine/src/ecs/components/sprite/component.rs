//! Core [`Sprite`] component type and its builder methods.

mod builders;

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
