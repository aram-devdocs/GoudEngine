//! Model loading and management methods for [`Renderer3D`].

mod lifecycle;

use super::animation::{bake_animations, AnimationPlayer, BoneChannelMap};
use super::core::Renderer3D;
use super::mesh::upload_buffer;
use super::model::Model3D;
use super::types::{Material3D, MaterialType, Object3D, PbrProperties};
use crate::core::types::ModelData;
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
        let floats_per_vertex: usize = if is_skinned { 16 } else { 8 };
        let mut mesh_object_ids = Vec::new();
        let mut mesh_material_ids = Vec::new();
        let mut bind_pose_vertices: Vec<Vec<f32>> = Vec::new();
        let mut bind_pose_bone_indices: Vec<Vec<[u32; 4]>> = Vec::new();
        let mut bind_pose_bone_weights: Vec<Vec<[f32; 4]>> = Vec::new();

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

            bind_pose_vertices.push(verts.clone());
            bind_pose_bone_indices.push(sub_bi);
            bind_pose_bone_weights.push(sub_bw);

            let object_id = self.next_object_id;
            self.next_object_id = self.next_object_id.wrapping_add(1);
            if self.next_object_id == 0 {
                self.next_object_id = 1;
            }
            let tri_vert_count = verts.len() / floats_per_vertex;
            let bounds = super::types::compute_bounding_sphere(&verts);
            self.objects.insert(
                object_id,
                Object3D {
                    buffer,
                    vertex_count: tri_vert_count as i32,
                    // CPU-side vertex copy for static batching (8 FPV layout).
                    // Skinned models use 16 FPV and are never static-batched.
                    vertices: if is_skinned {
                        Vec::new()
                    } else {
                        verts.clone()
                    },
                    position: Vector3::new(0.0, 0.0, 0.0),
                    rotation: Vector3::new(0.0, 0.0, 0.0),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    texture_id: 0,
                    bounds,
                    is_static: false,
                },
            );

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

            if let Some(mesh_mat) = material_opt {
                if let Some(ref tex_path) = mesh_mat.base_color_texture_path {
                    let model_dir = std::path::Path::new(source_path)
                        .parent()
                        .unwrap_or(std::path::Path::new(""));
                    let full_path = model_dir.join(tex_path);
                    if let Ok(img) = image::open(&full_path) {
                        let rgba = img.to_rgba8();
                        let (w, h) = rgba.dimensions();
                        use crate::libs::graphics::backend::types::{
                            TextureFilter, TextureFormat, TextureWrap,
                        };
                        if let Ok(tex_handle) = self.backend.create_texture(
                            w,
                            h,
                            TextureFormat::RGBA8,
                            TextureFilter::Linear,
                            TextureWrap::Repeat,
                            rgba.as_raw(),
                        ) {
                            if let Some(obj) = self.objects.get_mut(&object_id) {
                                obj.texture_id = tex_handle.index();
                            }
                        }
                    }
                }
            }

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
        self.next_model_id = self.next_model_id.wrapping_add(1);
        if self.next_model_id == 0 {
            self.next_model_id = 1;
        }

        if let Some(ref skeleton) = model_data.skeleton {
            let player = AnimationPlayer::new(skeleton.bones.len());
            self.animation_players.insert(model_id, player);
        }

        let bone_channel_maps: Vec<BoneChannelMap> = if let Some(ref skel) = model_data.skeleton {
            model_data
                .animations
                .iter()
                .map(|anim| BoneChannelMap::build(skel, anim))
                .collect()
        } else {
            Vec::new()
        };

        let baked_animation = if let Some(ref skel) = model_data.skeleton {
            if !model_data.animations.is_empty() {
                Some(bake_animations(
                    skel,
                    &model_data.animations,
                    &bone_channel_maps,
                    self.config.skinning.baked_animation_sample_rate,
                ))
            } else {
                None
            }
        } else {
            None
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
                bone_channel_maps,
                baked_animation,
            },
        );

        if is_skinned {
            if let Some(m) = self.models.get(&model_id) {
                self.skinned_object_ids
                    .extend(m.mesh_object_ids.iter().copied());
            }
        }

        model_id
    }
}
