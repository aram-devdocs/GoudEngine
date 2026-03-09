use super::*;

#[test]
fn input_click_dispatches_to_topmost_button_once() {
    let mut ui = UiManager::new();
    let back = ui.create_node(Some(UiComponent::Button(UiButton::default())));
    let front = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    for id in [back, front] {
        let node = ui.get_node_mut(id).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(120.0, 80.0));
    }

    ui.set_viewport_size(300, 200);
    ui.update();

    let mut input = InputManager::new();
    input.set_mouse_position(Vec2::new(20.0, 20.0));

    input.press_mouse_button(MouseButton::Button1);
    ui.process_input_frame(&mut input);
    let first_frame_events = ui.take_events();
    assert!(!first_frame_events
        .iter()
        .any(|e| matches!(e, UiEvent::Click(_))));

    input.update();
    input.release_mouse_button(MouseButton::Button1);
    ui.process_input_frame(&mut input);

    assert_eq!(ui.take_events(), vec![UiEvent::Click(front)]);

    input.update();
    ui.process_input_frame(&mut input);
    assert!(ui.take_events().is_empty());
}

#[test]
fn input_hover_emits_leave_then_enter_when_target_changes() {
    let mut ui = UiManager::new();
    let left = ui.create_node(Some(UiComponent::Button(UiButton::default())));
    let right = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    {
        let node = ui.get_node_mut(left).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(40.0, 40.0));
    }

    {
        let node = ui.get_node_mut(right).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(40.0, 40.0));
        node.set_margin(UiEdges::new(50.0, 0.0, 0.0, 0.0));
    }

    ui.set_viewport_size(200, 200);
    ui.update();

    let mut input = InputManager::new();

    input.set_mouse_position(Vec2::new(10.0, 10.0));
    ui.process_input_frame(&mut input);
    assert_eq!(ui.take_events(), vec![UiEvent::HoverEnter(left)]);

    input.update();
    input.set_mouse_position(Vec2::new(60.0, 10.0));
    ui.process_input_frame(&mut input);
    assert_eq!(
        ui.take_events(),
        vec![UiEvent::HoverLeave(left), UiEvent::HoverEnter(right)]
    );
}

#[test]
fn input_tab_focus_traversal_wraps_and_supports_shift_tab() {
    let mut ui = UiManager::new();
    let a = ui.create_node(Some(UiComponent::Button(UiButton::default())));
    let b = ui.create_node(Some(UiComponent::Button(UiButton::default())));
    let c = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    ui.set_parent(b, Some(a)).unwrap();
    ui.set_parent(c, Some(a)).unwrap();

    ui.set_viewport_size(200, 200);
    ui.update();

    let mut input = InputManager::new();

    input.press_key(Key::Tab);
    ui.process_input_frame(&mut input);
    assert_eq!(ui.focused_node(), Some(a));

    input.update();
    input.release_key(Key::Tab);
    ui.process_input_frame(&mut input);
    ui.take_events();

    input.update();
    input.press_key(Key::Tab);
    ui.process_input_frame(&mut input);
    assert_eq!(ui.focused_node(), Some(b));

    input.update();
    input.release_key(Key::Tab);
    ui.process_input_frame(&mut input);
    ui.take_events();

    input.update();
    input.press_key(Key::LeftShift);
    input.press_key(Key::Tab);
    ui.process_input_frame(&mut input);
    assert_eq!(ui.focused_node(), Some(a));
}

#[test]
fn input_enter_and_space_activate_focused_button() {
    let mut ui = UiManager::new();
    let button = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    {
        let node = ui.get_node_mut(button).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(40.0, 30.0));
    }

    ui.set_viewport_size(100, 100);
    ui.update();

    let mut input = InputManager::new();

    input.press_key(Key::Tab);
    ui.process_input_frame(&mut input);
    ui.take_events();
    assert_eq!(ui.focused_node(), Some(button));

    input.update();
    input.release_key(Key::Tab);
    ui.process_input_frame(&mut input);
    ui.take_events();

    input.update();
    input.press_key(Key::Enter);
    ui.process_input_frame(&mut input);
    assert_eq!(ui.take_events(), vec![UiEvent::Click(button)]);

    input.update();
    input.release_key(Key::Enter);
    ui.process_input_frame(&mut input);
    ui.take_events();

    input.update();
    input.press_key(Key::Space);
    ui.process_input_frame(&mut input);
    assert_eq!(ui.take_events(), vec![UiEvent::Click(button)]);
}

