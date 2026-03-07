//! Glyph atlas generation.
//!
//! Rasterizes printable ASCII glyphs and packs them into a single RGBA8
//! texture atlas using row-based bin packing.

use std::collections::HashMap;

use super::rasterizer::{rasterize_glyphs, GlyphMetrics};

/// UV rectangle describing a glyph's position within the atlas texture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvRect {
    /// Left edge in UV space (0.0..1.0).
    pub u_min: f32,
    /// Top edge in UV space (0.0..1.0).
    pub v_min: f32,
    /// Right edge in UV space (0.0..1.0).
    pub u_max: f32,
    /// Bottom edge in UV space (0.0..1.0).
    pub v_max: f32,
}

/// Glyph information stored in the atlas: UV coordinates and metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphInfo {
    /// UV rectangle for this glyph in the atlas texture.
    pub uv_rect: UvRect,
    /// Rasterization metrics for this glyph.
    pub metrics: GlyphMetrics,
}

/// A packed glyph atlas containing an RGBA8 texture and per-glyph metadata.
#[derive(Debug, Clone)]
pub struct GlyphAtlas {
    /// RGBA8 pixel data (4 bytes per pixel).
    texture_data: Vec<u8>,
    /// Atlas texture width in pixels.
    width: u32,
    /// Atlas texture height in pixels.
    height: u32,
    /// Per-character glyph info (UV + metrics).
    glyphs: HashMap<char, GlyphInfo>,
}

/// The range of printable ASCII characters (space through tilde).
const PRINTABLE_ASCII_START: u8 = 32;
const PRINTABLE_ASCII_END: u8 = 126;

/// 1-pixel padding between glyphs to avoid texture bleeding.
const GLYPH_PADDING: u32 = 1;

/// Maximum atlas dimension to prevent runaway allocation.
const MAX_ATLAS_SIZE: u32 = 4096;

impl GlyphAtlas {
    /// Generates an atlas for printable ASCII (32..=126) at the given size.
    ///
    /// The atlas is RGBA8 (white RGB + alpha from glyph coverage).
    /// Dimensions start at 256x256 and grow to the next power-of-two as needed.
    ///
    /// # Errors
    ///
    /// Returns an error if the glyphs cannot fit within the maximum atlas size.
    pub fn generate(font: &fontdue::Font, size_px: f32) -> Result<Self, String> {
        let chars: Vec<char> = (PRINTABLE_ASCII_START..=PRINTABLE_ASCII_END)
            .map(|b| b as char)
            .collect();

        let rasterized = rasterize_glyphs(font, size_px, &chars);

        // Try increasing atlas sizes until all glyphs fit.
        let mut atlas_size: u32 = 256;
        loop {
            if atlas_size > MAX_ATLAS_SIZE {
                return Err(format!(
                    "Glyphs do not fit in maximum atlas size ({0}x{0})",
                    MAX_ATLAS_SIZE
                ));
            }

            match Self::try_pack(&rasterized, atlas_size, atlas_size) {
                Some((texture_data, glyphs)) => {
                    return Ok(Self {
                        texture_data,
                        width: atlas_size,
                        height: atlas_size,
                        glyphs,
                    });
                }
                None => {
                    atlas_size = (atlas_size + 1).next_power_of_two();
                }
            }
        }
    }

    /// Attempts row-based bin packing of rasterized glyphs into an atlas of
    /// the given dimensions. Returns `None` if the glyphs don't fit.
    fn try_pack(
        rasterized: &[(char, super::rasterizer::RasterizedGlyph)],
        atlas_w: u32,
        atlas_h: u32,
    ) -> Option<(Vec<u8>, HashMap<char, GlyphInfo>)> {
        let mut texture_data = vec![0u8; (atlas_w * atlas_h * 4) as usize];
        let mut glyphs = HashMap::new();

        let mut cursor_x: u32 = GLYPH_PADDING;
        let mut cursor_y: u32 = GLYPH_PADDING;
        let mut row_height: u32 = 0;

        for (ch, glyph) in rasterized {
            // Zero-size glyphs (e.g., space) get a degenerate UV rect.
            if glyph.width == 0 || glyph.height == 0 {
                glyphs.insert(
                    *ch,
                    GlyphInfo {
                        uv_rect: UvRect {
                            u_min: 0.0,
                            v_min: 0.0,
                            u_max: 0.0,
                            v_max: 0.0,
                        },
                        metrics: glyph.metrics,
                    },
                );
                continue;
            }

            // Advance to next row if this glyph doesn't fit on the current one.
            if cursor_x + glyph.width + GLYPH_PADDING > atlas_w {
                cursor_x = GLYPH_PADDING;
                cursor_y += row_height + GLYPH_PADDING;
                row_height = 0;
            }

            // Check vertical overflow.
            if cursor_y + glyph.height + GLYPH_PADDING > atlas_h {
                return None;
            }

            // Blit glyph bitmap into atlas as RGBA8 (white + alpha).
            for gy in 0..glyph.height {
                for gx in 0..glyph.width {
                    let src_idx = (gy * glyph.width + gx) as usize;
                    let dst_x = cursor_x + gx;
                    let dst_y = cursor_y + gy;
                    let dst_idx = ((dst_y * atlas_w + dst_x) * 4) as usize;

                    let alpha = glyph.bitmap[src_idx];
                    texture_data[dst_idx] = 255; // R
                    texture_data[dst_idx + 1] = 255; // G
                    texture_data[dst_idx + 2] = 255; // B
                    texture_data[dst_idx + 3] = alpha; // A
                }
            }

            // Record UV coordinates.
            let u_min = cursor_x as f32 / atlas_w as f32;
            let v_min = cursor_y as f32 / atlas_h as f32;
            let u_max = (cursor_x + glyph.width) as f32 / atlas_w as f32;
            let v_max = (cursor_y + glyph.height) as f32 / atlas_h as f32;

            glyphs.insert(
                *ch,
                GlyphInfo {
                    uv_rect: UvRect {
                        u_min,
                        v_min,
                        u_max,
                        v_max,
                    },
                    metrics: glyph.metrics,
                },
            );

            cursor_x += glyph.width + GLYPH_PADDING;
            row_height = row_height.max(glyph.height);
        }

        Some((texture_data, glyphs))
    }

