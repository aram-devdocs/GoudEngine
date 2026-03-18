use super::core::Renderer3D;
use super::mesh::update_instance_buffer;
use super::types::{InstanceTransform, Particle, ParticleEmitter};
use cgmath::{Vector3, Vector4};

impl Renderer3D {
    /// Advance transient renderer systems that depend on frame time.
    pub fn update(&mut self, delta_time: f32) {
        for emitter in self.particle_emitters.values_mut() {
            let dt = delta_time.max(0.0);
            emitter.spawn_accumulator += emitter.config.emission_rate * dt;
            let spawn_count = emitter.spawn_accumulator.floor() as usize;
            emitter.spawn_accumulator -= spawn_count as f32;

            for _ in 0..spawn_count {
                if emitter.particles.len() >= emitter.config.max_particles {
                    break;
                }
                emitter.particles.push(Particle {
                    position: emitter.position,
                    velocity: sample_velocity(
                        emitter.config.velocity_min,
                        emitter.config.velocity_max,
                        emitter.spawn_counter,
                    ),
                    age: 0.0,
                    lifetime: emitter.config.lifetime.max(0.001),
                });
                emitter.spawn_counter = emitter.spawn_counter.wrapping_add(1);
            }

            for particle in &mut emitter.particles {
                particle.age += dt;
                particle.position += particle.velocity * dt;
            }
            emitter
                .particles
                .retain(|particle| particle.age < particle.lifetime);

            let instances = build_particle_instances(emitter);

            if let Err(e) =
                update_instance_buffer(self.backend.as_mut(), emitter.instance_buffer, &instances)
            {
                log::error!("Failed to update particle emitter buffer: {e}");
            }
        }
    }
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn sample_velocity(min: Vector3<f32>, max: Vector3<f32>, counter: u32) -> Vector3<f32> {
    let seed = counter as f32 * 0.618_034;
    let fx = seed.fract();
    let fy = (seed * 1.37).fract();
    let fz = (seed * 1.91).fract();
    Vector3::new(
        lerp(min.x, max.x, fx),
        lerp(min.y, max.y, fy),
        lerp(min.z, max.z, fz),
    )
}

pub(super) fn build_particle_instances(emitter: &ParticleEmitter) -> Vec<InstanceTransform> {
    emitter
        .particles
        .iter()
        .map(|particle| {
            let t = (particle.age / particle.lifetime).clamp(0.0, 1.0);
            let size = lerp(emitter.config.start_size, emitter.config.end_size, t);
            InstanceTransform {
                position: particle.position,
                rotation: Vector3::new(0.0, 0.0, 0.0),
                scale: Vector3::new(size, size, 1.0),
                color: Vector4::new(
                    lerp(emitter.config.start_color.x, emitter.config.end_color.x, t),
                    lerp(emitter.config.start_color.y, emitter.config.end_color.y, t),
                    lerp(emitter.config.start_color.z, emitter.config.end_color.z, t),
                    lerp(emitter.config.start_color.w, emitter.config.end_color.w, t),
                ),
            }
        })
        .collect()
}
