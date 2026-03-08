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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loaders::bitmap_font::asset::{BitmapCharInfo, BitmapFontAsset};
    use std::collections::HashMap;

    #[test]
    fn test_glyph_atlas_implements_glyph_info_provider() {
        let font_bytes = include_bytes!("../../../test_assets/fonts/test_font.ttf");
        let font = fontdue::Font::from_bytes(font_bytes as &[u8], fontdue::FontSettings::default())
            .expect("parse test font");
        let atlas = GlyphAtlas::generate(&font, 16.0).expect("generate atlas");

        // Access via the trait to confirm the impl works.
        let provider: &dyn GlyphInfoProvider = &atlas;
        let info = provider.glyph_info('A');
        assert!(
            info.is_some(),
            "GlyphAtlas trait impl should return Some for 'A'"
        );
    }

    #[test]
    fn test_bitmap_glyph_atlas_implements_glyph_info_provider() {
        let mut characters = HashMap::new();
        characters.insert(
            'X',
            BitmapCharInfo {
                x: 0,
                y: 0,
                width: 10,
                height: 12,
                x_offset: 0.0,
                y_offset: 0.0,
                x_advance: 11.0,
            },
        );

        let font = BitmapFontAsset {
            characters,
            texture_path: "test.png".to_string(),
            line_height: 16.0,
            base: 14.0,
            kernings: HashMap::new(),
            scale_w: 64,
            scale_h: 64,
            texture_data: None,
        };

        let texture_data = vec![0u8; 64 * 64 * 4];
        let atlas = BitmapGlyphAtlas::new(&font, 64, 64, texture_data);

        // Access via the trait to confirm the impl works.
        let provider: &dyn GlyphInfoProvider = &atlas;
        let info = provider.glyph_info('X');
        assert!(
            info.is_some(),
            "BitmapGlyphAtlas trait impl should return Some for 'X'"
        );
        assert!(
            provider.glyph_info('Z').is_none(),
            "BitmapGlyphAtlas trait impl should return None for missing char"
        );
    }
}
