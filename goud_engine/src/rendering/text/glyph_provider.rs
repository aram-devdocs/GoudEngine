//! Trait for abstracting glyph info lookups across font types.

use super::bitmap_atlas::BitmapGlyphAtlas;
use super::glyph_atlas::{GlyphAtlas, GlyphInfo};

/// Trait for types that provide per-character glyph info lookups.
///
/// Both [`GlyphAtlas`] (TrueType) and [`BitmapGlyphAtlas`] (bitmap) implement
/// this trait, allowing [`layout_text`](super::layout::layout_text) to work
/// with either font source.
pub trait GlyphInfoProvider {
    /// Returns glyph info for the given character, if present.
    fn glyph_info(&self, ch: char) -> Option<&GlyphInfo>;
}

impl GlyphInfoProvider for GlyphAtlas {
    fn glyph_info(&self, ch: char) -> Option<&GlyphInfo> {
        self.glyph_info(ch)
    }
}

impl GlyphInfoProvider for BitmapGlyphAtlas {
    fn glyph_info(&self, ch: char) -> Option<&GlyphInfo> {
        self.glyph_info(ch)
    }
}
