//! Model loading and management methods for [`Renderer3D`].

use super::animation::AnimationPlayer;
use super::core::Renderer3D;
use super::mesh::upload_buffer;
use super::model::{Model3D, ModelInstance3D};
use super::types::{Material3D, MaterialType, Object3D, PbrProperties};
use crate::core::types::{MeshBounds, ModelData};
use cgmath::{Vector3, Vector4};

impl Renderer3D {
    /// Load a model from parsed [`ModelData`] and return its handle.
    ///
    /// For each sub-mesh:
    /// 1. Extracts and packs vertices into the interleaved float layout.
    /// 2. Uploads a GPU buffer via the backend.
    /// 3. Creates an `Object3D` entry.
    /// 4. Creates a `Material3D` from the sub-mesh material properties.
    /// 5. Binds the material to the object.
    ///
    /// Returns `0` on failure (e.g. empty mesh or GPU upload error).
    pub fn load_model(&mut self, model_data: ModelData, source_path: &str) -> u32 {
        let mesh = &model_data.mesh;
        if mesh.is_empty() {
            log::warn!("load_model: mesh is empty for '{}'", source_path);
            return 0;
        }

        // Use CPU skinning via the standard shader — the GPU skinned shader
        // has projection issues. Vertex buffers are re-uploaded each frame
        // with bone-deformed positions by update_animations().
        let is_skinned = false;
        let floats_per_vertex: usize = if is_skinned { 16 } else { 8 };
        let mut mesh_object_ids = Vec::new();
        let mut mesh_material_ids = Vec::new();
        let mut all_bind_pose: Vec<Vec<([f32; 3], [f32; 3], [f32; 2], [u32; 4], [f32; 4])>> = Vec::new();

        // Skeleton bone data (parallel to mesh.vertices) for skinned models.
        let bone_indices = model_data
            .skeleton
            .as_ref()
            .map(|s| &s.bone_indices[..])
            .unwrap_or(&[]);
        let bone_weights = model_data
            .skeleton
            .as_ref()
            .map(|s| &s.bone_weights[..])
            .unwrap_or(&[]);

        // Process each sub-mesh as a separate Object3D.
        let sub_mesh_list: Vec<_> = if mesh.sub_meshes.is_empty() {
            vec![(0u32, mesh.indices.len() as u32, None)]
        } else {
            mesh.sub_meshes
                .iter()
                .map(|sm| (sm.start_index, sm.index_count, sm.material.as_ref()))
                .collect()
        };

        for (start_index, index_count, material_opt) in &sub_mesh_list {
            let start = *start_index as usize;
            let count = *index_count as usize;
            let end = (start + count).min(mesh.indices.len());
            let sub_indices = &mesh.indices[start..end];

            let vert_count = mesh.vertices.len();
            let has_skeleton = model_data.skeleton.is_some();
            let mut verts = Vec::with_capacity(count * floats_per_vertex);
            let mut bind_verts: Vec<([f32; 3], [f32; 3], [f32; 2], [u32; 4], [f32; 4])> =
                if has_skeleton { Vec::with_capacity(count) } else { Vec::new() };
            for &idx in sub_indices {
                let vi = idx as usize;
                if vi < vert_count {
                    let v = &mesh.vertices[vi];
                    verts.extend_from_slice(&v.position);
                    verts.extend_from_slice(&v.normal);
                    verts.extend_from_slice(&v.uv);
                    if has_skeleton {
                        let bi = bone_indices.get(vi).copied().unwrap_or([0; 4]);
                        let bw = bone_weights.get(vi).copied().unwrap_or([0.0; 4]);
                        bind_verts.push((v.position, v.normal, v.uv, bi, bw));
                    }
                }
            }

            if verts.is_empty() {
                continue;
            }

            let buffer = match upload_buffer(self.backend.as_mut(), &verts) {
                Ok(h) => h,
                Err(e) => {
                    log::error!("Failed to upload model sub-mesh buffer: {e}");
                    continue;
                }
            };

            let object_id = self.next_object_id;
            self.next_object_id += 1;
            let tri_vert_count = verts.len() / floats_per_vertex;
            self.objects.insert(
                object_id,
                Object3D {
                    buffer,
                    vertex_count: tri_vert_count as i32,
                    vertices: verts,
                    position: Vector3::new(0.0, 0.0, 0.0),
                    rotation: Vector3::new(0.0, 0.0, 0.0),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    texture_id: 0,
                },
            );

            // Create material from glTF material metadata.
            let material = if let Some(mesh_mat) = material_opt {
                Material3D {
                    material_type: MaterialType::Pbr,
                    color: Vector4::new(
                        mesh_mat.base_color_factor[0],
                        mesh_mat.base_color_factor[1],
                        mesh_mat.base_color_factor[2],
                        mesh_mat.base_color_factor[3],
                    ),
                    shininess: 32.0,
                    pbr: PbrProperties {
                        metallic: mesh_mat.metallic_factor,
                        roughness: mesh_mat.roughness_factor,
                        ao: 1.0,
                        albedo_map: 0,
                        normal_map: 0,
                        metallic_roughness_map: 0,
                    },
                }
            } else {
                Material3D::default()
            };

            let material_id = self.next_material_id;
            self.next_material_id += 1;
            self.materials.insert(material_id, material);
            self.object_materials.insert(object_id, material_id);

            mesh_object_ids.push(object_id);
            mesh_material_ids.push(material_id);
            all_bind_pose.push(bind_verts);
        }

        if mesh_object_ids.is_empty() {
            log::warn!("load_model: no sub-meshes created for '{}'", source_path);
            return 0;
        }

        let model_id = self.next_model_id;
        self.next_model_id += 1;

        // Create animation player if the model has a skeleton.
        if let Some(ref skeleton) = model_data.skeleton {
            let player = AnimationPlayer::new(skeleton.bones.len());
            self.animation_players.insert(model_id, player);
        }

        self.models.insert(
            model_id,
            Model3D {
                mesh_object_ids,
                mesh_material_ids,
                bounds: model_data.mesh.bounds,
                source_path: source_path.to_string(),
                skeleton: model_data.skeleton,
                animations: model_data.animations,
                is_skinned,
                bind_pose_vertices: all_bind_pose,
            },
        );

        model_id
    }

