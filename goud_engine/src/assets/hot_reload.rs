//! Hot reloading system for detecting and reloading changed assets.
//!
//! # Overview
//!
//! The hot reload system watches asset files for changes and automatically
//! reloads them without restarting the application. This is essential for
//! rapid iteration during development.
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                     HotReloadWatcher                        │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
//! │  │ File System  │  │   Channel    │  │    Event     │    │
//! │  │   Watcher    │──│   Receiver   │──│   Processor  │    │
//! │  └──────────────┘  └──────────────┘  └──────────────┘    │
//! └────────────────────────────────────────────────────────────┘
//!         │                                          │
//!         ▼                                          ▼
//!    File Change                              Asset Reload
//!     Detected                                  Triggered
//! ```
//!
//! # Example
//!
//! ```no_run
//! use goud_engine::assets::{AssetServer, HotReloadWatcher};
//!
//! let mut server = AssetServer::new();
//! let mut watcher = HotReloadWatcher::new(&server).unwrap();
//!
//! // In your game loop
//! loop {
//!     watcher.process_events(&mut server);
//!     // ... rest of game loop
//! }
//! ```

use crate::assets::AssetServer;
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

// =============================================================================
// AssetChangeEvent
// =============================================================================

/// Event representing a change to an asset file.
///
/// Emitted by the hot reload system when a file is modified, created, or deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetChangeEvent {
    /// Asset file was modified.
    Modified {
        /// The path to the modified asset.
        path: PathBuf,
    },

    /// Asset file was created.
    Created {
        /// The path to the newly created asset.
        path: PathBuf,
    },

    /// Asset file was deleted.
    Deleted {
        /// The path to the deleted asset.
        path: PathBuf,
    },

    /// Asset file was renamed.
    Renamed {
        /// The old path of the asset.
        from: PathBuf,
        /// The new path of the asset.
        to: PathBuf,
    },
}

impl AssetChangeEvent {
    /// Returns the primary path affected by this event.
    pub fn path(&self) -> &Path {
        match self {
            Self::Modified { path } | Self::Created { path } | Self::Deleted { path } => path,
            Self::Renamed { to, .. } => to,
        }
    }

    /// Returns the event kind as a string.
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Modified { .. } => "Modified",
            Self::Created { .. } => "Created",
            Self::Deleted { .. } => "Deleted",
            Self::Renamed { .. } => "Renamed",
        }
    }

    /// Returns true if this is a modification event.
    pub fn is_modified(&self) -> bool {
        matches!(self, Self::Modified { .. })
    }

    /// Returns true if this is a creation event.
    pub fn is_created(&self) -> bool {
        matches!(self, Self::Created { .. })
    }

    /// Returns true if this is a deletion event.
    pub fn is_deleted(&self) -> bool {
        matches!(self, Self::Deleted { .. })
    }

    /// Returns true if this is a rename event.
    pub fn is_renamed(&self) -> bool {
        matches!(self, Self::Renamed { .. })
    }
}

impl fmt::Display for AssetChangeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Modified { path } => write!(f, "Modified: {}", path.display()),
            Self::Created { path } => write!(f, "Created: {}", path.display()),
            Self::Deleted { path } => write!(f, "Deleted: {}", path.display()),
            Self::Renamed { from, to } => {
                write!(f, "Renamed: {} -> {}", from.display(), to.display())
            }
        }
    }
}

// =============================================================================
// HotReloadConfig
// =============================================================================

/// Configuration for hot reloading behavior.
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Whether hot reloading is enabled (default: true in debug, false in release).
    pub enabled: bool,

    /// Debounce delay to avoid duplicate reloads (default: 100ms).
    ///
    /// File systems often emit multiple events for a single change.
    /// This delay groups rapid changes together.
    pub debounce_duration: Duration,

    /// Whether to watch subdirectories recursively (default: true).
    pub recursive: bool,

    /// File extensions to watch (empty = watch all files).
    pub extensions: HashSet<String>,

    /// Whether to ignore hidden files (starting with '.').
    pub ignore_hidden: bool,

    /// Whether to ignore temporary files (ending with '~', '.tmp', '.swp', etc.).
    pub ignore_temp: bool,
}