#[test]
fn input_consumes_events_before_game_queries() {
    let mut ui = UiManager::new();
    let button = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    {
        let node = ui.get_node_mut(button).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(60.0, 40.0));
    }

    ui.set_viewport_size(200, 200);
    ui.update();

    let mut input = InputManager::new();
    input.set_mouse_position(Vec2::new(20.0, 20.0));

    input.press_mouse_button(MouseButton::Button1);
    ui.process_input_frame(&mut input);

    assert!(!input.mouse_button_pressed(MouseButton::Button1));
    assert!(!input.mouse_button_just_pressed(MouseButton::Button1));
    ui.process_input_frame(&mut input);
    assert!(!input.mouse_button_pressed(MouseButton::Button1));
    assert!(!input.mouse_button_just_pressed(MouseButton::Button1));

    input.update();
    input.release_mouse_button(MouseButton::Button1);
    ui.process_input_frame(&mut input);

    assert_eq!(ui.take_events(), vec![UiEvent::Click(button)]);
    assert!(!input.mouse_button_just_released(MouseButton::Button1));

    input.update();
    input.press_key(Key::Tab);
    ui.process_input_frame(&mut input);
    assert!(!input.key_just_pressed(Key::Tab));
    ui.process_input_frame(&mut input);
    assert!(!input.key_just_pressed(Key::Tab));

    input.update();
    input.release_key(Key::Tab);
    ui.process_input_frame(&mut input);

    input.update();
    input.press_key(Key::Enter);
    ui.process_input_frame(&mut input);

    assert_eq!(ui.take_events(), vec![UiEvent::Click(button)]);
    assert!(!input.key_just_pressed(Key::Enter));
}

#[test]
fn input_drag_off_button_release_is_consumed_without_click() {
    let mut ui = UiManager::new();
    let button = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    {
        let node = ui.get_node_mut(button).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(60.0, 40.0));
    }

    ui.set_viewport_size(200, 200);
    ui.update();

    let mut input = InputManager::new();
    input.set_mouse_position(Vec2::new(20.0, 20.0));
    input.press_mouse_button(MouseButton::Button1);
    ui.process_input_frame(&mut input);
    ui.take_events();

    input.update();
    input.set_mouse_position(Vec2::new(150.0, 150.0));
    input.release_mouse_button(MouseButton::Button1);
    ui.process_input_frame(&mut input);

    let release_events = ui.take_events();
    assert!(!release_events
        .iter()
        .any(|event| matches!(event, UiEvent::Click(_))));
    assert!(!input.mouse_button_just_released(MouseButton::Button1));
}

#[test]
fn input_cleanup_emits_events_for_stale_hover_and_focus() {
    let mut ui = UiManager::new();
    let button = ui.create_node(Some(UiComponent::Button(UiButton::default())));
    {
        let node = ui.get_node_mut(button).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(60.0, 40.0));
    }
    ui.set_viewport_size(200, 200);
    ui.update();

    let mut input = InputManager::new();
    input.set_mouse_position(Vec2::new(20.0, 20.0));
    ui.process_input_frame(&mut input);
    ui.take_events();

    input.update();
    input.press_key(Key::Tab);
    ui.process_input_frame(&mut input);
    ui.take_events();
    assert_eq!(ui.focused_node(), Some(button));

    ui.get_node_mut(button).unwrap().set_input_enabled(false);
    ui.process_input_frame(&mut input);
    assert_eq!(
        ui.take_events(),
        vec![
            UiEvent::HoverLeave(button),
            UiEvent::FocusChanged {
                previous: Some(button),
                current: None,
            },
        ]
    );
}

