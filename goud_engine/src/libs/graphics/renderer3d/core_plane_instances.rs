//! Instanced primitive (plane) support for [`Renderer3D`].
//!
//! `instantiate_plane(source_plane_id)` mirrors `instantiate_model`: it returns
//! a fresh per-instance handle whose transform can be updated independently,
//! while every instance that shares the same source plane is drawn through a
//! single instanced draw call. This collapses thousands of per-tile
//! `CreatePlane` primitives into one batch per source plane (issue #679).

use super::core::Renderer3D;
use super::mesh::{update_instance_buffer, upload_buffer, upload_instance_buffer};
use super::types::{InstanceTransform, InstancedMesh};
use cgmath::Vector3;
use std::mem;

/// Bookkeeping for the pool of plane instances that share one source primitive.
///
/// Each pool corresponds to exactly one [`InstancedMesh`] entry; the slot index
/// of an instance is its position in `instance_ids` and in
/// `InstancedMesh::instances`. Both vectors are kept aligned by every operation
/// in this module, so a swap-remove updates both at once.
#[derive(Debug)]
pub(in crate::libs::graphics::renderer3d) struct PlaneInstancePool {
    /// Source plane object id this pool was seeded from.
    pub(in crate::libs::graphics::renderer3d) source_plane_id: u32,
    /// Instance handles parallel to `InstancedMesh::instances`.
    pub(in crate::libs::graphics::renderer3d) instance_ids: Vec<u32>,
    /// Capacity (in slots) of the GPU instance buffer; recreated when grown.
    pub(in crate::libs::graphics::renderer3d) buffer_capacity_slots: usize,
    /// Set when CPU-side instances are out of sync with the GPU buffer.
    pub(in crate::libs::graphics::renderer3d) dirty: bool,
}

impl Renderer3D {
    /// Create an instance of a source plane primitive.
    ///
    /// Mirrors [`Renderer3D::instantiate_model`] for primitives. All instances
    /// of the same source plane render through one instanced draw call. The
    /// returned id can be passed to `set_object_position` / `set_object_rotation`
    /// / `set_object_scale` / `remove_object` (per-instance transform updates),
    /// and to `add_object_to_scene` (scene membership uses the source plane).
    ///
    /// Per-instance materials are not supported: the source plane's
    /// material/texture is captured at the time the pool is first created.
    /// Use one source plane per material to draw multiple materials.
    ///
    /// Destroying the source plane via [`Renderer3D::remove_object`] cascades
    /// to the pool: every instance handle is invalidated and the pool's GPU
    /// buffers are freed.
    ///
    /// Returns `None` when the source plane does not exist.
    pub fn instantiate_plane(&mut self, source_plane_id: u32) -> Option<u32> {
        let source = self.objects.get(&source_plane_id)?;
        let texture_id = source.texture_id;
        let source_vertices = source.vertices.clone();
        let vertex_count = source.vertex_count;

        let pool_mesh_id = if let Some(&existing) = self.source_plane_to_pool.get(&source_plane_id)
        {
            existing
        } else {
            let mesh_buffer = match upload_buffer(self.backend.as_mut(), &source_vertices) {
                Ok(handle) => handle,
                Err(e) => {
                    log::error!("Failed to create plane-instance mesh buffer: {e}");
                    return None;
                }
            };

            let initial_capacity_slots = 8usize;
            let seed: Vec<InstanceTransform> =
                vec![InstanceTransform::default(); initial_capacity_slots];
            let instance_buffer = match upload_instance_buffer(self.backend.as_mut(), &seed) {
                Ok(handle) => handle,
                Err(e) => {
                    log::error!("Failed to create plane-instance buffer: {e}");
                    self.backend.destroy_buffer(mesh_buffer);
                    return None;
                }
            };

            let new_mesh_id = self.next_instanced_mesh_id;
            self.next_instanced_mesh_id += 1;
            self.instanced_meshes.insert(
                new_mesh_id,
                InstancedMesh {
                    mesh_buffer,
                    vertex_count: vertex_count as u32,
                    instance_buffer,
                    instances: Vec::with_capacity(initial_capacity_slots),
                    texture_id,
                },
            );

            self.plane_instance_pools.insert(
                new_mesh_id,
                PlaneInstancePool {
                    source_plane_id,
                    instance_ids: Vec::with_capacity(initial_capacity_slots),
                    buffer_capacity_slots: initial_capacity_slots,
                    dirty: false,
                },
            );
            self.source_plane_to_pool
                .insert(source_plane_id, new_mesh_id);
            new_mesh_id
        };

        // Append the slot before allocating an id so a missing pool short-circuits
        // without burning an id from the shared object-id counter.
        let mesh = self.instanced_meshes.get_mut(&pool_mesh_id)?;
        let pool = self.plane_instance_pools.get_mut(&pool_mesh_id)?;

        let instance_id = self.next_object_id;
        self.next_object_id = self.next_object_id.wrapping_add(1);
        if self.next_object_id == 0 {
            self.next_object_id = 1;
        }

        let slot = mesh.instances.len();
        mesh.instances.push(InstanceTransform::default());
        pool.instance_ids.push(instance_id);
        pool.dirty = true;

        self.plane_instance_index
            .insert(instance_id, (pool_mesh_id, slot));

        Some(instance_id)
    }

