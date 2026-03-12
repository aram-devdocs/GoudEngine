use super::*;

#[test]
fn test_text_batch_new_is_empty() {
    let batch = TextBatch::new();
    let stats = batch.stats();
    assert_eq!(stats.glyph_count, 0);
    assert_eq!(stats.draw_calls, 0);
}

#[test]
fn test_text_batch_begin_clears_state() {
    let mut batch = TextBatch::new();
    // Manually push some data.
    batch.vertices.push(SpriteVertex {
        position: Vec2::new(0.0, 0.0),
        tex_coords: Vec2::new(0.0, 0.0),
        color: Color::WHITE,
    });
    batch.indices.push(0);
    batch.stats.glyph_count = 5;

    batch.begin();

    assert!(batch.vertices.is_empty());
    assert!(batch.indices.is_empty());
    assert!(batch.batches.is_empty());
    assert_eq!(batch.stats().glyph_count, 0);
}

#[test]
fn test_text_batch_default_equals_new() {
    let a = TextBatch::new();
    let b = TextBatch::default();
    assert_eq!(a.stats().glyph_count, b.stats().glyph_count);
    assert_eq!(a.stats().draw_calls, b.stats().draw_calls);
}

#[test]
fn test_text_batch_debug_format() {
    let batch = TextBatch::new();
    let debug = format!("{:?}", batch);
    assert!(debug.contains("TextBatch"));
}

#[test]
fn test_emit_glyph_quad_produces_correct_geometry() {
    let mut batch = TextBatch::new();
    let identity = crate::ecs::components::Mat3x3::IDENTITY;
    let uv = UvRect {
        u_min: 0.0,
        v_min: 0.0,
        u_max: 1.0,
        v_max: 1.0,
    };

    batch.emit_glyph_quad(10.0, 20.0, 8.0, 12.0, &uv, Color::WHITE, &identity);

    assert_eq!(batch.vertices.len(), 4);
    assert_eq!(batch.indices.len(), 6);

    // Verify positions.
    assert_eq!(batch.vertices[0].position, Vec2::new(10.0, 20.0));
    assert_eq!(batch.vertices[1].position, Vec2::new(18.0, 20.0));
    assert_eq!(batch.vertices[2].position, Vec2::new(18.0, 32.0));
    assert_eq!(batch.vertices[3].position, Vec2::new(10.0, 32.0));

    // Verify indices form two triangles.
    assert_eq!(&batch.indices[..], &[0, 1, 2, 2, 3, 0]);
}

#[test]
fn test_emit_two_quads_produces_correct_indices() {
    let mut batch = TextBatch::new();
    let identity = crate::ecs::components::Mat3x3::IDENTITY;
    let uv = UvRect {
        u_min: 0.0,
        v_min: 0.0,
        u_max: 1.0,
        v_max: 1.0,
    };

    batch.emit_glyph_quad(0.0, 0.0, 8.0, 8.0, &uv, Color::WHITE, &identity);
    batch.emit_glyph_quad(10.0, 0.0, 8.0, 8.0, &uv, Color::WHITE, &identity);

    assert_eq!(batch.vertices.len(), 8);
    assert_eq!(batch.indices.len(), 12);
    // Second quad indices should be offset by 4.
    assert_eq!(&batch.indices[6..], &[4, 5, 6, 6, 7, 4]);
}

#[test]
fn test_draw_text_with_world_and_null_backend_counts_glyphs() {
    use crate::assets::loaders::FontLoader;
    use crate::assets::AssetServer;
    use crate::ecs::components::Transform2D;
    use crate::ecs::World;
    use crate::libs::graphics::backend::null::NullBackend;

    // Set up a null render backend for headless testing.
    let mut backend = NullBackend::new();

    // Create an AssetServer with a FontLoader registered.
    let mut asset_server = AssetServer::new();
    asset_server.register_loader(FontLoader::default());

    // Load the test font from embedded bytes.
    let ttf_bytes = include_bytes!("../../../test_assets/fonts/test_font.ttf");
    let font_handle = asset_server
        .load_from_bytes::<crate::assets::loaders::FontAsset>("test_font.ttf", ttf_bytes);
    assert!(
        asset_server.is_loaded(&font_handle),
        "font asset should be loaded"
    );

    // Create a World and spawn an entity with Text + Transform2D.
    let mut world = World::new();
    let text = crate::ecs::components::Text::new(font_handle, "Hello").with_font_size(16.0);
    let transform = Transform2D::default();
    let _entity = world.spawn().insert(text).insert(transform).id();

    // Run the text batch pipeline.
    let mut batch = TextBatch::new();
    batch.begin();
    batch
        .draw_text(&world, &asset_server, &mut backend)
        .expect("draw_text should succeed with null backend");

    // "Hello" has 5 characters, so we expect 5 glyphs.
    assert_eq!(
        batch.stats().glyph_count,
        5,
        "expected 5 glyphs for 'Hello'"
    );
}

#[test]
fn test_text_batch_reuses_shader_across_frames() {
    use crate::core::math::Color;
    use crate::core::math::Vec2;
    use crate::ecs::components::Transform2D;
    use crate::libs::graphics::backend::null::NullBackend;
    use crate::libs::graphics::backend::types::{TextureFilter, TextureFormat, TextureWrap};
    use crate::libs::graphics::backend::TextureOps;
    use crate::rendering::text::{LayoutGlyph, TextBoundingBox, TextLayoutResult, UvRect};

    let mut backend = NullBackend::new();
    let mut batch = TextBatch::new();
    let texture = backend
        .create_texture(
            2,
            2,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
            &[255u8; 16],
        )
        .expect("null texture creation");

    let layout = TextLayoutResult {
        glyphs: vec![LayoutGlyph {
            x: 10.0,
            y: 20.0,
            character: 'A',
            uv_rect: UvRect {
                u_min: 0.0,
                v_min: 0.0,
                u_max: 1.0,
                v_max: 1.0,
            },
            size_x: 8.0,
            size_y: 12.0,
        }],
        bounding_box: TextBoundingBox {
            width: 8.0,
            height: 12.0,
        },
        line_count: 1,
    };
    let transform = Transform2D::from_position(Vec2::new(0.0, 0.0));

    batch.begin();
    batch.append_glyph_batch(&layout, Color::WHITE, &transform, texture);
    batch
        .end(&mut backend, (1280, 720))
        .expect("first text frame");

    batch.begin();
    batch.append_glyph_batch(&layout, Color::WHITE, &transform, texture);
    batch
        .end(&mut backend, (1280, 720))
        .expect("second text frame");

    assert_eq!(
        backend.shader_create_calls(),
        1,
        "text shader should be created once and reused across frames"
    );
}
