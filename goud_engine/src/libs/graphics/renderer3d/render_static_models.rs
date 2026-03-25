//! Instanced rendering path for imported static models.

use std::collections::{HashMap, HashSet};

use super::core::Renderer3D;
use super::frustum::Frustum;
use super::mesh::pack_instance_data;
use super::texture::TextureManagerTrait;
use super::types::InstanceTransform;
use crate::libs::graphics::backend::{
    types::{BufferType, BufferUsage, TextureHandle},
    PrimitiveTopology, VertexBufferBinding,
};
use cgmath::Vector4;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct StaticModelGroupKey {
    buffer: crate::libs::graphics::backend::BufferHandle,
    vertex_count: u32,
    color_bits: [u32; 4],
    texture_id: u32,
}

impl Renderer3D {
    pub(super) fn render_static_model_instances(
        &mut self,
        scene_model_filter: Option<&HashSet<u32>>,
        frustum: Option<&Frustum>,
        view_arr: &[f32; 16],
        proj_arr: &[f32; 16],
        shadow_matrix: &[f32; 16],
        shadows_enabled: bool,
        fog: &super::types::FogConfig,
        lights: &[super::types::Light],
        texture_manager: Option<&dyn TextureManagerTrait>,
    ) -> HashSet<u32> {
        struct StaticModelGroup {
            color: [f32; 4],
            instances: Vec<InstanceTransform>,
        }

        let mut handled_object_ids = HashSet::new();
        let mut groups: HashMap<StaticModelGroupKey, StaticModelGroup> = HashMap::new();
        let static_model_ids: Vec<u32> = self.static_model_ids.iter().copied().collect();

        for model_id in static_model_ids {
            if let Some(filter) = scene_model_filter {
                if !filter.contains(&model_id) {
                    continue;
                }
            }

            let (source_model_id, object_ids, material_ids) =
                if let Some(model) = self.models.get(&model_id) {
                    (
                        model_id,
                        model.mesh_object_ids.clone(),
                        model.mesh_material_ids.clone(),
                    )
                } else if let Some(instance) = self.model_instances.get(&model_id) {
                    (
                        instance.source_model_id,
                        instance.mesh_object_ids.clone(),
                        instance.mesh_material_ids.clone(),
                    )
                } else {
                    continue;
                };

            let Some(source_model) = self.models.get(&source_model_id) else {
                continue;
            };
            if source_model.skeleton.is_some() {
                continue;
            }

            let Some(&first_object_id) = object_ids.first() else {
                continue;
            };
            let Some(first_object) = self.objects.get(&first_object_id) else {
                continue;
            };

            if let Some(frustum) = frustum {
                let center = [
                    (source_model.bounds.min[0] + source_model.bounds.max[0]) * 0.5,
                    (source_model.bounds.min[1] + source_model.bounds.max[1]) * 0.5,
                    (source_model.bounds.min[2] + source_model.bounds.max[2]) * 0.5,
                ];
                let extent = [
                    source_model.bounds.max[0] - center[0],
                    source_model.bounds.max[1] - center[1],
                    source_model.bounds.max[2] - center[2],
                ];
                let world_center = first_object.position
                    + cgmath::Vector3::new(
                        center[0] * first_object.scale.x.abs(),
                        center[1] * first_object.scale.y.abs(),
                        center[2] * first_object.scale.z.abs(),
                    );
                let max_scale = first_object
                    .scale
                    .x
                    .abs()
                    .max(first_object.scale.y.abs())
                    .max(first_object.scale.z.abs());
                let world_radius =
                    (extent[0] * extent[0] + extent[1] * extent[1] + extent[2] * extent[2])
                        .sqrt()
                        * max_scale;
                if !frustum.intersects_sphere(world_center, world_radius) {
                    continue;
                }
            }

            for (submesh_index, object_id) in object_ids.iter().copied().enumerate() {
                let Some(object) = self.objects.get(&object_id) else {
                    continue;
                };
                handled_object_ids.insert(object_id);

                let material_id = material_ids.get(submesh_index).copied().unwrap_or(0);
                let color = self.resolve_material_color(material_id, object.texture_id);
                let key = StaticModelGroupKey {
                    buffer: object.buffer,
                    vertex_count: object.vertex_count as u32,
                    color_bits: [
                        color[0].to_bits(),
                        color[1].to_bits(),
                        color[2].to_bits(),
                        color[3].to_bits(),
                    ],
                    texture_id: object.texture_id,
                };
                groups
                    .entry(key)
                    .or_insert_with(|| StaticModelGroup {
                        color,
                        instances: Vec::new(),
                    })
                    .instances
                    .push(InstanceTransform {
                        position: object.position,
                        rotation: object.rotation,
                        scale: object.scale,
                        color: Vector4::new(color[0], color[1], color[2], color[3]),
                    });
            }
        }

        if groups.is_empty() {
            return handled_object_ids;
        }

        let _ = self.backend.bind_shader(self.instanced_shader_handle);
        let instanced_uniforms = self.instanced_uniforms.clone();
        self.apply_main_uniforms(
            view_arr,
            proj_arr,
            shadow_matrix,
            shadows_enabled,
            &instanced_uniforms,
            fog,
            lights,
        );

        for (key, group) in groups {
            let packed = pack_instance_data(&group.instances);
            let bytes = bytemuck::cast_slice(packed.as_slice());

            if self.static_model_instance_buffer.is_none()
                || self.static_model_instance_buffer_size < bytes.len()
            {
                if let Some(old) = self.static_model_instance_buffer.take() {
                    self.backend.destroy_buffer(old);
                }
                let alloc_size = bytes.len().next_power_of_two().max(64);
                let initial = vec![0u8; alloc_size];
                match self
                    .backend
                    .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, &initial)
                {
                    Ok(handle) => {
                        self.static_model_instance_buffer = Some(handle);
                        self.static_model_instance_buffer_size = alloc_size;
                    }
                    Err(error) => {
                        log::error!("Failed to create static model instance buffer: {error}");
                        continue;
                    }
                }
            }

            let Some(instance_buffer) = self.static_model_instance_buffer else {
                continue;
            };
            if let Err(error) = self.backend.update_buffer(instance_buffer, 0, bytes) {
                log::error!("Failed to update static model instance buffer: {error}");
                continue;
            }

            if key.texture_id > 0 {
                if let Some(texture_manager) = texture_manager {
                    texture_manager.bind_texture(key.texture_id, 0);
                } else {
                    let texture_handle = TextureHandle::new(key.texture_id, 1);
                    let _ = self.backend.bind_texture(texture_handle, 0);
                }
                self.backend
                    .set_uniform_int(self.instanced_uniforms.use_texture, 1);
                self.stats.texture_binds += 1;
            } else {
                self.backend
                    .set_uniform_int(self.instanced_uniforms.use_texture, 0);
            }

            self.backend.set_uniform_vec4(
                self.instanced_uniforms.object_color,
                group.color[0],
                group.color[1],
                group.color[2],
                group.color[3],
            );
            let bindings = [
                VertexBufferBinding::per_vertex(key.buffer, self.object_layout.clone()),
                VertexBufferBinding::per_instance(instance_buffer, self.instance_layout.clone()),
            ];
            let _ = self.backend.set_vertex_bindings(&bindings);
            let _ = self.backend.draw_arrays_instanced(
                PrimitiveTopology::Triangles,
                0,
                key.vertex_count,
                group.instances.len() as u32,
            );
            self.stats.draw_calls += 1;
            self.stats.instanced_draw_calls += 1;
            self.stats.active_instances += group.instances.len() as u32;
        }

        self.backend.unbind_shader();
        handled_object_ids
    }
}