    /// Cascade-destroy any plane-instance pool seeded from `source_plane_id`.
    ///
    /// Called from `remove_object` when the user destroys the source plane:
    /// the pool's GPU buffers are freed, every instance handle is invalidated,
    /// and the `source_plane_to_pool` reverse map is cleaned up so that the
    /// source-plane id cannot route future instances into a stale pool.
    pub(in crate::libs::graphics::renderer3d) fn destroy_plane_pool_for_source(
        &mut self,
        source_plane_id: u32,
    ) {
        let Some(mesh_id) = self.source_plane_to_pool.remove(&source_plane_id) else {
            return;
        };
        if let Some(pool) = self.plane_instance_pools.remove(&mesh_id) {
            for instance_id in &pool.instance_ids {
                self.plane_instance_index.remove(instance_id);
            }
        }
        if let Some(mesh) = self.instanced_meshes.remove(&mesh_id) {
            self.backend.destroy_buffer(mesh.mesh_buffer);
            self.backend.destroy_buffer(mesh.instance_buffer);
        }
    }

    /// Update a plane-instance transform component (position/rotation/scale).
    ///
    /// Returns `true` when `id` refers to a plane instance and was updated.
    /// Intended to be called from the regular object setters before they fall
    /// back to the dense-object map.
    pub(in crate::libs::graphics::renderer3d) fn try_update_plane_instance_transform(
        &mut self,
        id: u32,
        position: Option<Vector3<f32>>,
        rotation: Option<Vector3<f32>>,
        scale: Option<Vector3<f32>>,
    ) -> bool {
        let Some(&(mesh_id, slot)) = self.plane_instance_index.get(&id) else {
            return false;
        };
        let Some(mesh) = self.instanced_meshes.get_mut(&mesh_id) else {
            return false;
        };
        let Some(slot_data) = mesh.instances.get_mut(slot) else {
            return false;
        };
        if let Some(p) = position {
            slot_data.position = p;
        }
        if let Some(r) = rotation {
            slot_data.rotation = r;
        }
        if let Some(s) = scale {
            slot_data.scale = s;
        }
        if let Some(pool) = self.plane_instance_pools.get_mut(&mesh_id) {
            pool.dirty = true;
        }
        true
    }

    /// Remove a plane instance, swap-removing its slot to keep the buffer dense.
    ///
    /// Returns `true` when `id` referred to a plane instance.
    pub(in crate::libs::graphics::renderer3d) fn try_remove_plane_instance(
        &mut self,
        id: u32,
    ) -> bool {
        let Some((mesh_id, slot)) = self.plane_instance_index.remove(&id) else {
            return false;
        };

        let mut destroy_pool = false;
        if let Some(mesh) = self.instanced_meshes.get_mut(&mesh_id) {
            if let Some(pool) = self.plane_instance_pools.get_mut(&mesh_id) {
                let last = mesh.instances.len().saturating_sub(1);
                if slot < mesh.instances.len() {
                    if slot != last {
                        mesh.instances.swap(slot, last);
                        pool.instance_ids.swap(slot, last);
                        let moved_id = pool.instance_ids[slot];
                        self.plane_instance_index.insert(moved_id, (mesh_id, slot));
                    }
                    mesh.instances.pop();
                    pool.instance_ids.pop();
                    pool.dirty = true;
                }
                destroy_pool = mesh.instances.is_empty();
            }
        }

        if destroy_pool {
            if let Some(pool) = self.plane_instance_pools.remove(&mesh_id) {
                self.source_plane_to_pool.remove(&pool.source_plane_id);
            }
            if let Some(mesh) = self.instanced_meshes.remove(&mesh_id) {
                self.backend.destroy_buffer(mesh.mesh_buffer);
                self.backend.destroy_buffer(mesh.instance_buffer);
            }
        }

        true
    }

