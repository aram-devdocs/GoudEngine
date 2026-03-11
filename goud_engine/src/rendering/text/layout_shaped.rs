//! Unicode shaping + bidi-aware text layout.
//!
//! Uses `unicode-bidi` to compute visual runs and `rustybuzz` to shape each
//! run into positioned glyph indices.

use rustybuzz::{Direction as BidiDirection, Face, UnicodeBuffer};
use unicode_bidi::BidiInfo;

use super::direction::TextDirection;
use super::glyph_provider::GlyphInfoProvider;
use super::layout::{LayoutGlyph, TextBoundingBox, TextLayoutConfig, TextLayoutResult};

/// A shaped glyph output from RustyBuzz.
#[derive(Debug, Clone, Copy)]
pub struct ShapedGlyph {
    /// Font glyph index.
    pub glyph_id: u16,
    /// Byte cluster offset into the source line.
    pub cluster: usize,
    /// Horizontal advance in pixels.
    pub x_advance: f32,
    /// Horizontal offset in pixels.
    pub x_offset: f32,
}

/// A shaped visual line.
#[derive(Debug, Clone)]
pub struct ShapedLine {
    /// Source line text (logical order, no trailing newline).
    pub text: String,
    /// Shaped glyphs in visual draw order.
    pub glyphs: Vec<ShapedGlyph>,
}

/// Fully shaped text, split by explicit newlines.
#[derive(Debug, Clone, Default)]
pub struct ShapedText {
    /// Lines in logical order.
    pub lines: Vec<ShapedLine>,
}

impl ShapedText {
    /// Returns all unique glyph indices used by this shaped text.
    pub fn glyph_indices(&self) -> Vec<u16> {
        let mut indices = std::collections::BTreeSet::new();
        for line in &self.lines {
            for glyph in &line.glyphs {
                indices.insert(glyph.glyph_id);
            }
        }
        indices.into_iter().collect()
    }
}

#[derive(Debug, Clone)]
struct PlacedGlyph {
    glyph: LayoutGlyph,
    advance: f32,
    is_break: bool,
}

/// Shapes text (including bidi reordering for `Auto`) into visual runs.
pub fn shape_text(
    content: &str,
    font_bytes: &[u8],
    font_size: f32,
    direction: TextDirection,
) -> Result<ShapedText, String> {
    if content.is_empty() {
        return Ok(ShapedText::default());
    }

    let face = Face::from_slice(font_bytes, 0)
        .ok_or_else(|| "failed to parse font bytes for shaping".to_string())?;
    let units_per_em = face.units_per_em() as f32;
    let scale = if units_per_em > 0.0 {
        font_size / units_per_em
    } else {
        1.0
    };

    let mut lines = Vec::new();
    for raw_line in content.split('\n') {
        lines.push(shape_line(raw_line, &face, scale, direction));
    }

    Ok(ShapedText { lines })
}

