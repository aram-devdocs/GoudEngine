//! Feature Lab -- Rust SDK Headless Smoke Example
//!
//! This example exercises Rust SDK surface that the playable Flappy Bird sample
//! does not cover yet. It stays headless on purpose so it can validate scenes,
//! ECS composition, animation/input helpers, provider queries, and safe native
//! fallbacks without requiring a window or assets to load successfully.

use goudengine::{
    components::{
        AnimationClip, AudioChannel, AudioSource, Collider, Name, PlaybackMode, RigidBody, Sprite,
        SpriteAnimator, Transform2D,
    },
    input::{Key, MouseButton},
    GameConfig, GoudGame, Rect, Vec2,
};

struct CheckResult {
    name: &'static str,
    passed: bool,
}

fn record(results: &mut Vec<CheckResult>, name: &'static str, passed: bool) {
    results.push(CheckResult { name, passed });
}

fn main() {
    let config = GameConfig::new("Rust Feature Lab", 1280, 720)
        .with_target_fps(120)
        .with_fps_overlay(true)
        .with_physics_debug(true);
    let mut game = GoudGame::new(config).expect("failed to create headless game");

    let mut results = Vec::new();
    run_feature_surface_checks(&mut game, &mut results);

    let pass_count = results.iter().filter(|result| result.passed).count();
    let fail_count = results.len() - pass_count;

    println!("Feature Lab complete: {pass_count} pass, {fail_count} fail");
    for result in &results {
        let status = if result.passed { "PASS" } else { "FAIL" };
        println!("{status}: {}", result.name);
    }

    if fail_count > 0 {
        std::process::exit(1);
    }
}

