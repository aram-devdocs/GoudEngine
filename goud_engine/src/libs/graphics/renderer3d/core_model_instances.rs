//! Model instancing methods for [`Renderer3D`].

use super::animation::AnimationPlayer;
use super::core::Renderer3D;
use super::model::ModelInstance3D;
use super::types::Object3D;
use cgmath::Vector3;

impl Renderer3D {
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

            // Clone the material (cheap -- just a few floats).
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
}
