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
}