fn run_feature_surface_checks(game: &mut GoudGame, results: &mut Vec<CheckResult>) {
    let default_scene = game.scene_manager().default_scene();
    let sandbox_scene = game.create_scene("sandbox").expect("sandbox scene");

    record(
        results,
        "default scene is created automatically",
        game.scene_by_name("default") == Some(default_scene),
    );
    record(
        results,
        "new scenes can be created, named, and activated",
        game.scene_manager().scene_count() == 2
            && game.scene_manager().get_scene_name(sandbox_scene) == Some("sandbox")
            && game.set_scene_active(sandbox_scene, true).is_ok()
            && game.scene_manager().is_active(sandbox_scene),
    );
    record(
        results,
        "duplicate scene names and destroying default scene are rejected",
        game.create_scene("sandbox").is_err() && game.destroy_scene(default_scene).is_err(),
    );

    if let Some(scene) = game.scene_mut(sandbox_scene) {
        let probe = scene.spawn_empty();
        scene.insert(probe, Name::new("sandbox_probe"));
    }
    record(
        results,
        "scene worlds stay isolated from the default scene",
        game.entity_count() == 0
            && game
                .scene(sandbox_scene)
                .is_some_and(|scene| scene.entity_count() == 1),
    );

    let player = game
        .spawn()
        .with(Transform2D::from_position(Vec2::new(10.0, 20.0)))
        .with(Name::new("feature_player"))
        .with(Sprite::default().with_texture_path(
            "examples/csharp/flappy_goud/assets/sprites/yellowbird-downflap.png",
        ))
        .with(Collider::aabb(Vec2::new(8.0, 12.0)).with_density(0.5))
        .with(RigidBody::dynamic().with_velocity(Vec2::new(1.0, 0.0)))
        .with(
            AudioSource::default()
                .with_audio_path("examples/csharp/flappy_goud/assets/audio/wing.wav")
                .with_channel(AudioChannel::SFX)
                .with_volume(0.75),
        )
        .build();

    let animator_entity =
        game.spawn()
            .with(Transform2D::default())
            .with(Sprite::default().with_texture_path(
                "examples/csharp/flappy_goud/assets/sprites/yellowbird-midflap.png",
            ))
            .with(SpriteAnimator::new(
                AnimationClip::new(
                    vec![Rect::new(0.0, 0.0, 8.0, 8.0), Rect::new(8.0, 0.0, 8.0, 8.0)],
                    0.1,
                )
                .with_mode(PlaybackMode::OneShot),
            ))
            .build();

    record(
        results,
        "entity builder composes render, physics, audio, and animation components",
        game.has::<Transform2D>(player)
            && game.has::<Name>(player)
            && game.has::<Sprite>(player)
            && game.has::<Collider>(player)
            && game.has::<RigidBody>(player)
            && game.has::<AudioSource>(player)
            && game.has::<SpriteAnimator>(animator_entity),
    );

    game.update_frame(0.016, |_, world| {
        if let Some(transform) = world.get_mut::<Transform2D>(player) {
            transform.position.x += 1.0;
        }
        if let Some(animator) = world.get_mut::<SpriteAnimator>(animator_entity) {
            animator.pause();
            animator.reset();
        }
        if let Some(audio) = world.get_mut::<AudioSource>(player) {
            audio.play();
            audio.pause();
        }
    });

    let moved = game
        .get::<Transform2D>(player)
        .is_some_and(|transform| transform.position.x > 10.0);
    let animator_reset = game
        .get::<SpriteAnimator>(animator_entity)
        .is_some_and(|animator| {
            matches!(animator.clip.mode, PlaybackMode::OneShot)
                && animator.current_frame == 0
                && !animator.playing
                && animator.current_rect().is_some()
        });
    let removed_name = game
        .remove::<Name>(player)
        .is_some_and(|name| name.as_str() == "feature_player");
    record(
        results,
        "components can be mutated during updates and removed afterward",
        moved && animator_reset && removed_name,
    );

    game.map_action_key("Jump", Key::Space);
    game.map_action_mouse_button("Select", MouseButton::Button1);
    game.input_mut().press_key(Key::Space);
    game.input_mut().press_mouse_button(MouseButton::Button1);
    game.input_mut().set_mouse_position(Vec2::new(32.0, 48.0));
    game.input_mut().add_scroll_delta(Vec2::new(0.0, 1.0));

    let input_ok = game.is_key_pressed(Key::Space)
        && game.is_action_pressed("Jump")
        && game.is_mouse_button_pressed(MouseButton::Button1)
        && game.is_action_pressed("Select")
        && game.mouse_position() == (32.0, 48.0)
        && game.mouse_delta() == (32.0, 48.0)
        && game.scroll_delta() == (0.0, 1.0);

    game.input_mut().release_key(Key::Space);
    game.input_mut().release_mouse_button(MouseButton::Button1);

    record(
        results,
        "input mapping and pointer state are queryable headless",
        input_ok,
    );

    let render_cube = game.create_cube(0, 1.0, 1.0, 1.0);
    let light = game.add_light(
        0, 0.0, 1.0, 2.0, 0.0, -1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 5.0, 0.0,
    );

    record(
        results,
        "headless window and renderer APIs fail safely",
        !game.has_platform()
            && !game.has_2d_renderer()
            && !game.has_3d_renderer()
            && game.poll_events().is_err()
            && game.swap_buffers().is_err()
            && game.set_should_close(true).is_err()
            && !game.begin_render()
            && game.begin_2d_render().is_err()
            && !game.end_render()
            && game.end_2d_render().is_err()
            && game.draw_sprites().is_err()
            && render_cube == u32::MAX
            && !game.set_object_position(render_cube, 1.0, 2.0, 3.0)
            && !game.set_camera_position(0.0, 0.0, 0.0)
            && !game.configure_grid(true, 10.0, 4)
            && light == u32::MAX,
    );

    let render_caps = format!("{:?}", game.render_capabilities());
    let physics_caps = format!("{:?}", game.physics_capabilities());
    let audio_caps = format!("{:?}", game.audio_capabilities());
    let input_caps = format!("{:?}", game.input_capabilities());
    let network_caps = game
        .network_capabilities()
        .map(|caps| format!("{caps:?}"))
        .unwrap_or_else(|| "None".to_string());

    record(
        results,
        "provider capability queries and native metadata are available",
        !render_caps.is_empty()
            && !physics_caps.is_empty()
            && !audio_caps.is_empty()
            && !input_caps.is_empty()
            && !network_caps.is_empty()
            && game.title() == "Rust Feature Lab"
            && game.get_window_size() == (1280, 720)
            && game.get_framebuffer_size() == (1280, 720)
            && game.get_delta_time() > 0.0
            && game.fps_stats().frame_time_ms >= 0.0,
    );

    let missing_texture = game.load("examples/does-not-exist.png");
    record(
        results,
        "texture APIs return sentinels without an initialized renderer",
        missing_texture == u64::MAX && !game.destroy(missing_texture),
    );

    record(
        results,
        "non-default scenes can be deactivated and destroyed cleanly",
        game.set_scene_active(sandbox_scene, false).is_ok()
            && !game.scene_manager().is_active(sandbox_scene)
            && game.destroy_scene(sandbox_scene).is_ok()
            && game.scene_manager().scene_count() == 1,
    );
}
