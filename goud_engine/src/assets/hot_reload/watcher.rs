//! File system watcher for the hot reload system.

use crate::assets::AssetServer;
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use super::config::HotReloadConfig;
use super::events::AssetChangeEvent;

// =============================================================================
// HotReloadWatcher
// =============================================================================

/// Watches asset files for changes and triggers reloads.
///
/// # Thread Safety
///
/// `HotReloadWatcher` is `Send` but NOT `Sync`. It should be accessed from
/// the main thread and used with `AssetServer` in the game loop.
///
/// # Example
///
/// ```no_run
/// use goud_engine::assets::{AssetServer, HotReloadWatcher, HotReloadConfig};
/// use std::time::Duration;
///
/// let mut server = AssetServer::new();
/// let config = HotReloadConfig::new()
///     .with_debounce(Duration::from_millis(200))
///     .watch_extension("png")
///     .watch_extension("json");
///
/// let mut watcher = HotReloadWatcher::with_config(&server, config).unwrap();
///
/// // In game loop
/// loop {
///     watcher.process_events(&mut server);
///     // ... render, update, etc.
/// }
/// ```
pub struct HotReloadWatcher {
    /// File system watcher (kept alive to receive events).
    _watcher: RecommendedWatcher,

    /// Channel receiver for file system events.
    receiver: Receiver<NotifyResult<Event>>,

    /// Configuration.
    config: HotReloadConfig,

    /// Debounce tracking: path -> last event time.
    debounce_map: HashMap<PathBuf, Instant>,

    /// Paths currently being watched.
    watched_paths: HashSet<PathBuf>,

    /// Asset root directory (for relative path calculation).
    // Will be used in future when implementing actual reload logic
    _asset_root: PathBuf,
}

impl HotReloadWatcher {
    /// Creates a new hot reload watcher for the given asset server.
    ///
    /// Uses default configuration (enabled in debug builds).
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be initialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::{AssetServer, HotReloadWatcher};
    ///
    /// let server = AssetServer::new();
    /// let watcher = HotReloadWatcher::new(&server).unwrap();
    /// ```
    pub fn new(server: &AssetServer) -> NotifyResult<Self> {
        Self::with_config(server, HotReloadConfig::default())
    }

    /// Creates a new hot reload watcher with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be initialized.
    pub fn with_config(server: &AssetServer, config: HotReloadConfig) -> NotifyResult<Self> {
        let (sender, receiver) = channel();
        let asset_root = server.asset_root().to_path_buf();

        // Create watcher
        let watcher = Self::create_watcher(sender, &config)?;

        Ok(Self {
            _watcher: watcher,
            receiver,
            config,
            debounce_map: HashMap::new(),
            watched_paths: HashSet::new(),
            _asset_root: asset_root,
        })
    }

    /// Creates the underlying file system watcher.
    fn create_watcher(
        sender: Sender<NotifyResult<Event>>,
        _config: &HotReloadConfig,
    ) -> NotifyResult<RecommendedWatcher> {
        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = sender.send(res);
            },
            Config::default(),
        )?;

        Ok(watcher)
    }

    /// Starts watching a directory for changes.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory to watch (typically the asset root)
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be watched.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use goud_engine::assets::{AssetServer, HotReloadWatcher};
    /// # let server = AssetServer::new();
    /// # let mut watcher = HotReloadWatcher::new(&server).unwrap();
    /// watcher.watch("assets").unwrap();
    /// ```
    pub fn watch(&mut self, path: impl AsRef<Path>) -> NotifyResult<()> {
        let path = path.as_ref();
        let mode = if self.config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        self._watcher.watch(path, mode)?;
        self.watched_paths.insert(path.to_path_buf());

        Ok(())
    }

    /// Stops watching a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be unwatched.
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> NotifyResult<()> {
        let path = path.as_ref();
        self._watcher.unwatch(path)?;
        self.watched_paths.remove(path);

        Ok(())
    }

    /// Returns whether a path is currently being watched.
    pub fn is_watching(&self, path: &Path) -> bool {
        self.watched_paths.contains(path)
    }

    /// Returns the set of watched paths.
    pub fn watched_paths(&self) -> &HashSet<PathBuf> {
        &self.watched_paths
    }

    /// Returns the configuration.
    pub fn config(&self) -> &HotReloadConfig {
        &self.config
    }

    /// Returns a mutable reference to the configuration.
    ///
    /// Note: Changing the configuration does not affect already-watched paths.
    pub fn config_mut(&mut self) -> &mut HotReloadConfig {
        &mut self.config
    }

    /// Processes pending file system events and reloads changed assets.
    ///
    /// Call this once per frame in your game loop.
    ///
    /// # Returns
    ///
    /// The number of assets that were reloaded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use goud_engine::assets::{AssetServer, HotReloadWatcher};
    /// # let mut server = AssetServer::new();
    /// # let mut watcher = HotReloadWatcher::new(&server).unwrap();
    /// loop {
    ///     let reloaded = watcher.process_events(&mut server);
    ///     if reloaded > 0 {
    ///         println!("Reloaded {} assets", reloaded);
    ///     }
    ///     // ... rest of game loop
    /// }
    /// ```
    pub fn process_events(&mut self, _server: &mut AssetServer) -> usize {
        if !self.config.enabled {
            return 0;
        }

        let now = Instant::now();

        // Collect events (non-blocking)
        let mut change_events = Vec::new();
        while let Ok(event_result) = self.receiver.try_recv() {
            if let Ok(event) = event_result {
                if let Some(change_event) = self.process_file_event(&event, now) {
                    change_events.push(change_event);
                }
            }
        }

        // TODO: Actually reload assets via AssetServer
        // For now, just count the events
        let reload_count = change_events.len();

        // Clean up old debounce entries (keep last 1000)
        if self.debounce_map.len() > 1000 {
            self.debounce_map
                .retain(|_, time| now.duration_since(*time) < Duration::from_secs(10));
        }

        reload_count
    }

    /// Processes a single file system event.
    fn process_file_event(&mut self, event: &Event, now: Instant) -> Option<AssetChangeEvent> {
        // Filter event kind
        let change_event = match &event.kind {
            EventKind::Modify(_) => {
                let path = event.paths.first()?.clone();
                AssetChangeEvent::Modified { path }
            }
            EventKind::Create(_) => {
                let path = event.paths.first()?.clone();
                AssetChangeEvent::Created { path }
            }
            EventKind::Remove(_) => {
                let path = event.paths.first()?.clone();
                AssetChangeEvent::Deleted { path }
            }
            _ => return None, // Ignore other events
        };

        let path = change_event.path();

        // Check if we should watch this path
        if !self.config.should_watch(path) {
            return None;
        }

        // Debounce check
        if let Some(last_time) = self.debounce_map.get(path) {
            if now.duration_since(*last_time) < self.config.debounce_duration {
                return None; // Too soon, ignore
            }
        }

        // Update debounce map
        self.debounce_map.insert(path.to_path_buf(), now);

        Some(change_event)
    }

    /// Clears the debounce map.
    ///
    /// Useful for testing or forcing immediate reloads.
    pub fn clear_debounce(&mut self) {
        self.debounce_map.clear();
    }
}

