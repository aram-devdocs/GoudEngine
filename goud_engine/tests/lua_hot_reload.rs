//! Integration tests for Lua hot-reload watcher.
//!
//! Required features: lua, native

#[cfg(all(feature = "lua", feature = "native"))]
mod lua_hot_reload_tests {
    use goud_engine::sdk::game_config::GameConfig;
    use goud_engine::sdk::GoudGame;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_watch_lua_dir_creates_successfully() {
        let tmp = TempDir::new().expect("should create temp dir");
        let mut game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        let result = game.watch_lua_dir(tmp.path());
        assert!(result.is_ok(), "watch_lua_dir should succeed: {result:?}");
    }

    #[test]
    fn test_watch_lua_dir_nonexistent_fails() {
        let mut game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        let result = game.watch_lua_dir("/tmp/nonexistent_lua_dir_that_should_not_exist_12345");
        assert!(
            result.is_err(),
            "watch_lua_dir on nonexistent path should fail"
        );
    }

    #[test]
    fn test_file_change_detection() {
        let tmp = TempDir::new().expect("should create temp dir");
        let script_path = tmp.path().join("test_script.lua");

        // Write initial file before starting watcher.
        fs::write(&script_path, "-- initial").expect("should write initial file");

        let mut game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.watch_lua_dir(tmp.path())
            .expect("watch_lua_dir should succeed");

        // Drain any initial creation events.
        std::thread::sleep(std::time::Duration::from_millis(300));
        game.process_lua_hot_reload();

        // Modify the file.
        {
            let mut f = fs::File::create(&script_path).expect("should open file for writing");
            f.write_all(b"local x = 42").expect("should write");
            f.flush().expect("should flush");
        }

        // Wait for the file system event to propagate.
        std::thread::sleep(std::time::Duration::from_millis(500));

        // process_lua_hot_reload should not panic even with valid Lua.
        game.process_lua_hot_reload();
    }

    #[test]
    fn test_reload_valid_script_no_crash() {
        let tmp = TempDir::new().expect("should create temp dir");
        let script_path = tmp.path().join("valid.lua");
        fs::write(&script_path, "local x = 1 + 2").expect("should write file");

        let mut game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.watch_lua_dir(tmp.path())
            .expect("watch_lua_dir should succeed");

        // Wait for initial events.
        std::thread::sleep(std::time::Duration::from_millis(300));
        game.process_lua_hot_reload();

        // Modify with valid Lua.
        fs::write(&script_path, "local y = 3 + 4").expect("should write updated file");
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Should not panic.
        game.process_lua_hot_reload();
    }

    #[test]
    fn test_reload_syntax_error_does_not_crash() {
        let tmp = TempDir::new().expect("should create temp dir");
        let script_path = tmp.path().join("broken.lua");
        fs::write(&script_path, "local x = 1").expect("should write file");

        let mut game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.watch_lua_dir(tmp.path())
            .expect("watch_lua_dir should succeed");

        // Wait for initial events.
        std::thread::sleep(std::time::Duration::from_millis(300));
        game.process_lua_hot_reload();

        // Write a file with syntax errors.
        fs::write(&script_path, "if then end end").expect("should write broken file");
        std::thread::sleep(std::time::Duration::from_millis(500));

        // process_lua_hot_reload logs the error but should not panic.
        game.process_lua_hot_reload();
    }

    #[test]
    fn test_debounce_rapid_changes() {
        let tmp = TempDir::new().expect("should create temp dir");
        let script_path = tmp.path().join("debounce.lua");
        fs::write(&script_path, "-- v1").expect("should write file");

        let mut game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.watch_lua_dir(tmp.path())
            .expect("watch_lua_dir should succeed");

        // Drain initial events.
        std::thread::sleep(std::time::Duration::from_millis(300));
        game.process_lua_hot_reload();

        // Rapid successive writes (within the 200ms debounce window).
        for i in 0..5 {
            fs::write(&script_path, format!("-- v{}", i + 2)).expect("should write");
        }

        // Wait for events to propagate.
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Should not panic regardless of how many events came in.
        game.process_lua_hot_reload();
    }

    #[test]
    fn test_non_lua_files_ignored() {
        let tmp = TempDir::new().expect("should create temp dir");
        let txt_path = tmp.path().join("readme.txt");
        fs::write(&txt_path, "hello").expect("should write txt file");

        let mut game = GoudGame::new(GameConfig::default()).expect("headless game should init");
        game.watch_lua_dir(tmp.path())
            .expect("watch_lua_dir should succeed");

        std::thread::sleep(std::time::Duration::from_millis(300));

        // Modify non-lua file.
        fs::write(&txt_path, "world").expect("should write txt update");
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Should not panic or process any non-lua files.
        game.process_lua_hot_reload();
    }
}