    /// Destroy a model, removing all its owned objects and materials.
    ///
    /// Returns `true` if the model existed and was removed.
    pub fn destroy_model(&mut self, model_id: u32) -> bool {
        let model = match self.models.remove(&model_id) {
            Some(m) => m,
            None => return false,
        };

        for &obj_id in &model.mesh_object_ids {
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

    /// Create a lightweight instance that shares the source model's GPU buffers.
    ///
    /// Each instance gets its own Object3D entries (for independent transforms)
    /// but reuses the source model's vertex buffer handles — no GPU duplication.
    /// Returns the instance handle, or `None` if the source model does not exist.
    pub fn instantiate_model(&mut self, source_id: u32) -> Option<u32> {
        let source = self.models.get(&source_id)?;

        let mut instance_object_ids = Vec::with_capacity(source.mesh_object_ids.len());
        let mut instance_material_ids = Vec::with_capacity(source.mesh_material_ids.len());

        for (i, &src_obj_id) in source.mesh_object_ids.iter().enumerate() {
            let (buffer, vertex_count, texture_id) = match self.objects.get(&src_obj_id) {
                Some(o) => (o.buffer, o.vertex_count, o.texture_id),
                None => continue,
            };

            // Reuse the source model's GPU buffer — no upload, no clone.
            let new_obj_id = self.next_object_id;
            self.next_object_id += 1;
            self.objects.insert(
                new_obj_id,
                Object3D {
                    buffer, // shared GPU buffer handle
                    vertex_count,
                    vertices: Vec::new(), // no CPU copy needed
                    position: Vector3::new(0.0, 0.0, 0.0),
                    rotation: Vector3::new(0.0, 0.0, 0.0),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    texture_id,
                },
            );

            // Clone the material (cheap — just a few floats).
            let src_mat_id = source
                .mesh_material_ids
                .get(i)
                .and_then(|id| self.materials.get(id));
            let new_mat_id = if let Some(mat) = src_mat_id {
                let mid = self.next_material_id;
                self.next_material_id += 1;
                self.materials.insert(mid, mat.clone());
                self.object_materials.insert(new_obj_id, mid);
                mid
            } else {
                0
            };

            instance_object_ids.push(new_obj_id);
            instance_material_ids.push(new_mat_id);
        }

        if instance_object_ids.is_empty() {
            return None;
        }

        let instance_id = self.next_model_id;
        self.next_model_id += 1;

        // Create animation player if the source model has a skeleton.
        if let Some(ref skeleton) = source.skeleton {
            let player = AnimationPlayer::new(skeleton.bones.len());
            self.animation_players.insert(instance_id, player);
        }

        self.model_instances.insert(
            instance_id,
            ModelInstance3D {
                source_model_id: source_id,
                mesh_object_ids: instance_object_ids.clone(),
                mesh_material_ids: instance_material_ids,
            },
        );

        // If the source model is in a scene, add the new instance's objects to that scene too.
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                if scene.models.contains(&source_id) {
                    scene.add_model(instance_id);
                    for obj_id in &instance_object_ids {
                        scene.add_object(*obj_id);
                    }
                }
            }
        }

        Some(instance_id)
    }

