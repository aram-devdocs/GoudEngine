use super::core::build_particle_instances;
use super::postprocess::apply_fxaa_like_filter;
use super::shadow::{build_shadow_map_from_meshes, sample_shadow_factor, ShadowMesh};
use super::{
    types::{Particle, ParticleEmitter},
    InstanceTransform, ParticleEmitterConfig, PrimitiveCreateInfo, PrimitiveType, Renderer3D,
};
use crate::libs::graphics::backend::null::NullBackend;
use cgmath::{Matrix4, Vector3, Vector4};

fn make_renderer() -> Renderer3D {
    Renderer3D::new(Box::new(NullBackend::new()), 800, 600).expect("renderer should initialize")
}

#[test]
fn test_instanced_mesh_keeps_single_draw_call_for_many_instances() {
    let mut renderer = make_renderer();
    let single = vec![InstanceTransform::default()];
    let mesh_id = renderer.create_instanced_primitive(
        PrimitiveCreateInfo {
            primitive_type: PrimitiveType::Cube,
            width: 1.0,
            height: 1.0,
            depth: 1.0,
            segments: 1,
            texture_id: 0,
        },
        &single,
    );
    assert_ne!(mesh_id, 0);

    renderer.render(None);
    let stats = renderer.stats();
    assert_eq!(stats.instanced_draw_calls, 1);
    assert_eq!(stats.active_instances, 1);

    let many = vec![InstanceTransform::default(); 1000];
    assert!(renderer.set_instanced_mesh_instances(mesh_id, &many));
    renderer.render(None);
    let stats = renderer.stats();
    assert_eq!(stats.instanced_draw_calls, 1);
    assert_eq!(stats.active_instances, 1000);
}

#[test]
fn test_particle_emitter_updates_and_renders_as_instanced_draw() {
    let mut renderer = make_renderer();
    let emitter_id = renderer.create_particle_emitter(ParticleEmitterConfig {
        emission_rate: 20.0,
        max_particles: 128,
        lifetime: 2.0,
        ..Default::default()
    });
    assert_ne!(emitter_id, 0);

    renderer.update(0.5);
    renderer.render(None);
    let stats = renderer.stats();
    assert_eq!(stats.particle_draw_calls, 1);
    assert_eq!(stats.active_particles, 10);

    renderer.update(0.5);
    renderer.render(None);
    let stats = renderer.stats();
    assert_eq!(stats.particle_draw_calls, 1);
    assert_eq!(stats.active_particles, 20);
}

#[test]
fn test_particle_instance_interpolation_applies_color_and_size_over_lifetime() {
    let emitter = ParticleEmitter {
        position: Vector3::new(0.0, 0.0, 0.0),
        config: ParticleEmitterConfig {
            start_color: Vector4::new(1.0, 0.0, 0.0, 1.0),
            end_color: Vector4::new(0.0, 0.0, 1.0, 0.25),
            start_size: 2.0,
            end_size: 0.5,
            ..Default::default()
        },
        particles: vec![Particle {
            position: Vector3::new(1.0, 2.0, 3.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            age: 0.5,
            lifetime: 1.0,
        }],
        instance_buffer: crate::libs::graphics::backend::BufferHandle::INVALID,
        spawn_accumulator: 0.0,
        spawn_counter: 0,
    };

    let instances = build_particle_instances(&emitter);
    assert_eq!(instances.len(), 1);
    let instance = &instances[0];
    assert_eq!(instance.position, Vector3::new(1.0, 2.0, 3.0));
    assert!((instance.scale.x - 1.25).abs() < 0.0001);
    assert!((instance.color.x - 0.5).abs() < 0.0001);
    assert!((instance.color.z - 0.5).abs() < 0.0001);
    assert!((instance.color.w - 0.625).abs() < 0.0001);
}

#[test]
fn test_shadow_map_casts_shadow_from_cube_onto_plane() {
    let cube = super::mesh::generate_cube_vertices(1.0, 1.0, 1.0);
    let plane = super::mesh::generate_plane_vertices(6.0, 6.0);
    let meshes = [
        ShadowMesh {
            vertices: &cube,
            model: Matrix4::from_translation(Vector3::new(0.0, 0.5, 0.0)),
        },
        ShadowMesh {
            vertices: &plane,
            model: Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0)),
        },
    ];

    let shadow_map = build_shadow_map_from_meshes(&meshes, Vector3::new(-0.5, -1.0, -0.25), 128);
    let under_cube = sample_shadow_factor(&shadow_map, Vector3::new(0.0, 0.0, 0.0), 0.01);
    let outside_shadow = sample_shadow_factor(&shadow_map, Vector3::new(2.5, 0.0, 2.5), 0.01);

    assert!(under_cube > 0.5);
    assert!(outside_shadow < 0.5);
}

#[test]
fn test_fxaa_filter_softens_high_contrast_edge() {
    let input = vec![
        0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    ];
    let output = apply_fxaa_like_filter(3, 3, &input);

    let center = ((1 * 3 + 1) * 4) as usize;
    assert!(output[center] > 0);
    assert!(output[center] < 255);
}
