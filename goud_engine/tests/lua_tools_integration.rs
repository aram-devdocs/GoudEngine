//! Integration tests for Lua tool bindings (collision, entity, context).
//!
//! Required features: lua, native

#[cfg(all(feature = "lua", feature = "native"))]
mod lua_tools_tests {
    use goud_engine::sdk::game_config::GameConfig;
    use goud_engine::sdk::GoudGame;

    // =========================================================================
    // Collision helpers (pure math, no GL context needed)
    // =========================================================================

    #[test]
    fn test_lua_collision_aabb_overlap_true() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local result = goud_game.aabb_overlap(0, 0, 10, 10, 5, 5, 15, 15)
            assert(result == true, "overlapping AABBs should return true")
            "#,
            "test_aabb_overlap_true",
        )
        .expect("aabb_overlap with overlap should succeed");
    }

    #[test]
    fn test_lua_collision_aabb_overlap_false() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local result = goud_game.aabb_overlap(0, 0, 10, 10, 20, 20, 30, 30)
            assert(result == false, "separated AABBs should return false")
            "#,
            "test_aabb_overlap_false",
        )
        .expect("aabb_overlap without overlap should succeed");
    }

    #[test]
    fn test_lua_collision_circle_overlap_true() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local result = goud_game.circle_overlap(0, 0, 5, 3, 0, 5)
            assert(result == true, "overlapping circles should return true")
            "#,
            "test_circle_overlap_true",
        )
        .expect("circle_overlap with overlap should succeed");
    }

    #[test]
    fn test_lua_collision_circle_overlap_false() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local result = goud_game.circle_overlap(0, 0, 1, 100, 100, 1)
            assert(result == false, "separated circles should return false")
            "#,
            "test_circle_overlap_false",
        )
        .expect("circle_overlap without overlap should succeed");
    }

    #[test]
    fn test_lua_collision_distance() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local d = goud_game.distance(0, 0, 3, 4)
            assert(d > 4.99 and d < 5.01, "distance(0,0,3,4) should be ~5, got: " .. tostring(d))
            "#,
            "test_distance",
        )
        .expect("distance should return correct value");
    }

    #[test]
    fn test_lua_collision_distance_squared() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local d2 = goud_game.distance_squared(0, 0, 3, 4)
            assert(d2 > 24.9 and d2 < 25.1, "distance_squared should be ~25, got: " .. tostring(d2))
            "#,
            "test_distance_squared",
        )
        .expect("distance_squared should return correct value");
    }

    #[test]
    fn test_lua_collision_point_in_rect_true() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local result = goud_game.point_in_rect(5, 5, 0, 0, 10, 10)
            assert(result == true, "point (5,5) should be inside rect (0,0,10,10)")
            "#,
            "test_point_in_rect_true",
        )
        .expect("point_in_rect inside should succeed");
    }

    #[test]
    fn test_lua_collision_point_in_rect_false() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local result = goud_game.point_in_rect(15, 15, 0, 0, 10, 10)
            assert(result == false, "point (15,15) should be outside rect (0,0,10,10)")
            "#,
            "test_point_in_rect_false",
        )
        .expect("point_in_rect outside should succeed");
    }

    #[test]
    fn test_lua_collision_point_in_circle_true() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local result = goud_game.point_in_circle(1, 1, 0, 0, 5)
            assert(result == true, "point (1,1) should be inside circle at (0,0) r=5")
            "#,
            "test_point_in_circle_true",
        )
        .expect("point_in_circle inside should succeed");
    }

    #[test]
    fn test_lua_collision_point_in_circle_false() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local result = goud_game.point_in_circle(10, 10, 0, 0, 5)
            assert(result == false, "point (10,10) should be outside circle at (0,0) r=5")
            "#,
            "test_point_in_circle_false",
        )
        .expect("point_in_circle outside should succeed");
    }

    // =========================================================================
    // Entity spawn / despawn (require a registered context -- GL context needed)
    // =========================================================================

    #[test]
    #[ignore] // Requires a full native context with GL backend.
    fn test_lua_entity_spawn_empty() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local id = goud_game.spawn_empty()
            assert(type(id) == "number", "spawn_empty should return a number")
            assert(id > 0, "entity id should be positive")
            "#,
            "test_spawn_empty",
        )
        .expect("spawn_empty should succeed");
    }

    #[test]
    #[ignore] // Requires a full native context with GL backend.
    fn test_lua_entity_despawn() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local id = goud_game.spawn_empty()
            assert(goud_game.is_alive(id) == true, "entity should be alive after spawn")
            local result = goud_game.despawn(id)
            assert(result == 0, "despawn should return 0 (success)")
            "#,
            "test_despawn",
        )
        .expect("despawn should succeed");
    }

    #[test]
    #[ignore] // Requires a full native context with GL backend.
    fn test_lua_entity_count() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local before = goud_game.entity_count()
            goud_game.spawn_empty()
            goud_game.spawn_empty()
            local after = goud_game.entity_count()
            assert(after == before + 2, "entity_count should increase by 2, before=" .. tostring(before) .. " after=" .. tostring(after))
            "#,
            "test_entity_count",
        )
        .expect("entity_count should reflect spawned entities");
    }

    // =========================================================================
    // Context operations (require a registered context -- GL context needed)
    // =========================================================================

    #[test]
    #[ignore] // Requires a full native context with GL backend.
    fn test_lua_context_is_valid() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            local valid = goud_context.is_valid()
            assert(valid == true, "context should be valid")
            "#,
            "test_context_is_valid",
        )
        .expect("context is_valid should succeed");
    }

    // =========================================================================
    // Tool table registration verification (headless safe)
    // =========================================================================

    #[test]
    fn test_lua_goud_game_table_has_collision_functions() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            assert(goud_game ~= nil, "goud_game table should exist")
            assert(type(goud_game.aabb_overlap) == "function", "aabb_overlap should be a function")
            assert(type(goud_game.circle_overlap) == "function", "circle_overlap should be a function")
            assert(type(goud_game.distance) == "function", "distance should be a function")
            assert(type(goud_game.distance_squared) == "function", "distance_squared should be a function")
            assert(type(goud_game.point_in_rect) == "function", "point_in_rect should be a function")
            assert(type(goud_game.point_in_circle) == "function", "point_in_circle should be a function")
            "#,
            "test_collision_functions_exist",
        )
        .expect("collision functions should be registered on goud_game table");
    }

    #[test]
    fn test_lua_goud_game_table_has_entity_functions() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.execute_lua(
            r#"
            assert(goud_game ~= nil, "goud_game table should exist")
            assert(type(goud_game.spawn_empty) == "function", "spawn_empty should be a function")
            assert(type(goud_game.despawn) == "function", "despawn should be a function")
            assert(type(goud_game.entity_count) == "function", "entity_count should be a function")
            assert(type(goud_game.is_alive) == "function", "is_alive should be a function")
            "#,
            "test_entity_functions_exist",
        )
        .expect("entity functions should be registered on goud_game table");
    }

    // =========================================================================
    // Collision script file execution
    // =========================================================================

    #[test]
    fn test_lua_collision_script_file() {
        let game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        let script = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/lua_tests/collision_tests.lua"
        ))
        .expect("collision_tests.lua should exist");
        game.execute_lua(&script, "collision_tests.lua")
            .expect("collision_tests.lua should pass all assertions");
    }
}
