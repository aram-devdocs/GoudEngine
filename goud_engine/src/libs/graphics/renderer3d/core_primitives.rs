use super::core::Renderer3D;
use super::mesh::{
    generate_cube_vertices, generate_cylinder_vertices, generate_plane_vertices,
    generate_sphere_vertices, update_instance_buffer, upload_buffer, upload_instance_buffer,
};
use super::types::{
    compute_bounding_sphere, InstanceTransform, InstancedMesh, Object3D, ParticleEmitter,
    ParticleEmitterConfig, PrimitiveCreateInfo, PrimitiveType,
};
use cgmath::Vector3;

impl Renderer3D {
    /// Create a primitive object and return its ID.
    pub fn create_primitive(&mut self, info: PrimitiveCreateInfo) -> u32 {
        let vertices = match info.primitive_type {
            PrimitiveType::Cube => generate_cube_vertices(info.width, info.height, info.depth),
            PrimitiveType::Plane => generate_plane_vertices(info.width, info.depth),
            PrimitiveType::Sphere => generate_sphere_vertices(info.width / 2.0, info.segments),
            PrimitiveType::Cylinder => {
                generate_cylinder_vertices(info.width / 2.0, info.height, info.segments)
            }
        };

        let buffer = match upload_buffer(self.backend.as_mut(), &vertices) {
            Ok(h) => h,
            Err(e) => {
                log::error!("Failed to create primitive buffer: {e}");
                return 0;
            }
        };

        let id = self.next_object_id;
        self.next_object_id += 1;

        let bounds = compute_bounding_sphere(&vertices);
        self.objects.insert(
            id,
            Object3D {
                buffer,
                vertex_count: (vertices.len() / 8) as i32,
                vertices,
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: Vector3::new(0.0, 0.0, 0.0),
                scale: Vector3::new(1.0, 1.0, 1.0),
                texture_id: info.texture_id,
                bounds,
            },
        );

        id
    }

    /// Create an instanced primitive and return its ID.
    pub fn create_instanced_primitive(
        &mut self,
        info: PrimitiveCreateInfo,
        instances: &[InstanceTransform],
    ) -> u32 {
        let vertices = match info.primitive_type {
            PrimitiveType::Cube => generate_cube_vertices(info.width, info.height, info.depth),
            PrimitiveType::Plane => generate_plane_vertices(info.width, info.depth),
            PrimitiveType::Sphere => generate_sphere_vertices(info.width / 2.0, info.segments),
            PrimitiveType::Cylinder => {
                generate_cylinder_vertices(info.width / 2.0, info.height, info.segments)
            }
        };

        let mesh_buffer = match upload_buffer(self.backend.as_mut(), &vertices) {
            Ok(handle) => handle,
            Err(e) => {
                log::error!("Failed to create instanced mesh buffer: {e}");
                return 0;
            }
        };
        let instance_buffer = match upload_instance_buffer(self.backend.as_mut(), instances) {
            Ok(handle) => handle,
            Err(e) => {
                log::error!("Failed to create instanced mesh buffer: {e}");
                self.backend.destroy_buffer(mesh_buffer);
                return 0;
            }
        };

        let id = self.next_instanced_mesh_id;
        self.next_instanced_mesh_id += 1;
        self.instanced_meshes.insert(
            id,
            InstancedMesh {
                mesh_buffer,
                vertex_count: (vertices.len() / 8) as u32,
                instance_buffer,
                instances: instances.to_vec(),
                texture_id: info.texture_id,
            },
        );
        id
    }

    /// Replace the instances stored by an instanced primitive.
    pub fn set_instanced_mesh_instances(
        &mut self,
        id: u32,
        instances: &[InstanceTransform],
    ) -> bool {
        let Some(mesh) = self.instanced_meshes.get_mut(&id) else {
            return false;
        };
        if let Err(e) =
            update_instance_buffer(self.backend.as_mut(), mesh.instance_buffer, instances)
        {
            log::error!("Failed to update instanced mesh buffer: {e}");
            return false;
        }
        mesh.instances = instances.to_vec();
        true
    }

    /// Remove an instanced primitive.
    pub fn remove_instanced_mesh(&mut self, id: u32) -> bool {
        if let Some(mesh) = self.instanced_meshes.remove(&id) {
            self.backend.destroy_buffer(mesh.mesh_buffer);
            self.backend.destroy_buffer(mesh.instance_buffer);
            true
        } else {
            false
        }
    }

    /// Create a particle emitter and return its ID.
    pub fn create_particle_emitter(&mut self, config: ParticleEmitterConfig) -> u32 {
        let instance_buffer = match upload_instance_buffer(self.backend.as_mut(), &[]) {
            Ok(handle) => handle,
            Err(e) => {
                log::error!("Failed to create particle instance buffer: {e}");
                return 0;
            }
        };

        let id = self.next_particle_emitter_id;
        self.next_particle_emitter_id += 1;
        self.particle_emitters.insert(
            id,
            ParticleEmitter {
                position: Vector3::new(0.0, 0.0, 0.0),
                config,
                particles: Vec::new(),
                instance_buffer,
                spawn_accumulator: 0.0,
                spawn_counter: 0,
            },
        );
        id
    }

    /// Set particle emitter origin.
    pub fn set_particle_emitter_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(emitter) = self.particle_emitters.get_mut(&id) {
            emitter.position = Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Remove a particle emitter.
    pub fn remove_particle_emitter(&mut self, id: u32) -> bool {
        if let Some(emitter) = self.particle_emitters.remove(&id) {
            self.backend.destroy_buffer(emitter.instance_buffer);
            true
        } else {
            false
        }
    }
}
