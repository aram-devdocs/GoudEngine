//! Non-blocking asset loading operations (native-only).
//!
//! Uses a rayon thread pool for background file I/O and parsing, with results
//! communicated back to the main thread via `std::sync::mpsc` channels.

#[cfg(all(feature = "native", not(feature = "web")))]
use super::core::AssetServer;
#[cfg(all(feature = "native", not(feature = "web")))]
use super::core::LoadResult;
#[cfg(all(feature = "native", not(feature = "web")))]
use crate::assets::{Asset, AssetHandle, AssetId, AssetLoadError, AssetPath, LoadContext};
#[cfg(all(feature = "native", not(feature = "web")))]
use std::path::Path;

#[cfg(all(feature = "native", not(feature = "web")))]
impl AssetServer {
    /// Loads an asset asynchronously using a background thread.
    ///
    /// Returns a handle immediately in the `Loading` state. The actual file I/O
    /// and parsing happen on a rayon thread pool. Call [`process_loads`] each
    /// frame to drain completed results and transition handles to `Loaded` or
    /// `Failed`.
    ///
    /// # Deduplication
    ///
    /// If an asset with the same path is already loaded or loading, the existing
    /// handle is returned without spawning a new background task.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the asset file (relative to asset root)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let handle = server.load_async::<MyAsset>("data/config.test");
    /// // ... later, in the game loop:
    /// server.process_loads();
    /// if server.is_loaded(&handle) {
    ///     let asset = server.get(&handle).unwrap();
    /// }
    /// ```
    pub fn load_async<A: Asset>(&mut self, path: impl AsRef<Path>) -> AssetHandle<A> {
        let asset_path = AssetPath::new(path.as_ref().to_str().unwrap_or_default());

        // Deduplication: return existing handle if path already loaded/loading
        if let Some(handle) = self.storage.get_handle_by_path::<A>(asset_path.as_str()) {
            return handle;
        }

        // Reserve handle, set to Loading
        let handle = self.storage.reserve_with_path::<A>(asset_path.clone());
        if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
            entry.set_progress(0.0);
        }

        // Look up loader by extension
        let extension = match asset_path.extension() {
            Some(ext) => ext.to_string(),
            None => {
                if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
                    entry.set_failed("No file extension");
                }
                return handle;
            }
        };
        let loader = match self.loaders.get(&extension) {
            Some(l) => l.clone_boxed(),
            None => {
                if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
                    entry.set_failed(format!("No loader for extension: {}", extension));
                }
                return handle;
            }
        };

        // Capture values for background thread
        let asset_root = self.asset_root.clone();
        let sender = self.load_sender.clone();
        let handle_index = handle.index();
        let handle_generation = handle.generation();
        let asset_id = AssetId::of::<A>();
        let path_str = asset_path.as_str().to_string();

        rayon::spawn(move || {
            let full_path = asset_root.join(&path_str);
            let result = std::fs::read(&full_path)
                .map_err(|e| AssetLoadError::io_error(&full_path, e))
                .and_then(|bytes| {
                    let owned_path = AssetPath::from_string(path_str);
                    let mut context = LoadContext::new(owned_path);
                    loader.load_erased(&bytes, &mut context)
                });
            let _ = sender.send(LoadResult {
                handle_index,
                handle_generation,
                asset_id,
                result,
            });
        });

        handle
    }
}

#[cfg(feature = "native")]
impl super::core::AssetServer {
    /// Drains completed async load results and applies them to asset storage.
    ///
    /// This must be called from the main thread (typically once per frame) to
    /// transition async-loaded assets from `Loading` to `Loaded` or `Failed`.
    ///
    /// # Returns
    ///
    /// The number of load results processed in this call.
    pub fn process_loads(&mut self) -> usize {
        let mut count = 0;
        while let Ok(load_result) = self.load_receiver.try_recv() {
            match load_result.result {
                Ok(boxed) => {
                    self.storage.set_loaded_raw(
                        load_result.asset_id,
                        load_result.handle_index,
                        load_result.handle_generation,
                        boxed,
                    );
                }
                Err(error) => {
                    self.storage.set_failed_raw(
                        load_result.asset_id,
                        load_result.handle_index,
                        load_result.handle_generation,
                        error.to_string(),
                    );
                }
            }
            count += 1;
        }
        count
    }
}
