//! Asset category enumeration.

use std::fmt;

/// Categories of assets for engine-level organization.
///
/// Asset types help the engine determine how to handle different assets:
/// - GPU resources may need special upload handling
/// - Audio may be streamed or fully loaded
/// - Data assets may be parsed into structured data
///
/// # FFI Safety
///
/// This enum is `#[repr(u8)]` for stable, FFI-compatible representation.
/// Each variant can be converted to/from an integer for cross-language use.
///
/// # Example
///
/// ```
/// use goud_engine::assets::AssetType;
///
/// let asset_type = AssetType::Texture;
///
/// // FFI: convert to integer
/// let value: u8 = asset_type.into();
/// assert_eq!(value, 1);
///
/// // FFI: convert from integer
/// let recovered = AssetType::try_from(value).unwrap();
/// assert_eq!(recovered, AssetType::Texture);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AssetType {
    /// Custom asset type not in any predefined category.
    #[default]
    Custom = 0,

    /// Image data (PNG, JPG, etc.) uploaded to GPU.
    Texture = 1,

    /// Audio data (WAV, OGG, MP3, etc.).
    Audio = 2,

    /// 3D mesh data (vertices, indices, etc.).
    Mesh = 3,

    /// Shader source or compiled shader program.
    Shader = 4,

    /// Font data (TTF, OTF, etc.).
    Font = 5,

    /// Material definition (properties, textures, shader refs).
    Material = 6,

    /// Animation data (clips, keyframes, etc.).
    Animation = 7,

    /// Tiled map data (TMX, etc.).
    TiledMap = 8,

    /// Prefab/scene definition.
    Prefab = 9,

    /// Configuration data (JSON, TOML, etc.).
    Config = 10,

    /// Generic binary data.
    Binary = 11,

    /// Text content.
    Text = 12,
}

impl AssetType {
    /// Returns all asset type variants.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetType;
    ///
    /// let types = AssetType::all();
    /// assert!(types.contains(&AssetType::Texture));
    /// assert_eq!(types.len(), 13);
    /// ```
    pub const fn all() -> &'static [AssetType] {
        &[
            AssetType::Custom,
            AssetType::Texture,
            AssetType::Audio,
            AssetType::Mesh,
            AssetType::Shader,
            AssetType::Font,
            AssetType::Material,
            AssetType::Animation,
            AssetType::TiledMap,
            AssetType::Prefab,
            AssetType::Config,
            AssetType::Binary,
            AssetType::Text,
        ]
    }

    /// Returns the number of asset type variants.
    pub const fn count() -> usize {
        13
    }

    /// Returns true if this is a GPU-uploadable asset type.
    ///
    /// GPU assets require special handling for upload to graphics memory.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetType;
    ///
    /// assert!(AssetType::Texture.is_gpu_asset());
    /// assert!(AssetType::Mesh.is_gpu_asset());
    /// assert!(!AssetType::Audio.is_gpu_asset());
    /// ```
    #[inline]
    pub const fn is_gpu_asset(&self) -> bool {
        matches!(
            self,
            AssetType::Texture | AssetType::Mesh | AssetType::Shader | AssetType::Font
        )
    }

    /// Returns true if this is a streamable asset type.
    ///
    /// Streamable assets can be partially loaded and played before
    /// fully loaded (e.g., audio, video).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetType;
    ///
    /// assert!(AssetType::Audio.is_streamable());
    /// assert!(!AssetType::Texture.is_streamable());
    /// ```
    #[inline]
    pub const fn is_streamable(&self) -> bool {
        matches!(self, AssetType::Audio)
    }

    /// Returns a human-readable name for this asset type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetType;
    ///
    /// assert_eq!(AssetType::Texture.name(), "Texture");
    /// assert_eq!(AssetType::Audio.name(), "Audio");
    /// ```
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            AssetType::Custom => "Custom",
            AssetType::Texture => "Texture",
            AssetType::Audio => "Audio",
            AssetType::Mesh => "Mesh",
            AssetType::Shader => "Shader",
            AssetType::Font => "Font",
            AssetType::Material => "Material",
            AssetType::Animation => "Animation",
            AssetType::TiledMap => "TiledMap",
            AssetType::Prefab => "Prefab",
            AssetType::Config => "Config",
            AssetType::Binary => "Binary",
            AssetType::Text => "Text",
        }
    }
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<AssetType> for u8 {
    #[inline]
    fn from(value: AssetType) -> u8 {
        value as u8
    }
}

impl TryFrom<u8> for AssetType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AssetType::Custom),
            1 => Ok(AssetType::Texture),
            2 => Ok(AssetType::Audio),
            3 => Ok(AssetType::Mesh),
            4 => Ok(AssetType::Shader),
            5 => Ok(AssetType::Font),
            6 => Ok(AssetType::Material),
            7 => Ok(AssetType::Animation),
            8 => Ok(AssetType::TiledMap),
            9 => Ok(AssetType::Prefab),
            10 => Ok(AssetType::Config),
            11 => Ok(AssetType::Binary),
            12 => Ok(AssetType::Text),
            _ => Err(value),
        }
    }
}
