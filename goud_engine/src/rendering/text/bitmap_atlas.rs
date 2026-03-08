//! Bitmap font atlas adapter and unified [`FontAtlas`] enum.
//!
//! Wraps a [`BitmapFontAsset`] to provide the same [`GlyphInfo`] lookup
//! interface as the TrueType [`GlyphAtlas`], and defines a [`FontAtlas`]
//! enum that unifies both atlas types behind a common query surface.

use std::collections::HashMap;

use crate::assets::loaders::bitmap_font::asset::BitmapFontAsset;
use crate::libs::graphics::backend::render_backend::RenderBackend;
use crate::libs::graphics::backend::types::TextureHandle;

use super::glyph_atlas::{GlyphAtlas, GlyphInfo, UvRect};
use super::rasterizer::GlyphMetrics;

/// Atlas adapter for bitmap (BMFont) fonts.
///
/// Pre-computes a [`GlyphInfo`] map from [`BitmapFontAsset`] character data
/// and atlas dimensions. The GPU texture is created lazily from external
/// image data via [`ensure_gpu_texture`](Self::ensure_gpu_texture).
#[derive(Debug, Clone)]
pub struct BitmapGlyphAtlas {
    /// Per-character glyph info (UV + metrics).
    glyphs: HashMap<char, GlyphInfo>,
    /// Atlas texture width in pixels.
    atlas_width: u32,
    /// Atlas texture height in pixels.
    atlas_height: u32,
    /// Cached GPU texture handle.
    gpu_texture: Option<TextureHandle>,
    /// Raw RGBA8 texture data for GPU upload.
    texture_data: Vec<u8>,
}

impl BitmapGlyphAtlas {
    /// Builds an atlas from a bitmap font asset and its backing texture data.
    ///
    /// # Arguments
    ///
    /// * `font` - The parsed bitmap font asset with character metrics.
    /// * `atlas_width` - Width of the backing texture atlas in pixels.
    /// * `atlas_height` - Height of the backing texture atlas in pixels.
    /// * `texture_data` - RGBA8 pixel data for the atlas texture.
    pub fn new(
        font: &BitmapFontAsset,
        atlas_width: u32,
        atlas_height: u32,
        texture_data: Vec<u8>,
    ) -> Self {
        let w = atlas_width as f32;
        let h = atlas_height as f32;

        let glyphs = font
            .characters
            .iter()
            .map(|(&ch, info)| {
                let glyph_info = GlyphInfo {
                    uv_rect: UvRect {
                        u_min: info.x as f32 / w,
                        v_min: info.y as f32 / h,
                        u_max: (info.x + info.width) as f32 / w,
                        v_max: (info.y + info.height) as f32 / h,
                    },
                    metrics: GlyphMetrics {
                        advance_width: info.x_advance,
                        bearing_x: info.x_offset,
                        bearing_y: info.y_offset,
                        width: info.width as f32,
                        height: info.height as f32,
                    },
                };
                (ch, glyph_info)
            })
            .collect();

        Self {
            glyphs,
            atlas_width,
            atlas_height,
            gpu_texture: None,
            texture_data,
        }
    }

    /// Returns glyph info for the given character, if present.
    pub fn glyph_info(&self, ch: char) -> Option<&GlyphInfo> {
        self.glyphs.get(&ch)
    }

    /// Lazily uploads the atlas texture to the GPU and caches the handle.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to create the texture.
    pub fn ensure_gpu_texture(
        &mut self,
        backend: &mut dyn RenderBackend,
    ) -> Result<TextureHandle, String> {
        use crate::libs::graphics::backend::types::{TextureFilter, TextureFormat, TextureWrap};

        if let Some(handle) = self.gpu_texture {
            return Ok(handle);
        }

        let handle = backend
            .create_texture(
                self.atlas_width,
                self.atlas_height,
                TextureFormat::RGBA8,
                TextureFilter::Linear,
                TextureWrap::ClampToEdge,
                &self.texture_data,
            )
            .map_err(|e| format!("failed to create GPU texture for bitmap atlas: {e}"))?;

        self.gpu_texture = Some(handle);
        Ok(handle)
    }
}

/// Unified font atlas supporting both TrueType and bitmap font sources.
#[derive(Debug, Clone)]
pub enum FontAtlas {
    /// TrueType font atlas (rasterized from vector outlines).
    TrueType(GlyphAtlas),
    /// Bitmap font atlas (pre-rendered sprite sheet).
    Bitmap(BitmapGlyphAtlas),
}

