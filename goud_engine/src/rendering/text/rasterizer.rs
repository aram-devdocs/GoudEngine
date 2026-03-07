//! Glyph rasterization wrapper around `fontdue`.
//!
//! Converts font glyphs into CPU-side bitmaps with associated metrics,
//! ready for atlas packing.

/// Per-glyph metric data extracted during rasterization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphMetrics {
    /// Horizontal advance after rendering this glyph (in pixels).
    pub advance_width: f32,
    /// Horizontal bearing from the origin to the left edge of the glyph.
    pub bearing_x: f32,
    /// Y-offset of the glyph bounding box minimum (fontdue ymin).
    pub bearing_y: f32,
    /// Rasterized glyph width in pixels.
    pub width: f32,
    /// Rasterized glyph height in pixels.
    pub height: f32,
}

/// A single rasterized glyph with its grayscale bitmap and metrics.
///
/// Note: The `width` and `height` fields store the bitmap dimensions as `u32` for
/// array indexing and bitmap operations. The same values are duplicated in
/// `metrics.width` and `metrics.height` as `f32` for layout calculations.
#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    /// Grayscale coverage bitmap (one byte per pixel, 0..=255).
    pub bitmap: Vec<u8>,
    /// Bitmap width in pixels.
    pub width: u32,
    /// Bitmap height in pixels.
    pub height: u32,
    /// Metrics for this glyph.
    pub metrics: GlyphMetrics,
}

/// Rasterizes a set of characters at the given pixel size.
///
/// Returns a `Vec` of `(char, RasterizedGlyph)` pairs in the same
/// order as the input `chars` slice.
///
/// # Arguments
///
/// * `font`    - A parsed `fontdue::Font` reference.
/// * `size_px` - The pixel height to rasterize at.
/// * `chars`   - The characters to rasterize.
pub fn rasterize_glyphs(
    font: &fontdue::Font,
    size_px: f32,
    chars: &[char],
) -> Vec<(char, RasterizedGlyph)> {
    chars
        .iter()
        .map(|&ch| {
            let (metrics, bitmap) = font.rasterize(ch, size_px);
            let glyph = RasterizedGlyph {
                bitmap,
                width: metrics.width as u32,
                height: metrics.height as u32,
                metrics: GlyphMetrics {
                    advance_width: metrics.advance_width,
                    bearing_x: metrics.xmin as f32,
                    bearing_y: metrics.ymin as f32,
                    width: metrics.width as f32,
                    height: metrics.height as f32,
                },
            };
            (ch, glyph)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: parse the test TTF fixture into a `fontdue::Font`.
    fn test_font() -> fontdue::Font {
        let bytes = include_bytes!("../../../test_assets/fonts/test_font.ttf");
        fontdue::Font::from_bytes(bytes as &[u8], fontdue::FontSettings::default())
            .expect("test_font.ttf should parse")
    }

    #[test]
    fn test_rasterize_glyph_a_has_nonzero_dimensions() {
        let font = test_font();
        let glyphs = rasterize_glyphs(&font, 32.0, &['A']);

        assert_eq!(glyphs.len(), 1);
        let (ch, glyph) = &glyphs[0];
        assert_eq!(*ch, 'A');
        assert!(glyph.width > 0, "glyph width should be > 0");
        assert!(glyph.height > 0, "glyph height should be > 0");
    }

    #[test]
    fn test_rasterize_glyph_a_bitmap_has_nonzero_pixels() {
        let font = test_font();
        let glyphs = rasterize_glyphs(&font, 32.0, &['A']);
        let (_, glyph) = &glyphs[0];

        let nonzero_count = glyph.bitmap.iter().filter(|&&b| b > 0).count();
        assert!(nonzero_count > 0, "bitmap should have non-zero pixels");
    }

    #[test]
    fn test_rasterize_glyph_bitmap_length_matches_dimensions() {
        let font = test_font();
        let glyphs = rasterize_glyphs(&font, 24.0, &['B']);
        let (_, glyph) = &glyphs[0];

        let expected_len = (glyph.width * glyph.height) as usize;
        assert_eq!(glyph.bitmap.len(), expected_len);
    }

    #[test]
    fn test_rasterize_multiple_chars_preserves_order() {
        let font = test_font();
        let chars = vec!['X', 'Y', 'Z'];
        let glyphs = rasterize_glyphs(&font, 16.0, &chars);

        assert_eq!(glyphs.len(), 3);
        assert_eq!(glyphs[0].0, 'X');
        assert_eq!(glyphs[1].0, 'Y');
        assert_eq!(glyphs[2].0, 'Z');
    }

    #[test]
    fn test_rasterize_space_glyph_has_zero_dimensions() {
        let font = test_font();
        let glyphs = rasterize_glyphs(&font, 32.0, &[' ']);
        let (_, glyph) = &glyphs[0];

        // Space typically has zero width/height bitmap but nonzero advance.
        assert_eq!(glyph.width, 0);
        assert_eq!(glyph.height, 0);
        assert!(
            glyph.metrics.advance_width > 0.0,
            "space should have positive advance"
        );
    }

    #[test]
    fn test_rasterize_metrics_advance_width_positive_for_visible_glyph() {
        let font = test_font();
        let glyphs = rasterize_glyphs(&font, 20.0, &['M']);
        let (_, glyph) = &glyphs[0];

        assert!(
            glyph.metrics.advance_width > 0.0,
            "advance_width should be positive for 'M'"
        );
    }
}