#[test]
fn input_hit_test_respects_ancestor_visibility_and_input_enabled() {
    let mut ui = UiManager::new();
    let parent = ui.create_node(Some(UiComponent::Panel));
    let child = ui.create_node(Some(UiComponent::Button(UiButton::default())));
    ui.set_parent(child, Some(parent)).unwrap();
    ui.get_node_mut(parent)
        .unwrap()
        .set_size(Vec2::new(120.0, 80.0));
    ui.get_node_mut(child)
        .unwrap()
        .set_size(Vec2::new(60.0, 40.0));
    ui.set_viewport_size(200, 200);
    ui.update();

    let mut input = InputManager::new();
    input.set_mouse_position(Vec2::new(20.0, 20.0));
    ui.process_input_frame(&mut input);
    assert_eq!(ui.take_events(), vec![UiEvent::HoverEnter(child)]);

    ui.get_node_mut(parent).unwrap().set_input_enabled(false);
    ui.process_input_frame(&mut input);
    assert_eq!(ui.take_events(), vec![UiEvent::HoverLeave(child)]);

    ui.get_node_mut(parent).unwrap().set_input_enabled(true);
    ui.process_input_frame(&mut input);
    assert_eq!(ui.take_events(), vec![UiEvent::HoverEnter(child)]);

    ui.get_node_mut(parent).unwrap().set_visible(false);
    ui.process_input_frame(&mut input);
    assert_eq!(ui.take_events(), vec![UiEvent::HoverLeave(child)]);
}

#[test]
fn input_tab_focus_skips_buttons_with_non_interactive_ancestors() {
    let mut ui = UiManager::new();
    let parent = ui.create_node(Some(UiComponent::Panel));
    let blocked_child = ui.create_node(Some(UiComponent::Button(UiButton::default())));
    let enabled_button = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    ui.set_parent(blocked_child, Some(parent)).unwrap();
    ui.get_node_mut(parent).unwrap().set_input_enabled(false);

    ui.set_viewport_size(200, 200);
    ui.update();

    let mut input = InputManager::new();
    input.press_key(Key::Tab);
    ui.process_input_frame(&mut input);

    assert_eq!(ui.focused_node(), Some(enabled_button));
    assert_eq!(
        ui.take_events(),
        vec![UiEvent::FocusChanged {
            previous: None,
            current: Some(enabled_button),
        }]
    );
}

#[test]
fn input_focus_is_cleared_and_activation_blocked_when_ancestor_becomes_non_interactive() {
    let mut ui = UiManager::new();
    let parent = ui.create_node(Some(UiComponent::Panel));
    let button = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    ui.set_parent(button, Some(parent)).unwrap();

    {
        let node = ui.get_node_mut(parent).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(120.0, 80.0));
    }
    {
        let node = ui.get_node_mut(button).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(60.0, 40.0));
    }

    ui.set_viewport_size(200, 200);
    ui.update();

    let mut input = InputManager::new();
    input.press_key(Key::Tab);
    ui.process_input_frame(&mut input);
    ui.take_events();
    assert_eq!(ui.focused_node(), Some(button));

    input.update();
    input.release_key(Key::Tab);
    ui.process_input_frame(&mut input);
    ui.take_events();

    ui.get_node_mut(parent).unwrap().set_visible(false);

    input.update();
    input.press_key(Key::Enter);
    ui.process_input_frame(&mut input);

    let events = ui.take_events();
    assert_eq!(
        events,
        vec![
            UiEvent::HoverLeave(button),
            UiEvent::FocusChanged {
                previous: Some(button),
                current: None,
            },
        ]
    );
    assert!(!events
        .iter()
        .any(|event| matches!(event, UiEvent::Click(_))));
    assert_eq!(ui.focused_node(), None);
}

#[test]
fn remove_node_emits_hover_leave_for_hovered_descendant() {
    let mut ui = UiManager::new();
    let parent = ui.create_node(Some(UiComponent::Panel));
    let button = ui.create_node(Some(UiComponent::Button(UiButton::default())));

    ui.set_parent(button, Some(parent)).unwrap();
    ui.get_node_mut(parent)
        .unwrap()
        .set_size(Vec2::new(120.0, 80.0));
    ui.get_node_mut(button)
        .unwrap()
        .set_size(Vec2::new(60.0, 40.0));

    ui.set_viewport_size(200, 200);
    ui.update();

    let mut input = InputManager::new();
    input.set_mouse_position(Vec2::new(20.0, 20.0));
    ui.process_input_frame(&mut input);
    assert_eq!(ui.take_events(), vec![UiEvent::HoverEnter(button)]);
    assert_eq!(ui.hovered_node(), Some(button));

    assert!(ui.remove_node(parent));
    assert_eq!(ui.hovered_node(), None);
    assert_eq!(ui.take_events(), vec![UiEvent::HoverLeave(button)]);
}