    /// Set position on all sub-mesh objects of a model or instance.
    pub fn set_model_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        let obj_ids = self.collect_model_object_ids(id);
        if obj_ids.is_empty() {
            return false;
        }
        for obj_id in obj_ids {
            if let Some(obj) = self.objects.get_mut(&obj_id) {
                obj.position = Vector3::new(x, y, z);
            }
        }
        true
    }

    /// Set rotation (degrees) on all sub-mesh objects of a model or instance.
    pub fn set_model_rotation(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        let obj_ids = self.collect_model_object_ids(id);
        if obj_ids.is_empty() {
            return false;
        }
        for obj_id in obj_ids {
            if let Some(obj) = self.objects.get_mut(&obj_id) {
                obj.rotation = Vector3::new(x, y, z);
            }
        }
        true
    }

    /// Set scale on all sub-mesh objects of a model or instance.
    pub fn set_model_scale(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        let obj_ids = self.collect_model_object_ids(id);
        if obj_ids.is_empty() {
            return false;
        }
        for obj_id in obj_ids {
            if let Some(obj) = self.objects.get_mut(&obj_id) {
                obj.scale = Vector3::new(x, y, z);
            }
        }
        true
    }

    /// Override the material on a specific sub-mesh of a model or instance.
    ///
    /// If `mesh_index` is negative, applies the material to all sub-meshes.
    pub fn set_model_material(&mut self, model_id: u32, mesh_index: i32, material_id: u32) -> bool {
        let obj_ids = self.collect_model_object_ids(model_id);
        if obj_ids.is_empty() {
            return false;
        }

        if mesh_index < 0 {
            // Apply to all sub-meshes.
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
    ///
    /// Instance bounding boxes are inherited from their source model.
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
    ///
    /// `texture_id` is the GPU texture handle packed as `u32`.
    pub fn set_model_texture(&mut self, model_id: u32, mesh_index: i32, texture_id: u32) -> bool {
        let obj_ids = self.collect_model_object_ids(model_id);
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
    pub fn set_model_material_albedo_map(
        &mut self,
        model_id: u32,
        mesh_index: usize,
        texture_id: u32,
    ) -> bool {
        let mat_ids = self.collect_model_material_ids(model_id);
        if mesh_index >= mat_ids.len() {
            return false;
        }
        if let Some(mat) = self.materials.get_mut(&mat_ids[mesh_index]) {
            mat.pbr.albedo_map = texture_id;
            true
        } else {
            false
        }
    }

    /// Set the PBR normal map texture on a model's material.
    pub fn set_model_material_normal_map(
        &mut self,
        model_id: u32,
        mesh_index: usize,
        texture_id: u32,
    ) -> bool {
        let mat_ids = self.collect_model_material_ids(model_id);
        if mesh_index >= mat_ids.len() {
            return false;
        }
        if let Some(mat) = self.materials.get_mut(&mat_ids[mesh_index]) {
            mat.pbr.normal_map = texture_id;
            true
        } else {
            false
        }
    }

    /// Set the PBR metallic-roughness map texture on a model's material.
    pub fn set_model_material_metallic_roughness_map(
        &mut self,
        model_id: u32,
        mesh_index: usize,
        texture_id: u32,
    ) -> bool {
        let mat_ids = self.collect_model_material_ids(model_id);
        if mesh_index >= mat_ids.len() {
            return false;
        }
        if let Some(mat) = self.materials.get_mut(&mat_ids[mesh_index]) {
            mat.pbr.metallic_roughness_map = texture_id;
            true
        } else {
            false
        }
    }

    // -- Internal helpers --

    /// Collect object IDs for a model or model instance.
    fn collect_model_object_ids(&self, id: u32) -> Vec<u32> {
        if let Some(m) = self.models.get(&id) {
            m.mesh_object_ids.clone()
        } else if let Some(inst) = self.model_instances.get(&id) {
            inst.mesh_object_ids.clone()
        } else {
            Vec::new()
        }
    }

    /// Collect material IDs for a model or model instance.
    fn collect_model_material_ids(&self, id: u32) -> Vec<u32> {
        if let Some(m) = self.models.get(&id) {
            m.mesh_material_ids.clone()
        } else if let Some(inst) = self.model_instances.get(&id) {
            inst.mesh_material_ids.clone()
        } else {
            Vec::new()
        }
    }
}
