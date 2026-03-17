use super::*;
use crate::core::math::{Color, Vec2};
use crate::core::providers::input_types::MouseButton;
use crate::ecs::InputManager;
use crate::ui::{
    resolve_widget_visual, UiComponentVisual, UiImage, UiInteractionState, UiLabel,
    UiRenderCommand, UiSlider, UiStyleOverrides,
};

fn quad_commands_for_node(
    commands: &[UiRenderCommand],
    node_id: crate::ui::UiNodeId,
) -> Vec<crate::ui::UiQuadCommand> {
    commands
        .iter()
        .filter_map(|cmd| match cmd {
            UiRenderCommand::Quad(quad) if quad.node_id == node_id => Some(*quad),
            _ => None,
        })
        .collect()
}

#[test]
fn render_pass_emits_quad_textured_quad_and_text_requests() {
    let mut ui = UiManager::new();
    let panel = ui.create_node(Some(UiComponent::Panel));
    let image = ui.create_node(Some(UiComponent::Image(UiImage::new("assets/ui/icon.png"))));
    let label = ui.create_node(Some(UiComponent::Label(UiLabel::new("Play"))));

    ui.set_parent(image, Some(panel)).unwrap();
    ui.set_parent(label, Some(panel)).unwrap();

    ui.get_node_mut(panel)
        .unwrap()
        .set_size(Vec2::new(200.0, 120.0));
    ui.get_node_mut(image)
        .unwrap()
        .set_size(Vec2::new(64.0, 64.0));
    ui.get_node_mut(label)
        .unwrap()
        .set_size(Vec2::new(120.0, 24.0));
    ui.get_node_mut(label)
        .unwrap()
        .set_margin(UiEdges::new(70.0, 0.0, 0.0, 0.0));

    ui.set_viewport_size(300, 200);
    ui.update();

    let commands = ui.build_render_commands();

    assert!(commands
        .iter()
        .any(|cmd| matches!(cmd, UiRenderCommand::Quad(_))));
    assert!(commands
        .iter()
        .any(|cmd| matches!(cmd, UiRenderCommand::TexturedQuad(_))));
    assert!(commands
        .iter()
        .any(|cmd| matches!(cmd, UiRenderCommand::Text(_))));
}

#[test]
fn render_panel_and_button_emit_border_and_fill_quads() {
    let mut ui = UiManager::new();
    let panel = ui.create_node(Some(UiComponent::Panel));
    let button = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    ui.get_node_mut(panel)
        .unwrap()
        .set_size(Vec2::new(120.0, 64.0));
    ui.get_node_mut(button)
        .unwrap()
        .set_size(Vec2::new(140.0, 44.0));
    ui.get_node_mut(button)
        .unwrap()
        .set_margin(UiEdges::new(0.0, 80.0, 0.0, 0.0));
    ui.get_node_mut(button)
        .unwrap()
        .set_style_overrides(UiStyleOverrides {
            border_width: Some(6.0),
            ..UiStyleOverrides::default()
        });

    ui.set_viewport_size(320, 180);
    ui.update();

    let commands = ui.build_render_commands();
    let panel_quads = quad_commands_for_node(&commands, panel);
    let button_quads = quad_commands_for_node(&commands, button);

    assert_eq!(panel_quads.len(), 2);
    assert_eq!(
        panel_quads[0].rect,
        crate::core::math::Rect::new(0.0, 0.0, 120.0, 64.0)
    );
    assert_eq!(panel_quads[0].color, ui.theme().widgets.panel.normal.border);
    assert_eq!(
        panel_quads[1].rect,
        crate::core::math::Rect::new(4.0, 4.0, 112.0, 56.0)
    );
    assert_eq!(
        panel_quads[1].color,
        ui.theme().widgets.panel.normal.background
    );

    assert_eq!(button_quads.len(), 2);
    assert_eq!(
        button_quads[0].rect,
        crate::core::math::Rect::new(0.0, 0.0, 140.0, 44.0)
    );
    assert_eq!(
        button_quads[0].color,
        ui.theme().widgets.button.normal.border
    );
    assert_eq!(
        button_quads[1].rect,
        crate::core::math::Rect::new(6.0, 6.0, 128.0, 32.0)
    );
    assert_eq!(
        button_quads[1].color,
        ui.theme().widgets.button.normal.background
    );
}

#[test]
fn render_button_text_is_compositional_via_child_label() {
    let mut ui = UiManager::new();
    let button = ui.create_node(Some(UiComponent::Button(UiButton::default())));
    let label = ui.create_node(Some(UiComponent::Label(UiLabel::new("Launch"))));

    ui.set_parent(label, Some(button)).unwrap();

    ui.get_node_mut(button)
        .unwrap()
        .set_size(Vec2::new(180.0, 48.0));
    ui.get_node_mut(label)
        .unwrap()
        .set_size(Vec2::new(120.0, 22.0));

    ui.set_viewport_size(320, 180);
    ui.update();

    let commands = ui.build_render_commands();
    let text_count = commands
        .iter()
        .filter(|cmd| matches!(cmd, UiRenderCommand::Text(_)))
        .count();

    assert_eq!(text_count, 1);
}

