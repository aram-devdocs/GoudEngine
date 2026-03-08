//! Reference counting operations for `AssetServer`.

use super::core::AssetServer;
use crate::assets::{Asset, AssetHandle};

impl AssetServer {
    /// Increments the reference count for a handle.
    pub fn retain<A: Asset>(&mut self, handle: &AssetHandle<A>) -> Option<u32> {
        self.storage.retain(handle)
    }

    /// Decrements the reference count. Queues deferred unload when count hits 0.
    pub fn release<A: Asset>(&mut self, handle: &AssetHandle<A>) -> Option<u32> {
        let new_count = self.storage.release(handle)?;
        if new_count == 0 {
            self.pending_unloads.push((
                crate::assets::AssetId::of::<A>(),
                handle.index(),
                handle.generation(),
            ));
        }
        Some(new_count)
    }

    /// Returns the current reference count for a handle.
    pub fn ref_count<A: Asset>(&self, handle: &AssetHandle<A>) -> u32 {
        self.storage.ref_count(handle)
    }

    /// Drains the deferred-unload queue, removing assets still at ref count 0.
    ///
    /// Call once per frame at the end of the update loop.
    pub fn process_deferred_unloads(&mut self) {
        let pending = std::mem::take(&mut self.pending_unloads);
        for (asset_id, index, generation) in pending {
            // Check the asset is still alive
            if let Some(storage) = self.storage.get_any_storage(asset_id) {
                if !storage.is_alive_raw(index, generation) {
                    continue;
                }
            } else {
                continue;
            }

            // Check ref count is still 0 (retain may have been called)
            let count = self.storage.ref_count_raw(asset_id, index, generation);
            if count > 0 {
                continue;
            }

            // Clean up dependency graph via the erased path lookup
            if let Some(storage) = self.storage.get_any_storage(asset_id) {
                let untyped = crate::assets::UntypedAssetHandle::new(index, generation, asset_id);
                if let Some(path) = storage.get_path_untyped(&untyped) {
                    let path_str = path.as_str().to_string();
                    self.dependency_graph.remove_asset(&path_str);
                }
            }

            self.storage.remove_raw(asset_id, index, generation);
        }
    }
}
