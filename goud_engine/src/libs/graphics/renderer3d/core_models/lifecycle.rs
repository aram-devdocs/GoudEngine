//! Model destroy, property setters, and helper methods for [`Renderer3D`].

use super::super::core::Renderer3D;
use super::super::types::{Material3D, Object3D};
use crate::core::types::MeshBounds;
use cgmath::Vector3;

impl Renderer3D {
    /// Destroy a model, removing all its owned objects, materials, scene
    /// references, and orphaned instances.
    ///
    /// Returns `true` if the model existed and was removed.
    pub fn destroy_model(&mut self, model_id: u32) -> bool {
        if !self.models.contains_key(&model_id) {
            if self.model_instances.contains_key(&model_id) {
                return self.destroy_model_instance(model_id);
            }
            return false;
        }

        let model = self.models.get(&model_id).unwrap();
        log::trace!(
            "destroy_model({}): source='{}'",
            model_id,
            model.source_path
        );
        let obj_ids: Vec<u32> = model.mesh_object_ids.clone();

        for scene in self.scenes.values_mut() {
            scene.remove_model(model_id);
            for &oid in &obj_ids {
                scene.remove_object(oid);
            }
        }

        for &oid in &obj_ids {
            if self.objects.get(&oid).is_some_and(|o| o.is_static) {
                self.static_batch_dirty = true;
                break;
            }
        }

        let orphan_ids: Vec<u32> = self
            .model_instances
            .iter()
            .filter(|(_, inst)| inst.source_model_id == model_id)
            .map(|(&id, _)| id)
            .collect();
        for orphan_id in orphan_ids {
            self.destroy_model_instance(orphan_id);
        }

        let model = self.models.remove(&model_id).unwrap();

        for &obj_id in &model.mesh_object_ids {
            self.skinned_object_ids.remove(&obj_id);
            if let Some(obj) = self.objects.remove(&obj_id) {
                self.backend.destroy_buffer(obj.buffer);
            }
            self.object_materials.remove(&obj_id);
        }
        for &mat_id in &model.mesh_material_ids {
            self.materials.remove(&mat_id);
        }

        self.animation_players.remove(&model_id);

        true
    }

    /// Destroy a model instance, cleaning up its objects, materials, scene
    /// references, and animation player.
    pub(super) fn destroy_model_instance(&mut self, instance_id: u32) -> bool {
        let inst = match self.model_instances.remove(&instance_id) {
            Some(i) => i,
            None => return false,
        };

        for scene in self.scenes.values_mut() {
            scene.remove_model(instance_id);
            for &oid in &inst.mesh_object_ids {
                scene.remove_object(oid);
            }
        }

        let source_buffers: std::collections::HashSet<
            crate::libs::graphics::backend::BufferHandle,
        > = self
            .models
            .get(&inst.source_model_id)
            .map(|m| {
                m.mesh_object_ids
                    .iter()
                    .filter_map(|&oid| self.objects.get(&oid).map(|o| o.buffer))
                    .collect()
            })
            .unwrap_or_default();

        for &obj_id in &inst.mesh_object_ids {
            self.skinned_object_ids.remove(&obj_id);
            if let Some(obj) = self.objects.remove(&obj_id) {
                if obj.is_static {
                    self.static_batch_dirty = true;
                }
                if !source_buffers.contains(&obj.buffer) {
                    self.backend.destroy_buffer(obj.buffer);
                }
            }
            self.object_materials.remove(&obj_id);
        }
        for &mat_id in &inst.mesh_material_ids {
            self.materials.remove(&mat_id);
        }

        self.animation_players.remove(&instance_id);

        true
    }