#[test]
fn render_button_visual_state_resolves_pressed_and_disabled() {
    let mut ui = UiManager::new();
    let button = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    ui.get_node_mut(button)
        .unwrap()
        .set_size(Vec2::new(100.0, 40.0));
    ui.set_viewport_size(300, 200);
    ui.update();

    let mut input = InputManager::new();
    input.set_mouse_position(Vec2::new(20.0, 20.0));
    input.press_mouse_button(MouseButton::Button1);
    ui.process_input_frame(&mut input);

    let pressed_commands = ui.build_render_commands();
    let pressed_quads = quad_commands_for_node(&pressed_commands, button);
    assert_eq!(pressed_quads.len(), 2);

    let pressed_expected = resolve_widget_visual(
        UiComponentVisual::Button,
        ui.interaction_state_for(button),
        ui.theme(),
        ui.get_node(button).and_then(|node| node.style_overrides()),
    );
    assert_eq!(pressed_quads[0].color, pressed_expected.border);
    assert_eq!(pressed_quads[1].color, pressed_expected.background);

    ui.get_node_mut(button)
        .unwrap()
        .set_component(Some(UiComponent::Button(crate::ui::UiButton {
            enabled: false,
        })));

    let disabled_commands = ui.build_render_commands();
    let disabled_quads = quad_commands_for_node(&disabled_commands, button);
    assert_eq!(disabled_quads.len(), 2);

    let disabled_expected = resolve_widget_visual(
        UiComponentVisual::Button,
        UiInteractionState::Disabled,
        ui.theme(),
        None,
    );
    assert_eq!(disabled_quads[0].color, disabled_expected.border);
    assert_eq!(disabled_quads[1].color, disabled_expected.background);
}

#[test]
fn render_slider_emits_track_fill_and_knob_using_theme_spacing_and_overrides() {
    let mut ui = UiManager::new();
    let slider = ui.create_node(Some(UiComponent::Slider(UiSlider::new(-10.0, 10.0, 0.0))));
    ui.get_node_mut(slider)
        .unwrap()
        .set_size(Vec2::new(200.0, 20.0));
    ui.get_node_mut(slider)
        .unwrap()
        .set_style_overrides(UiStyleOverrides {
            foreground_color: Some(Color::YELLOW),
            border_color: Some(Color::GREEN),
            widget_spacing: Some(5.0),
            ..UiStyleOverrides::default()
        });

    ui.set_viewport_size(300, 100);
    ui.update();

    let commands = ui.build_render_commands();
    let slider_quads = quad_commands_for_node(&commands, slider);

    assert_eq!(slider_quads.len(), 3);
    assert_eq!(
        slider_quads[0].rect,
        crate::core::math::Rect::new(5.0, 8.0, 190.0, 4.0)
    );
    assert_eq!(
        slider_quads[0].color,
        ui.theme().widgets.slider.normal.background
    );
    assert_eq!(
        slider_quads[1].rect,
        crate::core::math::Rect::new(5.0, 8.0, 95.0, 4.0)
    );
    assert_eq!(slider_quads[1].color, Color::YELLOW);
    assert_eq!(
        slider_quads[2].rect,
        crate::core::math::Rect::new(95.0, 5.0, 10.0, 10.0)
    );
    assert_eq!(slider_quads[2].color, Color::GREEN);
}

#[test]
fn render_slider_knob_stays_robust_when_rect_is_smaller_than_knob() {
    let mut ui = UiManager::new();
    let slider = ui.create_node(Some(UiComponent::Slider(UiSlider::new(0.0, 10.0, 5.0))));
    ui.get_node_mut(slider)
        .unwrap()
        .set_size(Vec2::new(0.5, 0.5));
    ui.get_node_mut(slider)
        .unwrap()
        .set_style_overrides(UiStyleOverrides {
            widget_spacing: Some(16.0),
            ..UiStyleOverrides::default()
        });

    ui.set_viewport_size(20, 20);
    ui.update();

    let commands = ui.build_render_commands();
    let slider_quads = quad_commands_for_node(&commands, slider);
    assert_eq!(slider_quads.len(), 3);
    assert_eq!(
        slider_quads[2].rect,
        crate::core::math::Rect::new(0.0, 0.0, 1.0, 1.0)
    );
}

#[test]
fn render_top_left_anchor_stays_at_viewport_origin_after_resize() {
    let mut ui = UiManager::new();
    let panel = ui.create_node(Some(UiComponent::Panel));
    ui.get_node_mut(panel)
        .unwrap()
        .set_anchor(UiAnchor::TopLeft);
    ui.get_node_mut(panel)
        .unwrap()
        .set_size(Vec2::new(80.0, 32.0));

    ui.set_viewport_size(320, 180);
    ui.update();
    let initial_commands = ui.build_render_commands();

    ui.set_viewport_size(1920, 1080);
    ui.update();
    let resized_commands = ui.build_render_commands();

    let initial_rect = quad_commands_for_node(&initial_commands, panel)
        .into_iter()
        .next()
        .expect("top-left panel should emit a quad")
        .rect;
    let resized_rect = quad_commands_for_node(&resized_commands, panel)
        .into_iter()
        .next()
        .expect("resized top-left panel should emit a quad")
        .rect;

    assert_eq!(initial_rect, resized_rect);
    assert_eq!(
        initial_rect,
        crate::core::math::Rect::new(0.0, 0.0, 80.0, 32.0)
    );
}