impl fmt::Debug for HotReloadWatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HotReloadWatcher")
            .field("config", &self.config)
            .field("watched_paths", &self.watched_paths.len())
            .field("debounce_entries", &self.debounce_map.len())
            .finish()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_server() -> (AssetServer, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let server = AssetServer::with_root(temp_dir.path());
        (server, temp_dir)
    }

    #[test]
    fn test_new() {
        let (server, _temp_dir) = create_test_server();
        let watcher = HotReloadWatcher::new(&server);

        assert!(watcher.is_ok());
    }

    #[test]
    fn test_with_config() {
        let (server, _temp_dir) = create_test_server();
        let config = HotReloadConfig::new().with_enabled(false);
        let watcher = HotReloadWatcher::with_config(&server, config);

        assert!(watcher.is_ok());
        assert!(!watcher.unwrap().config().enabled);
    }

    #[test]
    fn test_watch() {
        let (server, temp_dir) = create_test_server();
        let mut watcher = HotReloadWatcher::new(&server).unwrap();

        let result = watcher.watch(temp_dir.path());
        assert!(result.is_ok());
        assert!(watcher.is_watching(temp_dir.path()));
        assert_eq!(watcher.watched_paths().len(), 1);
    }

    #[test]
    fn test_unwatch() {
        let (server, temp_dir) = create_test_server();
        let mut watcher = HotReloadWatcher::new(&server).unwrap();

        watcher.watch(temp_dir.path()).unwrap();
        assert!(watcher.is_watching(temp_dir.path()));

        watcher.unwatch(temp_dir.path()).unwrap();
        assert!(!watcher.is_watching(temp_dir.path()));
    }

    #[test]
    fn test_process_events_disabled() {
        let (mut server, temp_dir) = create_test_server();
        let config = HotReloadConfig::new().with_enabled(false);
        let mut watcher = HotReloadWatcher::with_config(&server, config).unwrap();

        watcher.watch(temp_dir.path()).unwrap();

        // Create a file
        let test_file = temp_dir.path().join("test.txt");
        fs::File::create(&test_file)
            .unwrap()
            .write_all(b"test")
            .unwrap();

        // Process events (should do nothing since disabled)
        std::thread::sleep(Duration::from_millis(50));
        let count = watcher.process_events(&mut server);

        assert_eq!(count, 0);
    }

    #[test]
    fn test_config_mut() {
        let (server, _temp_dir) = create_test_server();
        let mut watcher = HotReloadWatcher::new(&server).unwrap();

        watcher.config_mut().enabled = false;
        assert!(!watcher.config().enabled);
    }

    #[test]
    fn test_clear_debounce() {
        let (server, _temp_dir) = create_test_server();
        let mut watcher = HotReloadWatcher::new(&server).unwrap();

        watcher.clear_debounce();
        // Should not panic
    }

    #[test]
    fn test_debug() {
        let (server, _temp_dir) = create_test_server();
        let watcher = HotReloadWatcher::new(&server).unwrap();

        let debug = format!("{:?}", watcher);
        assert!(debug.contains("HotReloadWatcher"));
    }

    #[test]
    fn test_is_send() {
        fn requires_send<T: Send>() {}
        requires_send::<HotReloadWatcher>();
    }
}
