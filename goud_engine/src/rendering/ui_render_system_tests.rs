use super::*;
use crate::assets::loaders::TextureAsset;
use crate::core::math::{Color, Rect};
use crate::libs::graphics::backend::null::NullBackend;
use crate::rendering::ensure_ui_asset_loaders;
use crate::ui::{
    UiNodeId, UiQuadCommand, UiRenderCommand, UiTextCommand, UiTexturedQuadCommand,
    UI_DEFAULT_FONT_ASSET_PATH, UI_DEFAULT_FONT_FAMILY,
};

fn preload_test_fixtures(asset_server: &mut AssetServer) {
    ensure_ui_asset_loaders(asset_server);

    asset_server.load_from_bytes::<FontAsset>(
        UI_DEFAULT_FONT_ASSET_PATH,
        include_bytes!("../../test_assets/fonts/test_font.ttf"),
    );
    asset_server.load_from_bytes::<TextureAsset>(
        "ui/fixture-checker.png",
        include_bytes!("../../test_assets/fonts/test_bitmap.png"),
    );
}

#[test]
fn headless_run_renders_text_and_tracks_stats() {
    let mut system = UiRenderSystem::new();
    let mut backend = NullBackend::new();
    let mut assets = AssetServer::new();
    preload_test_fixtures(&mut assets);

    let node = UiNodeId::new(1, 1);
    let commands = vec![
        UiRenderCommand::Quad(UiQuadCommand {
            node_id: node,
            rect: Rect::new(0.0, 0.0, 32.0, 32.0),
            color: Color::RED,
        }),
        UiRenderCommand::TexturedQuad(UiTexturedQuadCommand {
            node_id: node,
            rect: Rect::new(4.0, 4.0, 16.0, 16.0),
            texture_path: "ui/fixture-checker.png".to_string(),
            tint: Color::WHITE,
        }),
        UiRenderCommand::Text(UiTextCommand {
            node_id: node,
            text: "HUD".to_string(),
            position: [12.0, 8.0],
            font_size: 16.0,
            color: Color::WHITE,
            font_family: UI_DEFAULT_FONT_FAMILY.to_string(),
        }),
    ];

    system
        .run(&commands, &mut assets, &mut backend, (800, 600))
        .expect("UiRenderSystem run should succeed with NullBackend");

    let stats = system.stats();
    assert_eq!(stats.quad_commands, 1);
    assert_eq!(stats.textured_quad_commands, 1);
    assert_eq!(stats.text_commands, 1);
    assert!(stats.quad_draw_calls >= 1);
    assert_eq!(stats.quad_index_count, 12);
    assert!(stats.text_glyph_count >= 3);
    assert!(stats.text_draw_calls >= 1);
}

#[test]
fn unknown_font_family_falls_back_to_f05() {
    let mut system = UiRenderSystem::new();
    let mut backend = NullBackend::new();
    let mut assets = AssetServer::new();
    preload_test_fixtures(&mut assets);

    let commands = vec![UiRenderCommand::Text(UiTextCommand {
        node_id: UiNodeId::new(2, 1),
        text: "Fallback".to_string(),
        position: [0.0, 0.0],
        font_size: 14.0,
        color: Color::WHITE,
        font_family: "NoSuchFontFamily".to_string(),
    })];

    system
        .run(&commands, &mut assets, &mut backend, (640, 360))
        .expect("UiRenderSystem should fallback to F05 when available");

    assert!(system.stats().text_glyph_count > 0);
}

#[test]
fn missing_font_assets_are_skipped_without_error() {
    let mut system = UiRenderSystem::new();
    let mut backend = NullBackend::new();
    let mut assets = AssetServer::new();

    let commands = vec![UiRenderCommand::Text(UiTextCommand {
        node_id: UiNodeId::new(3, 1),
        text: "NoFont".to_string(),
        position: [0.0, 0.0],
        font_size: 14.0,
        color: Color::WHITE,
        font_family: "NoSuchFontFamily".to_string(),
    })];

    system
        .run(&commands, &mut assets, &mut backend, (640, 360))
        .expect("UiRenderSystem should skip text if no font can be resolved");

    let stats = system.stats();
    assert_eq!(stats.text_commands, 1);
    assert_eq!(stats.text_glyph_count, 0);
    assert_eq!(stats.text_draw_calls, 0);
}

#[test]
fn headless_run_batches_ui_quads_without_sprite_batch_state() {
    let mut system = UiRenderSystem::new();
    let mut backend = NullBackend::new();
    let mut assets = AssetServer::new();
    preload_test_fixtures(&mut assets);

    let node = UiNodeId::new(4, 1);
    let commands = vec![
        UiRenderCommand::Quad(UiQuadCommand {
            node_id: node,
            rect: Rect::new(0.0, 0.0, 24.0, 24.0),
            color: Color::RED,
        }),
        UiRenderCommand::Quad(UiQuadCommand {
            node_id: node,
            rect: Rect::new(24.0, 0.0, 24.0, 24.0),
            color: Color::GREEN,
        }),
        UiRenderCommand::TexturedQuad(UiTexturedQuadCommand {
            node_id: node,
            rect: Rect::new(48.0, 0.0, 24.0, 24.0),
            texture_path: "ui/fixture-checker.png".to_string(),
            tint: Color::WHITE,
        }),
    ];

    system
        .run(&commands, &mut assets, &mut backend, (320, 180))
        .expect("UiRenderSystem should batch UI quads without a SpriteBatch");

    let stats = system.stats();
    assert_eq!(stats.quad_commands, 2);
    assert_eq!(stats.textured_quad_commands, 1);
    assert_eq!(stats.text_commands, 0);
    assert_eq!(stats.quad_draw_calls, 2);
    assert_eq!(stats.quad_index_count, 18);
}
