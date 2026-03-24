//! Model loading and management methods for [`Renderer3D`].

use super::animation::{AnimationPlayer, BoneChannelMap, BonePropertyNames};
use super::core::Renderer3D;
use super::mesh::upload_buffer;
use super::model::{Model3D, ModelInstance3D};
use super::types::{Material3D, MaterialType, Object3D, PbrProperties};
use crate::core::types::{MeshBounds, ModelData};
use cgmath::{Vector3, Vector4};

impl Renderer3D {
    /// Load a model from parsed [`ModelData`] and return its handle.
    /// Returns `0` on failure (e.g. empty mesh or GPU upload error).
    pub fn load_model(&mut self, model_data: ModelData, source_path: &str) -> u32 {
        let mesh = &model_data.mesh;
        if mesh.is_empty() {
            log::warn!("load_model: mesh is empty for '{}'", source_path);
            return 0;
        }

        let has_skeleton = model_data.skeleton.is_some();
        let gpu_skinning = has_skeleton
            && matches!(self.config.skinning.mode, super::config::SkinningMode::Gpu)
            && self.backend.supports_storage_buffers();
        let is_skinned = gpu_skinning;
        // GPU skinning: 16 floats/vertex (pos+normal+uv+bone_ids+bone_weights).
        // CPU skinning: 8 floats/vertex (pos+normal+uv); bone data stored separately.
        let floats_per_vertex: usize = if is_skinned { 16 } else { 8 };
        let mut mesh_object_ids = Vec::new();
        let mut mesh_material_ids = Vec::new();
        let mut bind_pose_vertices: Vec<Vec<f32>> = Vec::new();
        let mut bind_pose_bone_indices: Vec<Vec<[u32; 4]>> = Vec::new();
        let mut bind_pose_bone_weights: Vec<Vec<[f32; 4]>> = Vec::new();

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
            let mut verts = Vec::with_capacity(count * floats_per_vertex);
            let mut sub_bi: Vec<[u32; 4]> = Vec::with_capacity(count);
            let mut sub_bw: Vec<[f32; 4]> = Vec::with_capacity(count);
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
                        if is_skinned {
                            // GPU skinning: interleave bone_ids and bone_weights
                            // as floats in the vertex buffer.
                            verts.extend_from_slice(&[
                                bi[0] as f32,
                                bi[1] as f32,
                                bi[2] as f32,
                                bi[3] as f32,
                            ]);
                            verts.extend_from_slice(&bw);
                        }
                        sub_bi.push(bi);
                        sub_bw.push(bw);
                    }
                }
            }

            if verts.is_empty() {
                continue;
            }

            // GPU skinning: use Static buffer (vertices include bone data, GPU deforms).
            // CPU skinning: use Dynamic buffer (CPU deforms and re-uploads each frame).
            let buffer = if has_skeleton && !is_skinned {
                use crate::libs::graphics::backend::{BufferType, BufferUsage};
                self.backend
                    .create_buffer(
                        BufferType::Vertex,
                        BufferUsage::Dynamic,
                        bytemuck::cast_slice(&verts),
                    )
                    .map_err(|e| format!("Buffer creation failed: {e}"))
            } else {
                upload_buffer(self.backend.as_mut(), &verts)
            };
            let buffer = match buffer {
                Ok(h) => h,
                Err(e) => {
                    log::error!("Failed to upload model sub-mesh buffer: {e}");
                    continue;
                }
            };

            // Store bind-pose data for CPU skinning.
            bind_pose_vertices.push(verts.clone());
            bind_pose_bone_indices.push(sub_bi);
            bind_pose_bone_weights.push(sub_bw);

            let object_id = self.next_object_id;
            self.next_object_id += 1;
            let tri_vert_count = verts.len() / floats_per_vertex;
            // Compute bounding sphere from the vertex data before discarding it.
            let bounds = super::types::compute_bounding_sphere(&verts);
            self.objects.insert(
                object_id,
                Object3D {
                    buffer,
                    vertex_count: tri_vert_count as i32,
                    // Model sub-mesh vertices are NOT stored on Object3D to save
                    // memory at scale.  Bind-pose data for CPU skinning is already
                    // stored separately on Model3D::bind_pose_vertices.  The CPU
                    // shadow rasterizer skips objects with empty vertices.
                    vertices: Vec::new(),
                    position: Vector3::new(0.0, 0.0, 0.0),
                    rotation: Vector3::new(0.0, 0.0, 0.0),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    texture_id: 0,
                    bounds,
                    is_static: false,
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

        // Pre-compute bone property name strings once at load time to avoid
        // per-frame format!() allocations during animation sampling.
        let bone_count = model_data.skeleton.as_ref().map_or(0, |s| s.bones.len());
        let cached_bone_prop_names: Vec<BonePropertyNames> =
            (0..bone_count).map(BonePropertyNames::new).collect();

        // Pre-compute channel index maps for each animation clip so that
        // per-frame sampling uses direct array indexing instead of string
        // HashMap lookups. One BoneChannelMap per animation.
        let bone_channel_maps: Vec<BoneChannelMap> = if let Some(ref skel) = model_data.skeleton {
            model_data
                .animations
                .iter()
                .map(|anim| BoneChannelMap::build(skel, anim))
                .collect()
        } else {
            Vec::new()
        };

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
                bind_pose_vertices,
                bind_pose_bone_indices,
                bind_pose_bone_weights,
                cached_bone_prop_names,
                bone_channel_maps,
            },
        );

        // Update the persistent skinned object ID set.
        if is_skinned {
            if let Some(m) = self.models.get(&model_id) {
                self.skinned_object_ids
                    .extend(m.mesh_object_ids.iter().copied());
            }
        }

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

    /// Create an instance of a model with its own GPU resources.
    ///
    /// For skinned models each instance gets its own dynamic vertex buffer so
    /// that CPU skinning can deform vertices independently per instance.
    /// Non-skinned models still share the source buffer handle.
    ///
    /// Returns the instance handle, or `None` if the source model does not exist.
    pub fn instantiate_model(&mut self, source_id: u32) -> Option<u32> {
        let source = self.models.get(&source_id)?;
        let has_skeleton = source.skeleton.is_some();

        // Pre-collect bind-pose data we may need for creating instance buffers.
        let bind_poses: Vec<Vec<f32>> = if has_skeleton {
            source.bind_pose_vertices.clone()
        } else {
            Vec::new()
        };

        let mut instance_object_ids = Vec::with_capacity(source.mesh_object_ids.len());
        let mut instance_material_ids = Vec::with_capacity(source.mesh_material_ids.len());

        for (i, &src_obj_id) in source.mesh_object_ids.iter().enumerate() {
            let (src_buffer, vertex_count, texture_id, src_bounds) =
                match self.objects.get(&src_obj_id) {
                    Some(o) => (o.buffer, o.vertex_count, o.texture_id, o.bounds),
                    None => continue,
                };

            // CPU-skinned instances need their own dynamic buffer for per-frame re-upload.
            // GPU-skinned instances share the source buffer (GPU deforms via shader).
            let buffer = if has_skeleton && !source.is_skinned {
                if let Some(bp) = bind_poses.get(i) {
                    use crate::libs::graphics::backend::{BufferType, BufferUsage};
                    match self.backend.create_buffer(
                        BufferType::Vertex,
                        BufferUsage::Dynamic,
                        bytemuck::cast_slice(bp),
                    ) {
                        Ok(h) => h,
                        Err(e) => {
                            log::error!("Failed to create instance buffer: {e}");
                            src_buffer
                        }
                    }
                } else {
                    src_buffer
                }
            } else {
                src_buffer
            };

            let new_obj_id = self.next_object_id;
            self.next_object_id += 1;
            self.objects.insert(
                new_obj_id,
                Object3D {
                    buffer,
                    vertex_count,
                    vertices: Vec::new(),
                    position: Vector3::new(0.0, 0.0, 0.0),
                    rotation: Vector3::new(0.0, 0.0, 0.0),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    texture_id,
                    bounds: src_bounds,
                    is_static: false,
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

        // Update skinned object ID set if the source model is skinned.
        if source.is_skinned {
            self.skinned_object_ids
                .extend(instance_object_ids.iter().copied());
        }

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

    /// Apply a mutation to a specific sub-mesh material's PBR properties.
    fn set_pbr_map(
        &mut self,
        model_id: u32,
        mesh_index: usize,
        f: impl FnOnce(&mut Material3D),
    ) -> bool {
        let mat_ids = self.collect_model_material_ids(model_id);
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

    /// Collect object IDs for a model or model instance.
    fn collect_model_object_ids(&self, id: u32) -> Vec<u32> {
        self.models
            .get(&id)
            .map(|m| &m.mesh_object_ids)
            .or_else(|| self.model_instances.get(&id).map(|i| &i.mesh_object_ids))
            .cloned()
            .unwrap_or_default()
    }

    /// Collect material IDs for a model or model instance.
    fn collect_model_material_ids(&self, id: u32) -> Vec<u32> {
        self.models
            .get(&id)
            .map(|m| &m.mesh_material_ids)
            .or_else(|| self.model_instances.get(&id).map(|i| &i.mesh_material_ids))
            .cloned()
            .unwrap_or_default()
    }

    /// Apply a mutation to all Object3D sub-meshes of a model or instance.
    fn set_model_transform(&mut self, id: u32, f: impl Fn(&mut Object3D)) -> bool {
        let obj_ids = self.collect_model_object_ids(id);
        if obj_ids.is_empty() {
            return false;
        }
        for obj_id in obj_ids {
            if let Some(obj) = self.objects.get_mut(&obj_id) {
                f(obj);
            }
        }
        true
    }
}
