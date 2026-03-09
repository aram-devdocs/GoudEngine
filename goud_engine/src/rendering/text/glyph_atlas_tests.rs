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
            let overlaps =
                a.u_min < b.u_max && a.u_max > b.u_min && a.v_min < b.v_max && a.v_max > b.v_min;
            assert!(!overlaps, "UV overlap between '{}' and '{}'", ch_a, ch_b);
        }
    }
}

#[test]
fn test_atlas_glyph_info_returns_none_for_missing_char() {
    let font = test_font();
    let atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");
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
fn test_gpu_texture_returns_none_before_upload() {
    let font = test_font();
    let atlas = GlyphAtlas::generate(&font, 16.0).expect("atlas generation");
    assert!(atlas.gpu_texture().is_none());
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

#[test]
fn test_dynamic_char_addition_updates_char_lookup() {
    let font = test_font();
    let mut atlas = GlyphAtlas::generate(&font, 20.0).expect("atlas generation");

    assert!(atlas.glyph_info('é').is_none());
    let changed = atlas.ensure_chars(&font, ['é']).expect("ensure chars");
    assert!(changed);
    assert!(atlas.glyph_info('é').is_some());
}

#[test]
fn test_indexed_lookup_available_for_shaped_pipeline() {
    let font = test_font();
    let atlas = GlyphAtlas::generate(&font, 20.0).expect("atlas generation");
    let idx = font.lookup_glyph_index('A');
    assert!(atlas.glyph_info_indexed(idx).is_some());
}

#[test]
fn test_dynamic_growth_rebuilds_atlas_and_bumps_version() {
    let font = test_font();
    let mut atlas = GlyphAtlas::generate_for_chars(&font, 64.0, &['A']).expect("atlas generation");
    let initial_version = atlas.version();
    let initial_width = atlas.width();

    let all_indices: Vec<u16> = (0..font.glyph_count()).collect();
    let changed = atlas
        .ensure_glyph_indices(&font, all_indices)
        .expect("ensure glyph indices");

    assert!(changed);
    assert!(atlas.version() > initial_version);
    assert!(atlas.width() >= initial_width);
    assert!(atlas.is_dirty());
}

#[test]
fn test_rasterize_glyphs_char_path_retained() {
    let font = test_font();
    let rasterized = super::super::rasterizer::rasterize_glyphs(&font, 16.0, &['A']);
    assert_eq!(rasterized.len(), 1);
    assert_eq!(rasterized[0].0, 'A');
}
