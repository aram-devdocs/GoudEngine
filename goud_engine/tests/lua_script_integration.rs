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
}
