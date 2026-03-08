//! Text layout engine for positioning glyphs.
//!
//! Computes glyph positions, word-wrapping, alignment, and bounding boxes
//! from a string of text and a [`GlyphAtlas`].

use super::glyph_atlas::{GlyphAtlas, UvRect};

/// Horizontal text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TextAlignment {
    /// Align text to the left edge.
    #[default]
    Left,
    /// Center text horizontally.
    Center,
    /// Align text to the right edge.
    Right,
}

/// Configuration for text layout.
#[derive(Debug, Clone)]
pub struct TextLayoutConfig {
    /// Maximum width before word-wrapping. `None` means no wrapping.
    pub max_width: Option<f32>,
    /// Multiplier for line spacing (1.0 = default spacing).
    pub line_spacing: f32,
    /// Horizontal text alignment.
    pub alignment: TextAlignment,
}

impl Default for TextLayoutConfig {
    fn default() -> Self {
        Self {
            max_width: None,
            line_spacing: 1.0,
            alignment: TextAlignment::Left,
        }
    }
}

/// A single positioned glyph in a layout result.
#[derive(Debug, Clone)]
pub struct LayoutGlyph {
    /// X position of the glyph.
    pub x: f32,
    /// Y position of the glyph.
    pub y: f32,
    /// The character this glyph represents.
    pub character: char,
    /// UV rectangle in the atlas texture.
    pub uv_rect: UvRect,
    /// Width of the glyph in pixels.
    pub size_x: f32,
    /// Height of the glyph in pixels.
    pub size_y: f32,
}

/// Bounding box of a laid-out text block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextBoundingBox {
    /// Width of the bounding box.
    pub width: f32,
    /// Height of the bounding box.
    pub height: f32,
}

/// Result of laying out text.
#[derive(Debug, Clone)]
pub struct TextLayoutResult {
    /// Positioned glyphs ready for rendering.
    pub glyphs: Vec<LayoutGlyph>,
    /// Bounding box enclosing all glyphs.
    pub bounding_box: TextBoundingBox,
    /// Number of lines in the layout.
    pub line_count: usize,
}

