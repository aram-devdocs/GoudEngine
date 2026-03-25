//! Integration tests for Lua scripting support.

#[cfg(feature = "lua")]
mod lua_tests {
    use goud_engine::sdk::game_config::GameConfig;
    use goud_engine::sdk::GoudGame;

    #[test]
    fn test_lua_simple_execution() {
        let game = GoudGame::new(GameConfig::default()).expect("failed to create game");
        game.execute_lua("local x = 1 + 2", "test_simple")
            .expect("simple script should succeed");
    }

    #[test]
    fn test_lua_syntax_error() {
        let game = GoudGame::new(GameConfig::default()).expect("failed to create game");
        let result = game.execute_lua("if then end end", "test_syntax");
        assert!(result.is_err(), "syntax error should produce an error");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("Script"),
            "error category should be Script, got: {err_msg}"
        );
    }

    #[test]
    fn test_lua_runtime_error() {
        let game = GoudGame::new(GameConfig::default()).expect("failed to create game");
        let result = game.execute_lua("error('boom')", "test_runtime");
        assert!(result.is_err(), "runtime error should produce an error");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("boom"),
            "error should contain the message, got: {err_msg}"
        );
    }

    #[test]
    fn test_lua_type_factories_accessible() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        let result = game.execute_lua(
            "local c = Color({ r = 1.0, g = 0.0, b = 0.0, a = 1.0 })\nassert(c.r == 1.0)",
            "test_factories.lua",
        );
        assert!(
            result.is_ok(),
            "Color factory should be accessible: {result:?}"
        );
    }

    // =========================================================================
    // Type factory tests
    // =========================================================================

    #[test]
    fn test_lua_color_factory_and_fields() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local c = Color({ r = 0.5, g = 0.6, b = 0.7, a = 1.0 })
            assert(c.r == 0.5, "Color.r mismatch")
            assert(c.a == 1.0, "Color.a mismatch")
            c.r = 0.1
            assert(c.r > 0.09 and c.r < 0.11, "Color.r set mismatch")
            "#,
            "test_color_fields",
        )
        .expect("Color factory and field access should work");
    }

    #[test]
    fn test_lua_vec2_factory_and_fields() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local v = Vec2({ x = 3.0, y = 4.0 })
            assert(v.x == 3.0, "Vec2.x mismatch")
            assert(v.y == 4.0, "Vec2.y mismatch")
            v.x = 10.0
            assert(v.x == 10.0, "Vec2.x set mismatch")
            "#,
            "test_vec2_fields",
        )
        .expect("Vec2 factory and field access should work");
    }

    #[test]
    fn test_lua_rect_factory_and_fields() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local r = Rect({ x = 1.0, y = 2.0, width = 100.0, height = 50.0 })
            assert(r.x == 1.0, "Rect.x mismatch")
            assert(r.y == 2.0, "Rect.y mismatch")
            assert(r.width == 100.0, "Rect.width mismatch")
            assert(r.height == 50.0, "Rect.height mismatch")
            "#,
            "test_rect_fields",
        )
        .expect("Rect factory and field access should work");
    }

    #[test]
    fn test_lua_transform2d_factory_and_fields() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local t = Transform2D({ position_x = 5.0, position_y = 10.0, rotation = 1.5, scale_x = 2.0, scale_y = 2.0 })
            assert(t.position_x == 5.0, "Transform2D.position_x mismatch")
            assert(t.position_y == 10.0, "Transform2D.position_y mismatch")
            assert(t.rotation == 1.5, "Transform2D.rotation mismatch")
            assert(t.scale_x == 2.0, "Transform2D.scale_x mismatch")
            assert(t.scale_y == 2.0, "Transform2D.scale_y mismatch")
            "#,
            "test_transform2d_fields",
        )
        .expect("Transform2D factory and field access should work");
    }

    #[test]
    fn test_lua_sprite_factory_and_fields() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local s = Sprite({ color_r = 1.0, color_g = 0.5, color_b = 0.0, color_a = 1.0 })
            assert(s.color_r == 1.0, "Sprite.color_r mismatch")
            assert(s.flip_x == false, "Sprite.flip_x should default to false")
            assert(s.flip_y == false, "Sprite.flip_y should default to false")
            "#,
            "test_sprite_fields",
        )
        .expect("Sprite factory and field access should work");
    }

    #[test]
    fn test_lua_text_factory_and_fields() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local txt = Text({ font_size = 24.0, color_r = 1.0, color_g = 1.0, color_b = 1.0, color_a = 1.0 })
            assert(txt.font_size == 24.0, "Text.font_size mismatch")
            assert(txt.color_r == 1.0, "Text.color_r mismatch")
            "#,
            "test_text_fields",
        )
        .expect("Text factory and field access should work");
    }

    #[test]
    fn test_lua_type_defaults_zeroed() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local v = Vec2({})
            assert(v.x == 0.0, "Vec2 default x should be 0")
            assert(v.y == 0.0, "Vec2 default y should be 0")
            local t = Transform2D({})
            assert(t.position_x == 0.0, "Transform2D default position_x should be 0")
            assert(t.rotation == 0.0, "Transform2D default rotation should be 0")
            "#,
            "test_type_defaults",
        )
        .expect("type defaults should be zeroed");
    }

    // =========================================================================
    // Enum constant tests
    // =========================================================================

    #[test]
    fn test_lua_enum_key_table_exists() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            assert(key ~= nil, "key table should exist")
            assert(key.space == 32, "key.space should be 32")
            assert(key.w == 87, "key.w should be 87")
            assert(key.escape == 256, "key.escape should be 256")
            assert(key.enter == 257, "key.enter should be 257")
            "#,
            "test_enum_key",
        )
        .expect("key enum constants should be accessible");
    }

    #[test]
    fn test_lua_enum_mouse_button_table_exists() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            assert(mouse_button ~= nil, "mouse_button table should exist")
            "#,
            "test_enum_mouse_button",
        )
        .expect("mouse_button enum table should exist");
    }

    #[test]
    fn test_lua_enum_blend_mode_table_exists() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            assert(blend_mode ~= nil, "blend_mode table should exist")
            "#,
            "test_enum_blend_mode",
        )
        .expect("blend_mode enum table should exist");
    }

    #[test]
    fn test_lua_enum_text_alignment_table_exists() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            assert(text_alignment ~= nil, "text_alignment table should exist")
            "#,
            "test_enum_text_alignment",
        )
        .expect("text_alignment enum table should exist");
    }

    #[test]
    fn test_lua_multiple_enum_tables() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            assert(key ~= nil, "key table should exist")
            assert(mouse_button ~= nil, "mouse_button table should exist")
            assert(overlay_corner ~= nil, "overlay_corner table should exist")
            assert(blend_mode ~= nil, "blend_mode table should exist")
            assert(text_alignment ~= nil, "text_alignment table should exist")
            assert(text_direction ~= nil, "text_direction table should exist")
            assert(easing_type ~= nil, "easing_type table should exist")
            "#,
            "test_multiple_enums",
        )
        .expect("all enum tables should be registered");
    }

    // =========================================================================
    // Script file execution tests
    // =========================================================================

    #[test]
    fn test_lua_type_tests_script_file() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        let script = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/lua_tests/type_tests.lua"
        ))
        .expect("type_tests.lua should exist");
        game.execute_lua(&script, "type_tests.lua")
            .expect("type_tests.lua should pass all assertions");
    }

    #[test]
    fn test_lua_enum_tests_script_file() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        let script = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/lua_tests/enum_tests.lua"
        ))
        .expect("enum_tests.lua should exist");
        game.execute_lua(&script, "enum_tests.lua")
            .expect("enum_tests.lua should pass all assertions");
    }

    // =========================================================================
    // Tools table registration (native feature only)
    // =========================================================================

    #[test]
    #[cfg(feature = "native")]
    fn test_lua_goud_game_table_registered() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            assert(goud_game ~= nil, "goud_game table should exist")
            "#,
            "test_goud_game_table",
        )
        .expect("goud_game table should be registered when native feature is enabled");
    }

    #[test]
    #[cfg(feature = "native")]
    fn test_lua_goud_context_table_registered() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            assert(goud_context ~= nil, "goud_context table should exist")
            "#,
            "test_goud_context_table",
        )
        .expect("goud_context table should be registered when native feature is enabled");
    }
}