    /// Set position on all sub-mesh objects of a model or instance.
    pub fn set_model_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        self.set_model_transform(id, |obj| obj.position = Vector3::new(x, y, z))
    }

    /// Set rotation (degrees) on all sub-mesh objects of a model or instance.
    pub fn set_model_rotation(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        self.set_model_transform(id, |obj| obj.rotation = Vector3::new(x, y, z))
    }

    /// Set scale on all sub-mesh objects of a model or instance.
    pub fn set_model_scale(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        self.set_model_transform(id, |obj| obj.scale = Vector3::new(x, y, z))
    }

    /// Mark all sub-mesh objects of a model or instance as static for batching.
    pub fn set_model_static(&mut self, model_id: u32, is_static: bool) -> bool {
        let obj_ids = self.collect_model_object_ids(model_id).to_vec();
        if obj_ids.is_empty() {
            return false;
        }
        for &oid in &obj_ids {
            if let Some(obj) = self.objects.get_mut(&oid) {
                obj.is_static = is_static;
            }
        }
        self.static_batch_dirty = true;
        true
    }

    /// Override the material on a specific sub-mesh of a model or instance.
    pub fn set_model_material(&mut self, model_id: u32, mesh_index: i32, material_id: u32) -> bool {
        let obj_ids = self.collect_model_object_ids(model_id).to_vec();
        if obj_ids.is_empty() {
            return false;
        }

        if mesh_index < 0 {
            for obj_id in &obj_ids {
                self.object_materials.insert(*obj_id, material_id);
            }
            true
        } else if (mesh_index as usize) < obj_ids.len() {
            self.object_materials
                .insert(obj_ids[mesh_index as usize], material_id);
            true
        } else {
            false
        }
    }

    /// Returns the number of sub-meshes in a model or instance.
    pub fn get_model_mesh_count(&self, model_id: u32) -> Option<usize> {
        if let Some(m) = self.models.get(&model_id) {
            Some(m.mesh_object_ids.len())
        } else {
            self.model_instances
                .get(&model_id)
                .map(|inst| inst.mesh_object_ids.len())
        }
    }

    /// Returns the AABB bounding box of a model.
    pub fn get_model_bounding_box(&self, model_id: u32) -> Option<MeshBounds> {
        if let Some(m) = self.models.get(&model_id) {
            Some(m.bounds)
        } else if let Some(inst) = self.model_instances.get(&model_id) {
            self.models.get(&inst.source_model_id).map(|m| m.bounds)
        } else {
            None
        }
    }

    /// Set a texture on a specific sub-mesh object of a model/instance.
    pub fn set_model_texture(&mut self, model_id: u32, mesh_index: i32, texture_id: u32) -> bool {
        let obj_ids = self.collect_model_object_ids(model_id).to_vec();
        if obj_ids.is_empty() {
            return false;
        }

        if mesh_index < 0 {
            for obj_id in &obj_ids {
                if let Some(obj) = self.objects.get_mut(obj_id) {
                    obj.texture_id = texture_id;
                }
            }
            true
        } else if (mesh_index as usize) < obj_ids.len() {
            if let Some(obj) = self.objects.get_mut(&obj_ids[mesh_index as usize]) {
                obj.texture_id = texture_id;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Set the PBR albedo map texture on a model's material.
    pub fn set_model_material_albedo_map(&mut self, id: u32, idx: usize, tex: u32) -> bool {
        self.set_pbr_map(id, idx, |mat| mat.pbr.albedo_map = tex)
    }

    /// Set the PBR normal map texture on a model's material.
    pub fn set_model_material_normal_map(&mut self, id: u32, idx: usize, tex: u32) -> bool {
        self.set_pbr_map(id, idx, |mat| mat.pbr.normal_map = tex)
    }

    /// Set the PBR metallic-roughness map texture on a model's material.
    pub fn set_model_material_metallic_roughness_map(
        &mut self,
        id: u32,
        idx: usize,
        tex: u32,
    ) -> bool {
        self.set_pbr_map(id, idx, |mat| mat.pbr.metallic_roughness_map = tex)
    }

    // -- Internal helpers --

    fn set_pbr_map(
        &mut self,
        model_id: u32,
        mesh_index: usize,
        f: impl FnOnce(&mut Material3D),
    ) -> bool {
        let mat_ids = self.collect_model_material_ids(model_id).to_vec();
        if mesh_index >= mat_ids.len() {
            return false;
        }
        if let Some(mat) = self.materials.get_mut(&mat_ids[mesh_index]) {
            f(mat);
            true
        } else {
            false
        }
    }

    pub(in crate::libs::graphics::renderer3d) fn collect_model_object_ids(
        &self,
        id: u32,
    ) -> &[u32] {
        self.models
            .get(&id)
            .map(|m| m.mesh_object_ids.as_slice())
            .or_else(|| {
                self.model_instances
                    .get(&id)
                    .map(|i| i.mesh_object_ids.as_slice())
            })
            .unwrap_or(&[])
    }

    pub(in crate::libs::graphics::renderer3d) fn collect_model_material_ids(
        &self,
        id: u32,
    ) -> &[u32] {
        self.models
            .get(&id)
            .map(|m| m.mesh_material_ids.as_slice())
            .or_else(|| {
                self.model_instances
                    .get(&id)
                    .map(|i| i.mesh_material_ids.as_slice())
            })
            .unwrap_or(&[])
    }

    fn set_model_transform(&mut self, id: u32, f: impl Fn(&mut Object3D)) -> bool {
        let obj_ids = self.collect_model_object_ids(id).to_vec();
        if obj_ids.is_empty() {
            return false;
        }
        let mut has_static = false;
        for obj_id in obj_ids {
            if let Some(obj) = self.objects.get_mut(&obj_id) {
                f(obj);
                if obj.is_static {
                    has_static = true;
                }
            }
        }
        if has_static {
            self.static_batch_dirty = true;
        }
        true
    }
}
