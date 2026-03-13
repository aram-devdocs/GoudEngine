use super::*;
use crate::rendering::text::glyph_atlas::GlyphAtlas;

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
    let result = layout_text("", &atlas, 16.0, &config, None);

    assert_eq!(result.glyphs.len(), 0);
    assert_eq!(result.line_count, 0);
    assert_eq!(result.bounding_box.width, 0.0);
    assert_eq!(result.bounding_box.height, 0.0);
}

/// Also serves as a pipeline integration test: validates glyph count
/// without requiring a GL context (which TextBatch would need).
#[test]
fn test_layout_single_line() {
    let atlas = test_atlas();
    let config = TextLayoutConfig::default();
    let result = layout_text("Hello", &atlas, 16.0, &config, None);
    assert_eq!(result.glyphs.len(), 5);
    assert_eq!(result.line_count, 1);
    assert!(result.bounding_box.width > 0.0);
    assert!(result.bounding_box.height > 0.0);
    for i in 1..result.glyphs.len() {
        assert!(result.glyphs[i].x >= result.glyphs[i - 1].x);
    }
}

#[test]
fn test_layout_explicit_newline() {
    let atlas = test_atlas();
    let config = TextLayoutConfig::default();
    let result = layout_text("AB\nCD", &atlas, 16.0, &config, None);
    assert_eq!(result.line_count, 2);
    assert_eq!(result.glyphs.len(), 4);
    let (first_y, second_y) = (result.glyphs[0].y, result.glyphs[2].y);
    assert!(
        second_y > first_y,
        "line 2 y ({second_y}) <= line 1 y ({first_y})"
    );
}

#[test]
fn test_layout_center_alignment() {
    let atlas = test_atlas();
    let config = TextLayoutConfig {
        alignment: TextAlignment::Center,
        ..Default::default()
    };
    let result = layout_text("ABCDEF\nAB", &atlas, 16.0, &config, None);
    assert_eq!(result.line_count, 2);
    // Shorter line's first glyph should be offset to center.
    assert!(result.glyphs[6].x > 0.0);
}

#[test]
fn test_layout_right_alignment() {
    let atlas = test_atlas();
    let right_cfg = TextLayoutConfig {
        alignment: TextAlignment::Right,
        ..Default::default()
    };
    let result = layout_text("ABCDEF\nAB", &atlas, 16.0, &right_cfg, None);
    assert_eq!(result.line_count, 2);
    let right_x = result.glyphs[6].x;
    assert!(right_x > 0.0);
    // Right offset should exceed center offset.
    let center_cfg = TextLayoutConfig {
        alignment: TextAlignment::Center,
        ..Default::default()
    };
    let center_result = layout_text("ABCDEF\nAB", &atlas, 16.0, &center_cfg, None);
    assert!(right_x > center_result.glyphs[6].x);
}

#[test]
fn test_layout_word_wrap() {
    let atlas = test_atlas();
    let config = TextLayoutConfig {
        max_width: Some(50.0),
        ..Default::default()
    };

    let result = layout_text("Hello World Test", &atlas, 16.0, &config, None);

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
    let result = layout_text("Test", &atlas, 16.0, &config, None);

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
fn test_glyph_positions_advance_left_to_right() {
    let atlas = test_atlas();
    let config = TextLayoutConfig::default();
    let result = layout_text("AB", &atlas, 16.0, &config, None);

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

    let result_a = layout_text("Hi", &atlas, 16.0, &config, None);
    let result_b = layout_text("World", &atlas, 16.0, &config, None);

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
    let result_a_again = layout_text("Hi", &atlas, 16.0, &config, None);
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
    let result_1x = layout_text("A\nB", &atlas, font_size, &config_1x, None);

    let config_2x = TextLayoutConfig {
        line_spacing: 2.0,
        ..Default::default()
    };
    let result_2x = layout_text("A\nB", &atlas, font_size, &config_2x, None);

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

#[test]
fn test_baseline_alignment_across_different_glyphs() {
    let atlas = test_atlas();
    let config = TextLayoutConfig::default();
    let result = layout_text("AaxT", &atlas, 16.0, &config, None);

    assert_eq!(result.glyphs.len(), 4);

    // Non-descender characters should share the same baseline.
    // baseline = glyph_y + glyph_height (bottom of the glyph bitmap).
    let baselines: Vec<f32> = result
        .glyphs
        .iter()
        .filter(|g| g.size_y > 0.0)
        .map(|g| g.y + g.size_y)
        .collect();

    assert!(
        !baselines.is_empty(),
        "should have at least one visible glyph"
    );

    let first = baselines[0];
    for (i, &bl) in baselines.iter().enumerate() {
        assert!(
            (bl - first).abs() <= 1.0,
            "glyph {} baseline ({}) differs from first ({}) by more than 1px",
            i,
            bl,
            first
        );
    }
}

#[test]
fn test_kerning_adjusts_glyph_positions() {
    let atlas = test_atlas();
    let config = TextLayoutConfig::default();

    // Layout without kerning.
    let result_no_kern = layout_text("AV", &atlas, 16.0, &config, None);
    assert_eq!(result_no_kern.glyphs.len(), 2);

    // Layout with negative kerning between A and V (tightens spacing).
    let mut kerning = HashMap::new();
    kerning.insert(('A', 'V'), -3.0);
    let result_with_kern = layout_text("AV", &atlas, 16.0, &config, Some(&kerning));
    assert_eq!(result_with_kern.glyphs.len(), 2);

    // The V glyph should be 3 pixels closer with kerning.
    let v_x_no_kern = result_no_kern.glyphs[1].x;
    let v_x_with_kern = result_with_kern.glyphs[1].x;
    assert!(
        (v_x_no_kern - v_x_with_kern - 3.0).abs() < 0.01,
        "kerning should shift V left by 3px: no_kern={}, with_kern={}",
        v_x_no_kern,
        v_x_with_kern
    );
}