/// Lays out already-shaped text using glyph atlas metrics.
pub fn layout_shaped_text(
    shaped: &ShapedText,
    atlas: &impl GlyphInfoProvider,
    font_size: f32,
    config: &TextLayoutConfig,
) -> TextLayoutResult {
    if shaped.lines.is_empty() {
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
    let mut wrapped_lines: Vec<Vec<PlacedGlyph>> = Vec::new();

    for line in &shaped.lines {
        let candidates = build_line_candidates(line, atlas);
        if let Some(max_width) = config.max_width {
            wrapped_lines.extend(wrap_line(candidates, max_width));
        } else {
            wrapped_lines.push(candidates);
        }
    }

    let line_widths: Vec<f32> = wrapped_lines.iter().map(|line| line_width(line)).collect();
    let max_line_width = line_widths.iter().copied().fold(0.0f32, f32::max);

    let mut final_glyphs = Vec::new();
    for (line_idx, line) in wrapped_lines.iter().enumerate() {
        let align_offset = match config.alignment {
            crate::core::types::TextAlignment::Left => 0.0,
            crate::core::types::TextAlignment::Center => {
                (max_line_width - line_widths[line_idx]) / 2.0
            }
            crate::core::types::TextAlignment::Right => max_line_width - line_widths[line_idx],
        };
        let y_offset = line_idx as f32 * line_height;

        for placed in line {
            final_glyphs.push(LayoutGlyph {
                x: placed.glyph.x + align_offset,
                y: placed.glyph.y + y_offset,
                character: placed.glyph.character,
                uv_rect: placed.glyph.uv_rect,
                size_x: placed.glyph.size_x,
                size_y: placed.glyph.size_y,
            });
        }
    }

    TextLayoutResult {
        glyphs: final_glyphs,
        bounding_box: TextBoundingBox {
            width: max_line_width,
            height: wrapped_lines.len() as f32 * line_height,
        },
        line_count: wrapped_lines.len(),
    }
}

/// Convenience helper: shape + layout in one call.
pub fn layout_text_shaped(
    content: &str,
    atlas: &impl GlyphInfoProvider,
    font_bytes: &[u8],
    font_size: f32,
    config: &TextLayoutConfig,
    direction: TextDirection,
) -> Result<TextLayoutResult, String> {
    let shaped = shape_text(content, font_bytes, font_size, direction)?;
    Ok(layout_shaped_text(&shaped, atlas, font_size, config))
}

fn shape_line(line: &str, face: &Face<'_>, scale: f32, direction: TextDirection) -> ShapedLine {
    if line.is_empty() {
        return ShapedLine {
            text: String::new(),
            glyphs: Vec::new(),
        };
    }

    let glyphs = match direction {
        TextDirection::LeftToRight => shape_visual_run(face, line, 0, scale, false),
        TextDirection::RightToLeft => shape_visual_run(face, line, 0, scale, true),
        TextDirection::Auto => shape_line_auto(face, line, scale),
    };

    ShapedLine {
        text: line.to_string(),
        glyphs,
    }
}

fn shape_line_auto(face: &Face<'_>, line: &str, scale: f32) -> Vec<ShapedGlyph> {
    let bidi_info = BidiInfo::new(line, None);
    let mut out = Vec::new();

    for paragraph in &bidi_info.paragraphs {
        let (levels, runs) = bidi_info.visual_runs(paragraph, paragraph.range.clone());
        for run in runs {
            let run_start = run.start;
            let Some(run_text) = line.get(run.clone()) else {
                continue;
            };
            let is_rtl = levels
                .get(run_start)
                .map(|level| level.is_rtl())
                .unwrap_or(false);
            out.extend(shape_visual_run(face, run_text, run_start, scale, is_rtl));
        }
    }

    out
}

fn shape_visual_run(
    face: &Face<'_>,
    run_text: &str,
    cluster_offset: usize,
    scale: f32,
    is_rtl: bool,
) -> Vec<ShapedGlyph> {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(run_text);
    buffer.set_direction(if is_rtl {
        BidiDirection::RightToLeft
    } else {
        BidiDirection::LeftToRight
    });

    let shaped = rustybuzz::shape(face, &[], buffer);
    shaped
        .glyph_infos()
        .iter()
        .zip(shaped.glyph_positions().iter())
        .map(|(info, pos)| ShapedGlyph {
            glyph_id: info.glyph_id as u16,
            cluster: cluster_offset + info.cluster as usize,
            x_advance: pos.x_advance as f32 * scale,
            x_offset: pos.x_offset as f32 * scale,
        })
        .collect()
}

fn build_line_candidates(line: &ShapedLine, atlas: &impl GlyphInfoProvider) -> Vec<PlacedGlyph> {
    let mut cursor_x = 0.0f32;
    let mut out = Vec::new();

    for shaped in &line.glyphs {
        let info = match atlas.glyph_info_indexed(shaped.glyph_id) {
            Some(info) => info,
            None => continue,
        };

        let ch = char_at_cluster(&line.text, shaped.cluster).unwrap_or('\0');
        let advance = if shaped.x_advance.abs() > f32::EPSILON {
            shaped.x_advance
        } else {
            info.metrics.advance_width
        };

        out.push(PlacedGlyph {
            glyph: LayoutGlyph {
                x: cursor_x + shaped.x_offset + info.metrics.bearing_x,
                y: -info.metrics.bearing_y,
                character: ch,
                uv_rect: info.uv_rect,
                size_x: info.metrics.width,
                size_y: info.metrics.height,
            },
            advance,
            is_break: ch.is_whitespace(),
        });

        cursor_x += advance;
    }

    out
}

fn wrap_line(glyphs: Vec<PlacedGlyph>, max_width: f32) -> Vec<Vec<PlacedGlyph>> {
    if glyphs.is_empty() {
        return vec![Vec::new()];
    }

    let mut wrapped = Vec::new();
    let mut current = Vec::new();
    let mut cursor_x = 0.0f32;
    let mut last_break: Option<(usize, f32)> = None;

    for mut glyph in glyphs {
        let glyph_start = cursor_x;
        if glyph.is_break {
            last_break = Some((current.len(), glyph_start));
        }

        cursor_x += glyph.advance;

        if cursor_x > max_width && !current.is_empty() {
            if let Some((break_idx, break_x)) = last_break {
                if break_idx >= current.len() {
                    // The wrap-triggering glyph is itself a whitespace break that has not
                    // been pushed into `current` yet. Drop that trailing break and wrap the
                    // existing line instead of splitting past the end.
                    wrapped.push(trim_trailing_whitespace(std::mem::take(&mut current)));
                    cursor_x = 0.0;
                    last_break = None;
                    continue;
                }
                let break_advance = current
                    .get(break_idx)
                    .map(|g: &PlacedGlyph| g.advance)
                    .unwrap_or(0.0);
                let shift = break_x + break_advance;
                let remainder = current.split_off(break_idx + 1);
                wrapped.push(trim_trailing_whitespace(std::mem::take(&mut current)));
                current = remainder
                    .into_iter()
                    .map(|mut g| {
                        g.glyph.x -= shift;
                        g
                    })
                    .collect();
                cursor_x -= shift;
                last_break = recompute_break(&current);
            } else {
                glyph.glyph.x -= glyph_start;
                wrapped.push(std::mem::take(&mut current));
                cursor_x = glyph.advance;
                last_break = None;
            }
        }

        current.push(glyph);
    }

    wrapped.push(trim_trailing_whitespace(current));
    wrapped
}

fn recompute_break(line: &[PlacedGlyph]) -> Option<(usize, f32)> {
    let mut cursor = 0.0f32;
    let mut last = None;
    for (idx, glyph) in line.iter().enumerate() {
        if glyph.is_break {
            last = Some((idx, cursor));
        }
        cursor += glyph.advance;
    }
    last
}

fn trim_trailing_whitespace(mut line: Vec<PlacedGlyph>) -> Vec<PlacedGlyph> {
    while line.last().map(|g| g.is_break).unwrap_or(false) {
        line.pop();
    }
    line
}

fn line_width(line: &[PlacedGlyph]) -> f32 {
    line.last()
        .map(|g| g.glyph.x + g.glyph.size_x)
        .unwrap_or(0.0)
}

fn char_at_cluster(line: &str, cluster: usize) -> Option<char> {
    if cluster >= line.len() {
        return None;
    }
    line.get(cluster..)?.chars().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::text::GlyphAtlas;

    fn parse_font(bytes: &[u8]) -> fontdue::Font {
        fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).expect("parse font")
    }

    fn layout_for_text(
        font: &fontdue::Font,
        font_bytes: &[u8],
        text: &str,
        direction: TextDirection,
    ) -> TextLayoutResult {
        let mut atlas = GlyphAtlas::generate(font, 28.0).expect("atlas generation");
        let shaped = shape_text(text, font_bytes, 28.0, direction).expect("shape");
        atlas
            .ensure_glyph_indices(font, shaped.glyph_indices())
            .expect("ensure glyphs");
        let config = TextLayoutConfig::default();
        layout_shaped_text(&shaped, &atlas, 28.0, &config)
    }

    #[test]
    fn ascii_shaped_layout_produces_glyphs() {
        let bytes = include_bytes!("../../../test_assets/fonts/test_font.ttf");
        let font = parse_font(bytes);
        let layout = layout_for_text(&font, bytes, "Hello", TextDirection::Auto);
        assert!(!layout.glyphs.is_empty());
        assert!(layout.bounding_box.width > 0.0);
    }

    #[test]
    fn cjk_shaped_layout_produces_glyphs() {
        let bytes = include_bytes!("../../../test_assets/fonts/test_unicode_cjk.ttf");
        let font = parse_font(bytes);
        let layout = layout_for_text(&font, bytes, "你好世界", TextDirection::Auto);
        assert!(!layout.glyphs.is_empty());
        assert!(layout.bounding_box.width > 0.0);
    }

    #[test]
    fn rtl_shaped_layout_produces_glyphs() {
        let bytes = include_bytes!("../../../test_assets/fonts/test_unicode_rtl.ttf");
        let font = parse_font(bytes);
        let layout = layout_for_text(&font, bytes, "مرحبا", TextDirection::Auto);
        assert!(!layout.glyphs.is_empty());
        assert!(layout.bounding_box.width > 0.0);
    }

    #[test]
    fn rtl_and_ltr_shaping_order_differs() {
        let bytes = include_bytes!("../../../test_assets/fonts/test_unicode_rtl.ttf");
        let auto = shape_text("مرحبا", bytes, 24.0, TextDirection::RightToLeft).expect("shape");
        let ltr = shape_text("مرحبا", bytes, 24.0, TextDirection::LeftToRight).expect("shape");
        let rtl_ids: Vec<u16> = auto.lines[0].glyphs.iter().map(|g| g.glyph_id).collect();
        let ltr_ids: Vec<u16> = ltr.lines[0].glyphs.iter().map(|g| g.glyph_id).collect();
        assert_ne!(rtl_ids, ltr_ids);
    }

    #[test]
    fn wrapping_whitespace_break_at_overflow_does_not_panic() {
        let bytes = include_bytes!("../../../test_assets/fonts/test_font.ttf");
        let font = parse_font(bytes);
        let mut atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");
        let text =
            "One shared sandbox: same assets, same controls, same feature probes in every SDK.";
        let shaped = shape_text(text, bytes, 16.0, TextDirection::Auto).expect("shape");
        atlas
            .ensure_glyph_indices(&font, shaped.glyph_indices())
            .expect("ensure glyphs");

        let config = TextLayoutConfig {
            max_width: Some(430.0),
            ..TextLayoutConfig::default()
        };

        let layout = layout_shaped_text(&shaped, &atlas, 16.0, &config);
        assert!(layout.line_count > 1);
        assert!(!layout.glyphs.is_empty());
    }
}
