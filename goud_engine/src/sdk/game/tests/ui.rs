use super::*;

#[test]
fn test_update_frame_ui_consumes_mouse_event_before_game_queries() {
    let mut game = GoudGame::default();
    let button = game
        .ui_manager_mut()
        .create_node(Some(UiComponent::Button(UiButton::default())));

    {
        let node = game.ui_manager_mut().get_node_mut(button).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(100.0, 40.0));
    }

    game.input_mut().set_mouse_position(Vec2::new(10.0, 10.0));
    game.input_mut().press_mouse_button(MouseButton::Button1);

    game.update_frame(0.016, |_ctx, _world| {});

    assert!(!game.is_mouse_button_just_pressed(MouseButton::Button1));
    assert!(!game.is_mouse_button_pressed(MouseButton::Button1));
}

#[test]
fn test_update_frame_ui_consumes_tab_and_enter_activation() {
    let mut game = GoudGame::default();
    let button = game
        .ui_manager_mut()
        .create_node(Some(UiComponent::Button(UiButton::default())));

    {
        let node = game.ui_manager_mut().get_node_mut(button).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(100.0, 40.0));
    }

    game.input_mut().press_key(Key::Tab);
    game.update_frame(0.016, |_ctx, _world| {});
    assert!(!game.is_key_just_pressed(Key::Tab));
    game.ui_manager_mut().take_events();

    game.input_mut().update();
    game.input_mut().release_key(Key::Tab);
    game.update_frame(0.016, |_ctx, _world| {});
    game.ui_manager_mut().take_events();

    game.input_mut().update();
    game.input_mut().press_key(Key::Enter);
    game.update_frame(0.016, |_ctx, _world| {});

    assert!(!game.is_key_just_pressed(Key::Enter));
    let events = game.ui_manager_mut().take_events();
    assert!(events
        .iter()
        .any(|event| matches!(event, crate::ui::UiEvent::Click(id) if *id == button)));
}

#[test]
fn test_update_frame_headless_with_ui_render_commands_is_safe() {
    let mut game = GoudGame::default();

    let panel = game.ui_manager_mut().create_node(Some(UiComponent::Panel));
    let label = game
        .ui_manager_mut()
        .create_node(Some(UiComponent::Label(UiLabel::new("Headless UI"))));
    let image = game
        .ui_manager_mut()
        .create_node(Some(UiComponent::Image(UiImage::new(
            "ui://fixture-checker",
        ))));

    game.ui_manager_mut()
        .set_parent(label, Some(panel))
        .unwrap();
    game.ui_manager_mut()
        .set_parent(image, Some(panel))
        .unwrap();

    game.ui_manager_mut()
        .get_node_mut(panel)
        .unwrap()
        .set_size(Vec2::new(320.0, 100.0));
    game.ui_manager_mut()
        .get_node_mut(label)
        .unwrap()
        .set_size(Vec2::new(220.0, 24.0));
    game.ui_manager_mut()
        .get_node_mut(image)
        .unwrap()
        .set_size(Vec2::new(64.0, 64.0));

    game.update_frame(0.016, |_ctx, _world| {});

    let commands = game.ui_manager_mut().build_render_commands();
    assert!(!commands.is_empty());
}

#[test]
fn test_update_frame_renders_ui_after_scene_callback_mutates_ui() {
    let mut game = GoudGame::default();
    let label = game
        .ui_manager_mut()
        .create_node(Some(UiComponent::Label(UiLabel::new(""))));
    let ui_manager: *mut UiManager = game.ui_manager_mut() as *mut UiManager;

    game.update_frame(0.016, move |_ctx, _world| {
        // SAFETY: The callback runs synchronously on the same thread. This test
        // only mutates `ui_manager`, which is a disjoint field from the `world`
        // borrow handed to the callback.
        let node = unsafe { (&mut *ui_manager).get_node_mut(label).unwrap() };
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(96.0, 24.0));
        node.set_component(Some(UiComponent::Label(UiLabel::new("Late HUD"))));
    });

    assert!(
        game.last_ui_command_count() > 0,
        "render_ui_frame should see UI mutations performed inside the scene callback"
    );
}

#[test]
fn test_update_frame_ui_commands_remain_screen_space_when_world_moves() {
    let mut game = GoudGame::default();
    let panel = game.ui_manager_mut().create_node(Some(UiComponent::Panel));
    game.ui_manager_mut()
        .get_node_mut(panel)
        .unwrap()
        .set_anchor(UiAnchor::TopLeft);
    game.ui_manager_mut()
        .get_node_mut(panel)
        .unwrap()
        .set_size(Vec2::new(80.0, 32.0));

    let entity = game
        .spawn()
        .with(Transform2D::from_position(Vec2::new(16.0, 24.0)))
        .build();

    game.update_frame(0.016, |_ctx, world| {
        let transform = world.get_mut::<Transform2D>(entity).unwrap();
        transform.position = Vec2::new(320.0, 240.0);
    });
    let commands_after_first_move = game.ui_manager_mut().build_render_commands();

    game.update_frame(0.016, |_ctx, world| {
        let transform = world.get_mut::<Transform2D>(entity).unwrap();
        transform.position = Vec2::new(-480.0, 512.0);
    });
    let commands_after_second_move = game.ui_manager_mut().build_render_commands();

    let first_rect = commands_after_first_move
        .iter()
        .find_map(|command| match command {
            UiRenderCommand::Quad(quad) if quad.node_id == panel => Some(quad.rect),
            _ => None,
        })
        .expect("expected panel quad after first move");
    let second_rect = commands_after_second_move
        .iter()
        .find_map(|command| match command {
            UiRenderCommand::Quad(quad) if quad.node_id == panel => Some(quad.rect),
            _ => None,
        })
        .expect("expected panel quad after second move");

    assert_eq!(first_rect, second_rect);
    assert_eq!(first_rect.x, 0.0);
    assert_eq!(first_rect.y, 0.0);
}
