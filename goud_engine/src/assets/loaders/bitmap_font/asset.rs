//! [`BitmapFontAsset`] definition.

use std::collections::HashMap;

use crate::assets::{Asset, AssetType};

/// Per-character information in a bitmap font.
#[derive(Debug, Clone)]
pub struct BitmapCharInfo {
    /// X position of the character in the texture (pixels).
    pub x: u32,
    /// Y position of the character in the texture (pixels).
    pub y: u32,
    /// Width of the character in the texture (pixels).
    pub width: u32,
    /// Height of the character in the texture (pixels).
    pub height: u32,
    /// Horizontal offset when rendering (pixels).
    pub x_offset: f32,
    /// Vertical offset when rendering (pixels).
    pub y_offset: f32,
    /// Horizontal advance after rendering this character (pixels).
    pub x_advance: f32,
}

/// A bitmap font asset loaded from BMFont text format (.fnt).
///
/// Contains character metrics, kerning pairs, and a reference to the
/// backing texture atlas.
#[derive(Debug, Clone)]
pub struct BitmapFontAsset {
    /// Per-character glyph information.
    pub characters: HashMap<char, BitmapCharInfo>,
    /// Path to the texture atlas for this font.
    pub texture_path: String,
    /// Line height in pixels.
    pub line_height: f32,
    /// Baseline offset from the top of the line.
    pub base: f32,
    /// Kerning adjustments for character pairs.
    pub kernings: HashMap<(char, char), f32>,
    /// Atlas texture width in pixels (from BMFont `scaleW`).
    pub scale_w: u32,
    /// Atlas texture height in pixels (from BMFont `scaleH`).
    pub scale_h: u32,
    /// Pre-loaded RGBA8 texture data, if available.
    ///
    /// Populated when the bitmap font texture is loaded alongside the
    /// `.fnt` file. When `None`, a transparent placeholder is used.
    pub texture_data: Option<Vec<u8>>,
}

impl BitmapFontAsset {
    /// Returns the character info for the given character, if present.
    pub fn char_info(&self, ch: char) -> Option<&BitmapCharInfo> {
        self.characters.get(&ch)
    }

    /// Returns the kerning adjustment for a character pair.
    pub fn kerning(&self, first: char, second: char) -> f32 {
        self.kernings.get(&(first, second)).copied().unwrap_or(0.0)
    }

    /// Returns the number of characters in this font.
    pub fn char_count(&self) -> usize {
        self.characters.len()
    }
}

impl Asset for BitmapFontAsset {
    fn asset_type_name() -> &'static str {
        "BitmapFont"
    }

    fn asset_type() -> AssetType {
        AssetType::Font
    }

    fn extensions() -> &'static [&'static str] {
        &["fnt"]
    }
}
