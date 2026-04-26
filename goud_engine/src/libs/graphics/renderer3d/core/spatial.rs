//! Spatial-index helpers for [`Renderer3D`].
//!
//! These methods keep the renderer's `spatial_index` in sync with the
//! `objects` map and translate per-object [`super::super::types::Object3D`]
//! state into the world-space AABB the index expects. They are deliberately
//! tiny so the call sites in `core_primitives.rs`, `core_models/mod.rs`,
//! `core_model_instances.rs`, and `object_transforms.rs` only have to add a
//! one-line hook after each insert/update/remove.

use super::super::spatial_index::world_aabb_from_sphere;
use super::Renderer3D;

impl Renderer3D {
    /// Refresh `id` in the spatial index by reading the current state of the
    /// object out of `self.objects`. No-op when the ID is not registered.
    pub(in crate::libs::graphics::renderer3d) fn spatial_index_refresh(&mut self, id: u32) {
        let Some(obj) = self.objects.get(&id) else {
            return;
        };
        let (min, max) = world_aabb_from_sphere(
            obj.position,
            obj.bounds.center,
            obj.bounds.radius,
            obj.scale,
        );
        self.spatial_index.insert(id, min, max);
    }

    /// Drop `id` from the spatial index. Returns `true` when the entry
    /// existed.
    pub(in crate::libs::graphics::renderer3d) fn spatial_index_remove(&mut self, id: u32) -> bool {
        self.spatial_index.remove(id)
    }
}