impl FontAtlas {
    /// Returns glyph info for the given character, delegating to the variant.
    pub fn glyph_info(&self, ch: char) -> Option<&GlyphInfo> {
        match self {
            Self::TrueType(atlas) => atlas.glyph_info(ch),
            Self::Bitmap(atlas) => atlas.glyph_info(ch),
        }
    }

    /// Lazily uploads the atlas texture and returns its GPU handle.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to create the texture.
    pub fn ensure_gpu_texture(
        &mut self,
        backend: &mut dyn RenderBackend,
    ) -> Result<TextureHandle, String> {
        match self {
            Self::TrueType(atlas) => atlas.ensure_gpu_texture(backend),
            Self::Bitmap(atlas) => atlas.ensure_gpu_texture(backend),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loaders::bitmap_font::asset::{BitmapCharInfo, BitmapFontAsset};

    fn test_bitmap_font() -> BitmapFontAsset {
        let mut characters = HashMap::new();
        characters.insert(
            'A',
            BitmapCharInfo {
                x: 0,
                y: 0,
                width: 16,
                height: 16,
                x_offset: 0.0,
                y_offset: 0.0,
                x_advance: 18.0,
            },
        );
        characters.insert(
            'B',
            BitmapCharInfo {
                x: 16,
                y: 0,
                width: 14,
                height: 16,
                x_offset: 1.0,
                y_offset: 0.0,
                x_advance: 17.0,
            },
        );

        BitmapFontAsset {
            characters,
            texture_path: "font.png".to_string(),
            line_height: 20.0,
            base: 16.0,
            kernings: HashMap::new(),
            scale_w: 64,
            scale_h: 64,
            texture_data: None,
        }
    }

    #[test]
    fn test_bitmap_atlas_glyph_info() {
        let font = test_bitmap_font();
        let texture_data = vec![0u8; 64 * 64 * 4];
        let atlas = BitmapGlyphAtlas::new(&font, 64, 64, texture_data);

        let info = atlas.glyph_info('A').expect("A should be in atlas");
        assert!(info.uv_rect.u_max > info.uv_rect.u_min);
        assert!(info.uv_rect.v_max > info.uv_rect.v_min);
        assert_eq!(info.metrics.advance_width, 18.0);
    }

    #[test]
    fn test_bitmap_atlas_missing_char() {
        let font = test_bitmap_font();
        let texture_data = vec![0u8; 64 * 64 * 4];
        let atlas = BitmapGlyphAtlas::new(&font, 64, 64, texture_data);

        assert!(atlas.glyph_info('Z').is_none());
    }

    #[test]
    fn test_bitmap_atlas_uv_coordinates() {
        let font = test_bitmap_font();
        let texture_data = vec![0u8; 128 * 128 * 4];
        let atlas = BitmapGlyphAtlas::new(&font, 128, 128, texture_data);

        let info = atlas.glyph_info('A').unwrap();
        assert_eq!(info.uv_rect.u_min, 0.0 / 128.0);
        assert_eq!(info.uv_rect.v_min, 0.0 / 128.0);
        assert_eq!(info.uv_rect.u_max, 16.0 / 128.0);
        assert_eq!(info.uv_rect.v_max, 16.0 / 128.0);
    }

    #[test]
    fn test_font_atlas_enum_truetype_delegates() {
        let font_bytes = include_bytes!("../../../test_assets/fonts/test_font.ttf");
        let font = fontdue::Font::from_bytes(font_bytes as &[u8], fontdue::FontSettings::default())
            .expect("parse");
        let glyph_atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");

        let font_atlas = FontAtlas::TrueType(glyph_atlas);
        assert!(font_atlas.glyph_info('A').is_some());
        assert!(font_atlas.glyph_info('\u{1F600}').is_none());
    }

    #[test]
    fn test_font_atlas_enum_bitmap_delegates() {
        let font = test_bitmap_font();
        let texture_data = vec![0u8; 64 * 64 * 4];
        let bitmap_atlas = BitmapGlyphAtlas::new(&font, 64, 64, texture_data);

        let font_atlas = FontAtlas::Bitmap(bitmap_atlas);
        assert!(font_atlas.glyph_info('A').is_some());
        assert!(font_atlas.glyph_info('Z').is_none());
    }
}