impl HotReloadConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self {
            enabled: cfg!(debug_assertions), // Enabled in debug builds by default
            debounce_duration: Duration::from_millis(100),
            recursive: true,
            extensions: HashSet::new(), // Empty = watch all
            ignore_hidden: true,
            ignore_temp: true,
        }
    }

    /// Sets whether hot reloading is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the debounce duration.
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }

    /// Sets whether to watch recursively.
    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Adds a file extension to watch (e.g., "png", "json").
    pub fn watch_extension(mut self, ext: impl Into<String>) -> Self {
        self.extensions.insert(ext.into());
        self
    }

    /// Sets whether to ignore hidden files.
    pub fn with_ignore_hidden(mut self, ignore: bool) -> Self {
        self.ignore_hidden = ignore;
        self
    }

    /// Sets whether to ignore temporary files.
    pub fn with_ignore_temp(mut self, ignore: bool) -> Self {
        self.ignore_temp = ignore;
        self
    }

    /// Returns true if a path should be watched based on configuration.
    pub fn should_watch(&self, path: &Path) -> bool {
        // Check hidden files
        if self.ignore_hidden {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    return false;
                }
            }
        }

        // Check temporary files
        if self.ignore_temp {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with('~')
                    || name.ends_with(".tmp")
                    || name.ends_with(".swp")
                    || name.ends_with(".bak")
                {
                    return false;
                }
            }
        }

        // Check extension filter
        if !self.extensions.is_empty() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !self.extensions.contains(ext) {
                    return false;
                }
            } else {
                // No extension and we have filters = ignore
                return false;
            }
        }

        true
    }
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self::new()
    }
}

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
    #[allow(dead_code)]
    watcher: RecommendedWatcher,

    /// Channel receiver for file system events.
    receiver: Receiver<NotifyResult<Event>>,

    /// Configuration.
    config: HotReloadConfig,

    /// Debounce tracking: path -> last event time.
    debounce_map: HashMap<PathBuf, Instant>,

    /// Paths currently being watched.
    watched_paths: HashSet<PathBuf>,

    /// Asset root directory (for relative path calculation).
    #[allow(dead_code)] // Will be used in future when implementing actual reload logic
    asset_root: PathBuf,
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
            watcher,
            receiver,
            config,
            debounce_map: HashMap::new(),
            watched_paths: HashSet::new(),
            asset_root,
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

        self.watcher.watch(path, mode)?;
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
        self.watcher.unwatch(path)?;
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

    // =============================================================================
    // AssetChangeEvent Tests
    // =============================================================================

    mod asset_change_event {
        use super::*;

        #[test]
        fn test_modified() {
            let event = AssetChangeEvent::Modified {
                path: PathBuf::from("test.png"),
            };

            assert_eq!(event.path(), Path::new("test.png"));
            assert_eq!(event.kind_str(), "Modified");
            assert!(event.is_modified());
            assert!(!event.is_created());
            assert!(!event.is_deleted());
            assert!(!event.is_renamed());
        }

        #[test]
        fn test_created() {
            let event = AssetChangeEvent::Created {
                path: PathBuf::from("new.png"),
            };

            assert_eq!(event.path(), Path::new("new.png"));
            assert_eq!(event.kind_str(), "Created");
            assert!(!event.is_modified());
            assert!(event.is_created());
        }

        #[test]
        fn test_deleted() {
            let event = AssetChangeEvent::Deleted {
                path: PathBuf::from("old.png"),
            };

            assert_eq!(event.path(), Path::new("old.png"));
            assert_eq!(event.kind_str(), "Deleted");
            assert!(event.is_deleted());
        }

        #[test]
        fn test_renamed() {
            let event = AssetChangeEvent::Renamed {
                from: PathBuf::from("old.png"),
                to: PathBuf::from("new.png"),
            };

            assert_eq!(event.path(), Path::new("new.png")); // Returns "to" path
            assert_eq!(event.kind_str(), "Renamed");
            assert!(event.is_renamed());
        }

        #[test]
        fn test_display() {
            let event = AssetChangeEvent::Modified {
                path: PathBuf::from("test.png"),
            };
            let display = format!("{}", event);
            assert!(display.contains("Modified"));
            assert!(display.contains("test.png"));
        }

        #[test]
        fn test_clone() {
            let event = AssetChangeEvent::Modified {
                path: PathBuf::from("test.png"),
            };
            let cloned = event.clone();
            assert_eq!(event, cloned);
        }
    }

    // =============================================================================
    // HotReloadConfig Tests
    // =============================================================================

    mod hot_reload_config {
        use super::*;

        #[test]
        fn test_default() {
            let config = HotReloadConfig::default();
            assert_eq!(config.enabled, cfg!(debug_assertions));
            assert_eq!(config.debounce_duration, Duration::from_millis(100));
            assert!(config.recursive);
            assert!(config.ignore_hidden);
            assert!(config.ignore_temp);
            assert!(config.extensions.is_empty());
        }

        #[test]
        fn test_with_enabled() {
            let config = HotReloadConfig::new().with_enabled(false);
            assert!(!config.enabled);
        }

        #[test]
        fn test_with_debounce() {
            let config = HotReloadConfig::new().with_debounce(Duration::from_millis(500));
            assert_eq!(config.debounce_duration, Duration::from_millis(500));
        }

        #[test]
        fn test_with_recursive() {
            let config = HotReloadConfig::new().with_recursive(false);
            assert!(!config.recursive);
        }

        #[test]
        fn test_watch_extension() {
            let config = HotReloadConfig::new()
                .watch_extension("png")
                .watch_extension("json");

            assert!(config.extensions.contains("png"));
            assert!(config.extensions.contains("json"));
            assert_eq!(config.extensions.len(), 2);
        }

        #[test]
        fn test_should_watch_all() {
            let config = HotReloadConfig::new();

            // No extension filter = watch all
            assert!(config.should_watch(Path::new("test.png")));
            assert!(config.should_watch(Path::new("test.json")));
        }

        #[test]
        fn test_should_watch_with_extension_filter() {
            let config = HotReloadConfig::new().watch_extension("png");

            assert!(config.should_watch(Path::new("test.png")));
            assert!(!config.should_watch(Path::new("test.json")));
            assert!(!config.should_watch(Path::new("no_extension")));
        }

        #[test]
        fn test_should_watch_hidden_files() {
            let config = HotReloadConfig::new().with_ignore_hidden(true);

            assert!(config.should_watch(Path::new("test.png")));
            assert!(!config.should_watch(Path::new(".hidden.png")));
        }

        #[test]
        fn test_should_watch_temp_files() {
            let config = HotReloadConfig::new().with_ignore_temp(true);

            assert!(config.should_watch(Path::new("test.png")));
            assert!(!config.should_watch(Path::new("test.png~")));
            assert!(!config.should_watch(Path::new("test.tmp")));
            assert!(!config.should_watch(Path::new("test.swp")));
            assert!(!config.should_watch(Path::new("test.bak")));
        }

        #[test]
        fn test_should_watch_allow_hidden() {
            let config = HotReloadConfig::new().with_ignore_hidden(false);

            assert!(config.should_watch(Path::new(".hidden.png")));
        }

        #[test]
        fn test_clone() {
            let config = HotReloadConfig::new().watch_extension("png");
            let cloned = config.clone();

            assert_eq!(config.enabled, cloned.enabled);
            assert_eq!(config.extensions.len(), cloned.extensions.len());
        }

        #[test]
        fn test_debug() {
            let config = HotReloadConfig::new();
            let debug = format!("{:?}", config);
            assert!(debug.contains("HotReloadConfig"));
        }
    }

    // =============================================================================
    // HotReloadWatcher Tests
    // =============================================================================

    mod hot_reload_watcher {
        use super::*;

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
}