    /// Re-upload GPU instance buffers for any pool whose CPU instances changed
    /// since the last upload. Called once per frame before the instanced pass.
    pub(in crate::libs::graphics::renderer3d) fn flush_dirty_plane_instance_pools(&mut self) {
        // Reuse the persistent scratch Vec so the per-frame allocation is
        // bounded by the largest pool count we have ever seen.
        let mut dirty_ids = mem::take(&mut self.scratch_dirty_plane_pool_ids);
        dirty_ids.clear();
        for (id, pool) in &self.plane_instance_pools {
            if pool.dirty {
                dirty_ids.push(*id);
            }
        }
        if dirty_ids.is_empty() {
            self.scratch_dirty_plane_pool_ids = dirty_ids;
            return;
        }

        for mesh_id in dirty_ids.drain(..) {
            // Borrow the InstancedMesh for instance slice and capacity in one read.
            let needed_slots = match self.instanced_meshes.get(&mesh_id) {
                Some(m) => m.instances.len(),
                None => continue,
            };
            let capacity = self
                .plane_instance_pools
                .get(&mesh_id)
                .map(|p| p.buffer_capacity_slots)
                .unwrap_or(0);

            if needed_slots > capacity {
                // Grow path: swap the live instances out, pad to the new
                // capacity (zero-initialized tail slots are not drawn since
                // the draw call's instance_count uses the live len), upload,
                // swap the live vec back. No .clone() of the instance vec.
                let new_capacity = needed_slots.next_power_of_two().max(8);
                let mut padded = match self.instanced_meshes.get_mut(&mesh_id) {
                    Some(m) => mem::take(&mut m.instances),
                    None => continue,
                };
                padded.resize(new_capacity, InstanceTransform::default());

                let upload_result = upload_instance_buffer(self.backend.as_mut(), &padded);

                // Restore the live slice (truncate the tail back to needed_slots).
                padded.truncate(needed_slots);
                if let Some(mesh) = self.instanced_meshes.get_mut(&mesh_id) {
                    mesh.instances = padded;
                }

                let new_buffer = match upload_result {
                    Ok(handle) => handle,
                    Err(e) => {
                        log::error!("Failed to grow plane-instance buffer: {e}");
                        continue;
                    }
                };
                if let Some(mesh) = self.instanced_meshes.get_mut(&mesh_id) {
                    let old_buffer = mesh.instance_buffer;
                    mesh.instance_buffer = new_buffer;
                    self.backend.destroy_buffer(old_buffer);
                }
                if let Some(pool) = self.plane_instance_pools.get_mut(&mesh_id) {
                    pool.buffer_capacity_slots = new_capacity;
                    pool.dirty = false;
                }
            } else {
                // In-place update: split-borrow `self.instanced_meshes` and
                // `self.backend` (disjoint fields) to avoid cloning the Vec.
                let (buffer_handle, instances_ptr, instances_len) =
                    match self.instanced_meshes.get(&mesh_id) {
                        Some(m) => (m.instance_buffer, m.instances.as_ptr(), m.instances.len()),
                        None => continue,
                    };
                // SAFETY: instances_ptr/len are derived from
                // self.instanced_meshes[mesh_id].instances above. No code in
                // update_instance_buffer mutates self.instanced_meshes (it
                // only touches self.backend, a disjoint field), so the slice
                // is valid for the duration of this call.
                let slice = unsafe { std::slice::from_raw_parts(instances_ptr, instances_len) };
                if let Err(e) = update_instance_buffer(self.backend.as_mut(), buffer_handle, slice)
                {
                    log::error!("Failed to update plane-instance buffer: {e}");
                    continue;
                }
                if let Some(pool) = self.plane_instance_pools.get_mut(&mesh_id) {
                    pool.dirty = false;
                }
            }
        }

        self.scratch_dirty_plane_pool_ids = dirty_ids;
    }
}