/// Lays out text into positioned glyphs using metrics from a [`GlyphAtlas`].
///
/// Handles word-wrapping at `config.max_width`, explicit newlines, and
/// horizontal alignment. Returns positioned glyphs with UV coordinates
/// for rendering.
pub fn layout_text(
    content: &str,
    atlas: &GlyphAtlas,
    font_size: f32,
    config: &TextLayoutConfig,
) -> TextLayoutResult {
    if content.is_empty() {
        return TextLayoutResult {
            glyphs: Vec::new(),
            bounding_box: TextBoundingBox {
                width: 0.0,
                height: 0.0,
            },
            line_count: 0,
        };
    }

    let line_height = font_size * config.line_spacing;

    // Collect glyphs into lines, handling word-wrap and explicit newlines.
    let mut lines: Vec<Vec<LayoutGlyph>> = Vec::new();
    let mut current_line: Vec<LayoutGlyph> = Vec::new();
    let mut cursor_x: f32 = 0.0;
    let mut last_space_idx: Option<usize> = None;
    let mut cursor_x_at_last_space: f32 = 0.0;

    for ch in content.chars() {
        if ch == '\n' {
            lines.push(std::mem::take(&mut current_line));
            cursor_x = 0.0;
            last_space_idx = None;
            continue;
        }

        let info = match atlas.glyph_info(ch) {
            Some(info) => info,
            None => continue,
        };

        let glyph_x = cursor_x + info.metrics.bearing_x;
        let glyph_y = -info.metrics.bearing_y;

        let glyph = LayoutGlyph {
            x: glyph_x,
            y: glyph_y,
            character: ch,
            uv_rect: info.uv_rect,
            size_x: info.metrics.width,
            size_y: info.metrics.height,
        };

        if ch == ' ' {
            last_space_idx = Some(current_line.len());
            cursor_x_at_last_space = cursor_x;
        }

        cursor_x += info.metrics.advance_width;

        // Check word-wrap.
        if let Some(max_w) = config.max_width {
            if cursor_x > max_w && !current_line.is_empty() {
                if let Some(space_idx) = last_space_idx {
                    // Break at the last space: everything up to (not including)
                    // the space goes on the current line.
                    let remainder: Vec<LayoutGlyph> = current_line.drain(space_idx..).collect();
                    lines.push(std::mem::take(&mut current_line));

                    // Re-position the remainder (skip the space itself).
                    let offset = cursor_x_at_last_space
                        + atlas
                            .glyph_info(' ')
                            .map(|i| i.metrics.advance_width)
                            .unwrap_or(0.0);
                    for mut g in remainder.into_iter().skip(1) {
                        g.x -= offset;
                        current_line.push(g);
                    }
                    // Re-add current glyph with adjusted x
                    let mut adjusted = glyph;
                    adjusted.x -= offset;
                    current_line.push(adjusted);
                    cursor_x -= offset;
                    last_space_idx = None;
                } else {
                    // No space to break at; force break before this glyph.
                    lines.push(std::mem::take(&mut current_line));
                    let mut adjusted = glyph;
                    adjusted.x = info.metrics.bearing_x;
                    current_line.push(adjusted);
                    cursor_x = info.metrics.advance_width;
                    last_space_idx = None;
                }
                continue;
            }
        }

        current_line.push(glyph);
    }

    // Push the last line.
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Compute per-line widths for alignment.
    let line_widths: Vec<f32> = lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                0.0
            } else {
                let last = &line[line.len() - 1];
                last.x + last.size_x
            }
        })
        .collect();

    let max_line_width = line_widths.iter().cloned().fold(0.0f32, f32::max);

    // Apply alignment and assign final y positions.
    let mut final_glyphs: Vec<LayoutGlyph> = Vec::new();
    for (line_idx, line) in lines.iter().enumerate() {
        let line_width = line_widths[line_idx];
        let align_offset = match config.alignment {
            TextAlignment::Left => 0.0,
            TextAlignment::Center => (max_line_width - line_width) / 2.0,
            TextAlignment::Right => max_line_width - line_width,
        };

        let y_offset = line_idx as f32 * line_height;

        for g in line {
            final_glyphs.push(LayoutGlyph {
                x: g.x + align_offset,
                y: g.y + y_offset,
                character: g.character,
                uv_rect: g.uv_rect,
                size_x: g.size_x,
                size_y: g.size_y,
            });
        }
    }

    let total_height = if lines.is_empty() {
        0.0
    } else {
        lines.len() as f32 * line_height
    };

    TextLayoutResult {
        glyphs: final_glyphs,
        bounding_box: TextBoundingBox {
            width: max_line_width,
            height: total_height,
        },
        line_count: lines.len(),
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

    fn test_atlas() -> GlyphAtlas {
        let font = test_font();
        GlyphAtlas::generate(&font, 16.0).expect("atlas generation")
    }

    #[test]
    fn test_layout_empty_string() {
        let atlas = test_atlas();
        let config = TextLayoutConfig::default();
        let result = layout_text("", &atlas, 16.0, &config);

        assert_eq!(result.glyphs.len(), 0);
        assert_eq!(result.line_count, 0);
        assert_eq!(result.bounding_box.width, 0.0);
        assert_eq!(result.bounding_box.height, 0.0);
    }

    #[test]
    fn test_layout_single_line() {
        let atlas = test_atlas();
        let config = TextLayoutConfig::default();
        let result = layout_text("Hello", &atlas, 16.0, &config);

        assert_eq!(result.glyphs.len(), 5);
        assert_eq!(result.line_count, 1);
        assert!(result.bounding_box.width > 0.0);
        assert!(result.bounding_box.height > 0.0);

        // Glyphs should be ordered left to right.
        for i in 1..result.glyphs.len() {
            assert!(
                result.glyphs[i].x >= result.glyphs[i - 1].x,
                "glyph {} should be right of glyph {}",
                i,
                i - 1
            );
        }
    }

    #[test]
    fn test_layout_explicit_newline() {
        let atlas = test_atlas();
        let config = TextLayoutConfig::default();
        let result = layout_text("AB\nCD", &atlas, 16.0, &config);

        assert_eq!(result.line_count, 2);
        // Should have 4 visible glyphs (newline is not rendered).
        assert_eq!(result.glyphs.len(), 4);

        // Second line glyphs should have larger y than first line glyphs.
        let first_line_y = result.glyphs[0].y;
        let second_line_y = result.glyphs[2].y;
        assert!(
            second_line_y > first_line_y,
            "second line y ({}) should be > first line y ({})",
            second_line_y,
            first_line_y
        );
    }

    #[test]
    fn test_layout_center_alignment() {
        let atlas = test_atlas();
        let config = TextLayoutConfig {
            alignment: TextAlignment::Center,
            ..Default::default()
        };

        // Two lines of different lengths.
        let result = layout_text("ABCDEF\nAB", &atlas, 16.0, &config);
        assert_eq!(result.line_count, 2);

        // The shorter line should be offset to center.
        // First glyph of the shorter line should have x > 0.
        let short_line_first_x = result.glyphs[6].x; // 'A' of "AB"
        assert!(
            short_line_first_x > 0.0,
            "center-aligned short line should be offset (got {})",
            short_line_first_x
        );
    }

    #[test]
    fn test_layout_right_alignment() {
        let atlas = test_atlas();
        let config = TextLayoutConfig {
            alignment: TextAlignment::Right,
            ..Default::default()
        };

        let result = layout_text("ABCDEF\nAB", &atlas, 16.0, &config);
        assert_eq!(result.line_count, 2);

        // The shorter line should be right-aligned.
        let short_line_first_x = result.glyphs[6].x;
        assert!(
            short_line_first_x > 0.0,
            "right-aligned short line should be offset (got {})",
            short_line_first_x
        );

        // Right-aligned short line's offset should be greater than center offset.
        let center_config = TextLayoutConfig {
            alignment: TextAlignment::Center,
            ..Default::default()
        };
        let center_result = layout_text("ABCDEF\nAB", &atlas, 16.0, &center_config);
        let center_x = center_result.glyphs[6].x;
        assert!(
            short_line_first_x > center_x,
            "right offset ({}) should be > center offset ({})",
            short_line_first_x,
            center_x
        );
    }

    #[test]
    fn test_layout_word_wrap() {
        let atlas = test_atlas();
        let config = TextLayoutConfig {
            max_width: Some(50.0),
            ..Default::default()
        };

        let result = layout_text("Hello World Test", &atlas, 16.0, &config);

        // With a narrow max_width, text should wrap to multiple lines.
        assert!(
            result.line_count > 1,
            "expected word-wrap to produce >1 line, got {}",
            result.line_count
        );
    }

    #[test]
    fn test_layout_bounding_box() {
        let atlas = test_atlas();
        let config = TextLayoutConfig::default();
        let result = layout_text("Test", &atlas, 16.0, &config);

        assert!(result.bounding_box.width > 0.0);
        assert!(result.bounding_box.height > 0.0);

        // All glyphs should be within the bounding box width.
        for g in &result.glyphs {
            assert!(
                g.x + g.size_x <= result.bounding_box.width + 0.01,
                "glyph '{}' at x={} with size_x={} exceeds bbox width {}",
                g.character,
                g.x,
                g.size_x,
                result.bounding_box.width
            );
        }
    }

    #[test]
    fn test_layout_produces_correct_glyph_count() {
        let atlas = test_atlas();
        let config = TextLayoutConfig::default();
        let result = layout_text("Hello", &atlas, 16.0, &config);

        assert_eq!(
            result.glyphs.len(),
            5,
            "layout of 'Hello' should produce exactly 5 LayoutGlyphs"
        );
    }

    #[test]
    fn test_glyph_positions_advance_left_to_right() {
        let atlas = test_atlas();
        let config = TextLayoutConfig::default();
        let result = layout_text("AB", &atlas, 16.0, &config);

        assert_eq!(result.glyphs.len(), 2);
        assert!(
            result.glyphs[1].x > result.glyphs[0].x,
            "glyph[1].x ({}) should be > glyph[0].x ({})",
            result.glyphs[1].x,
            result.glyphs[0].x
        );
    }

    #[test]
    fn test_layout_results_are_independent_per_call() {
        let atlas = test_atlas();
        let config = TextLayoutConfig::default();

        let result_a = layout_text("Hi", &atlas, 16.0, &config);
        let result_b = layout_text("World", &atlas, 16.0, &config);

        assert_eq!(
            result_a.glyphs.len(),
            2,
            "first layout_text('Hi') should produce 2 glyphs"
        );
        assert_eq!(
            result_b.glyphs.len(),
            5,
            "second layout_text('World') should produce 5 glyphs"
        );

        // Verify results are truly independent: re-layout the first string
        // and confirm the result is unchanged.
        let result_a_again = layout_text("Hi", &atlas, 16.0, &config);
        assert_eq!(
            result_a.glyphs.len(),
            result_a_again.glyphs.len(),
            "repeated layout should produce identical glyph counts"
        );
    }

    #[test]
    fn test_layout_line_spacing() {
        let atlas = test_atlas();
        let font_size = 16.0;

        let config_1x = TextLayoutConfig {
            line_spacing: 1.0,
            ..Default::default()
        };
        let result_1x = layout_text("A\nB", &atlas, font_size, &config_1x);

        let config_2x = TextLayoutConfig {
            line_spacing: 2.0,
            ..Default::default()
        };
        let result_2x = layout_text("A\nB", &atlas, font_size, &config_2x);

        // With 2x line spacing, the second line should be further down.
        let y_diff_1x = result_1x.glyphs[1].y - result_1x.glyphs[0].y;
        let y_diff_2x = result_2x.glyphs[1].y - result_2x.glyphs[0].y;

        assert!(
            (y_diff_2x - 2.0 * y_diff_1x).abs() < 0.01,
            "2x spacing y_diff ({}) should be ~2x of 1x spacing y_diff ({})",
            y_diff_2x,
            y_diff_1x
        );
    }
}