    /// Returns glyph info (UV + metrics) for the given character, if present.
    pub fn glyph_info(&self, ch: char) -> Option<&GlyphInfo> {
        self.glyphs.get(&ch)
    }

    /// Returns the raw RGBA8 texture data.
    pub fn texture_data(&self) -> &[u8] {
        &self.texture_data
    }

    /// Returns the atlas texture width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the atlas texture height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_font() -> fontdue::Font {
        let bytes = include_bytes!("../../../test_assets/fonts/test_font.ttf");
        fontdue::Font::from_bytes(bytes as &[u8], fontdue::FontSettings::default())
            .expect("test_font.ttf should parse")
    }

    #[test]
    fn test_atlas_contains_all_printable_ascii() {
        let font = test_font();
        let atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");

        for code in PRINTABLE_ASCII_START..=PRINTABLE_ASCII_END {
            let ch = code as char;
            assert!(
                atlas.glyph_info(ch).is_some(),
                "atlas missing char '{}' ({})",
                ch,
                code
            );
        }
    }

    #[test]
    fn test_atlas_dimensions_are_power_of_two() {
        let font = test_font();
        let atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");

        assert!(atlas.width().is_power_of_two(), "width not pow2");
        assert!(atlas.height().is_power_of_two(), "height not pow2");
    }

    #[test]
    fn test_atlas_texture_data_length_matches_dimensions() {
        let font = test_font();
        let atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");

        let expected = (atlas.width() * atlas.height() * 4) as usize;
        assert_eq!(atlas.texture_data().len(), expected);
    }

    #[test]
    fn test_atlas_no_uv_overlap_for_visible_glyphs() {
        let font = test_font();
        let atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");

        // Collect visible glyph UV rects (skip zero-area ones like space).
        let rects: Vec<(char, UvRect)> = (PRINTABLE_ASCII_START..=PRINTABLE_ASCII_END)
            .filter_map(|code| {
                let ch = code as char;
                let info = atlas.glyph_info(ch)?;
                let uv = &info.uv_rect;
                if (uv.u_max - uv.u_min).abs() < f32::EPSILON {
                    return None;
                }
                Some((ch, *uv))
            })
            .collect();

        // Check all pairs for overlap.
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                let (ch_a, a) = &rects[i];
                let (ch_b, b) = &rects[j];
                let overlaps = a.u_min < b.u_max
                    && a.u_max > b.u_min
                    && a.v_min < b.v_max
                    && a.v_max > b.v_min;
                assert!(!overlaps, "UV overlap between '{}' and '{}'", ch_a, ch_b);
            }
        }
    }

    #[test]
    fn test_atlas_glyph_info_returns_none_for_missing_char() {
        let font = test_font();
        let atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");

        // A non-ASCII char should not be in the atlas.
        assert!(atlas.glyph_info('\u{1F600}').is_none());
    }

    #[test]
    fn test_atlas_visible_glyph_has_nonzero_uv_area() {
        let font = test_font();
        let atlas = GlyphAtlas::generate(&font, 24.0).expect("atlas generation");

        let info = atlas.glyph_info('A').expect("'A' should be in atlas");
        let area =
            (info.uv_rect.u_max - info.uv_rect.u_min) * (info.uv_rect.v_max - info.uv_rect.v_min);
        assert!(area > 0.0, "visible glyph should have nonzero UV area");
    }

    #[test]
    fn test_atlas_space_glyph_has_zero_uv_area() {
        let font = test_font();
        let atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");

        let info = atlas.glyph_info(' ').expect("space should be in atlas");
        let area =
            (info.uv_rect.u_max - info.uv_rect.u_min) * (info.uv_rect.v_max - info.uv_rect.v_min);
        assert!(
            area.abs() < f32::EPSILON,
            "space glyph should have zero UV area"
        );
    }

}
