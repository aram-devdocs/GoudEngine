//! Texture configuration types: [`TextureSettings`], [`TextureColorSpace`], [`TextureWrapMode`].

use std::fmt;

/// Configuration for texture loading.
///
/// These settings control how textures are decoded and processed during loading.
#[derive(Debug, Clone)]
pub struct TextureSettings {
    /// Whether to flip the texture vertically (Y-axis).
    ///
    /// OpenGL expects textures with the origin at the bottom-left, but most
    /// image formats have the origin at the top-left. Set this to `true` when
    /// loading textures for OpenGL.
    pub flip_vertical: bool,

    /// Color space interpretation of the texture.
    pub color_space: TextureColorSpace,

    /// Wrap mode for texture coordinates outside [0, 1].
    pub wrap_mode: TextureWrapMode,

    /// Whether to generate mipmaps for this texture.
    ///
    /// Note: This setting is informational only. Actual mipmap generation
    /// happens during GPU upload, not during asset loading.
    pub generate_mipmaps: bool,
}

impl Default for TextureSettings {
    fn default() -> Self {
        Self {
            flip_vertical: true, // Default to OpenGL convention
            color_space: TextureColorSpace::Srgb,
            wrap_mode: TextureWrapMode::Repeat,
            generate_mipmaps: true,
        }
    }
}

// =============================================================================
// TextureColorSpace
// =============================================================================

/// Color space of a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextureColorSpace {
    /// Standard RGB (linear color space)
    Linear,
    /// sRGB (gamma-corrected color space) - default for most images
    #[default]
    Srgb,
}

impl TextureColorSpace {
    /// Returns the string representation of the color space.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Srgb => "sRGB",
        }
    }
}

impl fmt::Display for TextureColorSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// TextureWrapMode
// =============================================================================

/// Texture wrap mode for coordinates outside [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextureWrapMode {
    /// Repeat the texture (default)
    #[default]
    Repeat,
    /// Mirror the texture at boundaries
    MirroredRepeat,
    /// Clamp to edge pixels
    ClampToEdge,
    /// Clamp to border color
    ClampToBorder,
}

impl TextureWrapMode {
    /// Returns the string representation of the wrap mode.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Repeat => "Repeat",
            Self::MirroredRepeat => "MirroredRepeat",
            Self::ClampToEdge => "ClampToEdge",
            Self::ClampToBorder => "ClampToBorder",
        }
    }
}

impl fmt::Display for TextureWrapMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
