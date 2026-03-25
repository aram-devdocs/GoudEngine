use super::core::AssetServer;
#[cfg(feature = "desktop-native")]
use crate::assets::{HotReloadConfig, HotReloadWatcher};

impl AssetServer {
    /// Creates a hot reload watcher for this asset server.
    ///
    /// The watcher will detect file changes in the asset root directory
    /// and automatically reload modified assets.
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be initialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::AssetServer;
    ///
    /// let mut server = AssetServer::new();
    /// let mut watcher = server.create_hot_reload_watcher().unwrap();
    ///
    /// // In game loop
    /// loop {
    ///     watcher.process_events(&mut server);
    ///     // ... rest of game loop
    /// }
    /// ```
    #[cfg(feature = "desktop-native")]
    pub fn create_hot_reload_watcher(&self) -> notify::Result<HotReloadWatcher> {
        HotReloadWatcher::new(self)
    }

    /// Creates a hot reload watcher with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be initialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::{AssetServer, HotReloadConfig};
    /// use std::time::Duration;
    ///
    /// let mut server = AssetServer::new();
    /// let config = HotReloadConfig::new()
    ///     .with_debounce(Duration::from_millis(200))
    ///     .watch_extension("png")
    ///     .watch_extension("json");
    ///
    /// let mut watcher = server.create_hot_reload_watcher_with_config(config).unwrap();
    ///
    /// // In game loop
    /// loop {
    ///     watcher.process_events(&mut server);
    ///     // ... rest of game loop
    /// }
    /// ```
    #[cfg(feature = "desktop-native")]
    pub fn create_hot_reload_watcher_with_config(
        &self,
        config: HotReloadConfig,
    ) -> notify::Result<HotReloadWatcher> {
        HotReloadWatcher::with_config(self, config)
    }
}
