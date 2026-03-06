//! Configuration types for the hot reload system.

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
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
