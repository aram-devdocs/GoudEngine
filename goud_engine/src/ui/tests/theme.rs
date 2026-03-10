use crate::core::math::Color;
use crate::core::math::Vec2;
use crate::ui::{
    resolve_widget_visual, UiComponent, UiComponentVisual, UiInteractionState, UiLabel, UiManager,
    UiRenderCommand, UiStyleOverrides, UiTheme, UiVisualStyle, UI_DEFAULT_FONT_FAMILY,
};

#[test]
fn theme_light_and_dark_have_distinct_palette_and_defaults() {
    let light = UiTheme::light();
    let dark = UiTheme::dark();

    assert_ne!(light.palette.surface, dark.palette.surface);
    assert_eq!(light.typography.default_font_family, UI_DEFAULT_FONT_FAMILY);
    assert_eq!(dark.typography.default_font_family, UI_DEFAULT_FONT_FAMILY);
    assert!(light.typography.default_font_size > 0.0);
    assert!(light.spacing.medium > 0.0);
}

#[test]
fn theme_visual_resolution_uses_state_palette() {
    let theme = UiTheme::dark();

    let hovered = resolve_widget_visual(
        UiComponentVisual::Button,
        UiInteractionState::Hovered,
        &theme,
        None,
    );

    assert_eq!(hovered.background, theme.widgets.button.hovered.background);
}

#[test]
fn theme_visual_resolution_applies_style_overrides() {
    let theme = UiTheme::light();
    let overrides = UiStyleOverrides {
        background_color: Some(Color::RED),
        foreground_color: Some(Color::BLUE),
        border_color: Some(Color::GREEN),
        ..UiStyleOverrides::default()
    };

    let resolved = resolve_widget_visual(
        UiComponentVisual::Label,
        UiInteractionState::Normal,
        &theme,
        Some(&overrides),
    );

    assert_eq!(
        resolved,
        UiVisualStyle {
            background: Color::RED,
            text: Color::BLUE,
            border: Color::GREEN,
        }
    );
}

#[test]
fn theme_runtime_switch_updates_rendered_visuals() {
    let mut ui = UiManager::new();
    let panel = ui.create_node(Some(UiComponent::Panel));
    let label = ui.create_node(Some(UiComponent::Label(UiLabel::new("Theme Label"))));
    ui.set_parent(label, Some(panel)).unwrap();
    ui.get_node_mut(panel)
        .unwrap()
        .set_size(Vec2::new(120.0, 40.0));
    ui.get_node_mut(label)
        .unwrap()
        .set_size(Vec2::new(100.0, 20.0));
    ui.set_viewport_size(200, 100);
    ui.update();
    let initial_epoch = ui.layout_epoch();

    let light_quads: Vec<_> = ui
        .build_render_commands()
        .into_iter()
        .filter_map(|command| match command {
            UiRenderCommand::Quad(quad) if quad.node_id == panel => Some(quad),
            _ => None,
        })
        .collect();
    let light_label_text = ui
        .build_render_commands()
        .into_iter()
        .find_map(|command| match command {
            UiRenderCommand::Text(text) if text.node_id == label => Some(text),
            _ => None,
        })
        .expect("label text command should exist with light theme");

    ui.set_theme(UiTheme::dark());

    let dark_quads: Vec<_> = ui
        .build_render_commands()
        .into_iter()
        .filter_map(|command| match command {
            UiRenderCommand::Quad(quad) if quad.node_id == panel => Some(quad),
            _ => None,
        })
        .collect();
    let dark_label_text = ui
        .build_render_commands()
        .into_iter()
        .find_map(|command| match command {
            UiRenderCommand::Text(text) if text.node_id == label => Some(text),
            _ => None,
        })
        .expect("label text command should exist with dark theme");

    assert_eq!(ui.layout_epoch(), initial_epoch + 1);
    assert_eq!(light_quads.len(), 2);
    assert_eq!(dark_quads.len(), 2);
    assert_ne!(light_quads[0].color, dark_quads[0].color);
    assert_eq!(dark_quads[0].color, ui.theme().widgets.panel.normal.border);
    assert_eq!(
        dark_quads[1].color,
        ui.theme().widgets.panel.normal.background
    );
    assert_ne!(light_label_text.color, dark_label_text.color);
    assert_eq!(dark_label_text.color, ui.theme().widgets.label.normal.text);
}

#[test]
fn theme_set_same_theme_does_not_invalidate_layout_epoch() {
    let mut ui = UiManager::new();
    let panel = ui.create_node(Some(UiComponent::Panel));
    ui.get_node_mut(panel)
        .unwrap()
        .set_size(Vec2::new(40.0, 20.0));
    ui.set_viewport_size(200, 100);
    ui.update();

    let epoch_before = ui.layout_epoch();
    ui.set_theme(ui.theme().clone());
    let _ = ui.build_render_commands();

    assert_eq!(ui.layout_epoch(), epoch_before);
}
