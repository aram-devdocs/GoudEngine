//! Text layout engine for positioning glyphs.
//!
//! Computes glyph positions, word-wrapping, alignment, and bounding boxes
//! from a string of text and a [`GlyphAtlas`](super::glyph_atlas::GlyphAtlas).

use std::collections::HashMap;

use super::glyph_atlas::UvRect;
use super::glyph_provider::GlyphInfoProvider;

// Re-export TextAlignment from its canonical location in core::types.
pub use crate::core::types::TextAlignment;

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

/// Lays out text into positioned glyphs, optionally applying kerning.
///
/// When `kerning` is `Some`, each `(prev, cur)` pair's value is added to
/// `cursor_x` before positioning the glyph.
pub fn layout_text(
    content: &str,
    atlas: &impl GlyphInfoProvider,
    font_size: f32,
    config: &TextLayoutConfig,
    kerning: Option<&HashMap<(char, char), f32>>,
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
    let mut prev_char: Option<char> = None;

    for ch in content.chars() {
        if ch == '\n' {
            lines.push(std::mem::take(&mut current_line));
            cursor_x = 0.0;
            last_space_idx = None;
            prev_char = None;
            continue;
        }

        let info = match atlas.glyph_info(ch) {
            Some(info) => info,
            None => continue,
        };

        // Apply kerning adjustment from the previous character, if available.
        if let (Some(kern_map), Some(prev)) = (kerning, prev_char) {
            if let Some(&kern_value) = kern_map.get(&(prev, ch)) {
                cursor_x += kern_value;
            }
        }

        let glyph_x = cursor_x + info.metrics.bearing_x;
        let glyph_y = info.metrics.bearing_y;

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
        prev_char = Some(ch);

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
#[path = "layout_tests.rs"]
mod tests;
