//! Web-specific asset loading operations (WASM/browser).
//!
//! This module is only compiled when the `web` feature is enabled.

// All contents are gated behind `#[cfg(feature = "web")]` to avoid
// unused-import warnings on native builds.

#[cfg(feature = "web")]
use super::core::AssetServer;
#[cfg(feature = "web")]
use crate::assets::{Asset, AssetHandle, AssetPath};
#[cfg(feature = "web")]
use std::path::Path;

#[cfg(feature = "web")]
impl AssetServer {
    /// Loads an asset asynchronously via the browser Fetch API.
    ///
    /// Constructs a URL from the asset root and path, fetches the bytes over
    /// HTTP, then runs them through the registered loader. Only available when
    /// the `web` feature is enabled (wasm32 targets).
    ///
    /// Returns an existing handle if an asset with the same path is already loaded.
    ///
    /// # Arguments
    ///
    /// * `path` - Asset path relative to the asset root (e.g., "textures/player.png")
    pub async fn load_async<A: Asset>(&mut self, path: impl AsRef<Path>) -> AssetHandle<A> {
        let asset_path = AssetPath::new(path.as_ref().to_str().unwrap_or_default());

        if let Some(handle) = self.storage.get_handle_by_path::<A>(asset_path.as_str()) {
            return handle;
        }

        let handle = self.storage.reserve_with_path::<A>(asset_path.clone());

        let url = format!("{}/{}", self.asset_root.display(), asset_path.as_str());

        match crate::assets::web_fetch::web_fetch(&url).await {
            Ok(bytes) => match self.parse_bytes::<A>(&asset_path, &bytes) {
                Ok((asset, dependencies, embedded_assets)) => {
                    self.storage.set_loaded(&handle, asset);
                    self.load_embedded_assets(&embedded_assets);
                    // Record dependencies in the graph
                    let asset_str = asset_path.as_str().to_string();
                    for dep in &dependencies {
                        let _ = self.dependency_graph.add_dependency(&asset_str, dep);
                    }
                }
                Err(error) => {
                    if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
                        entry.set_failed(error.to_string());
                    }
                }
            },
            Err(error) => {
                if let Some(entry) = self.storage.get_entry_mut::<A>(&handle) {
                    entry.set_failed(error.to_string());
                }
            }
        }

        handle
    }
}
