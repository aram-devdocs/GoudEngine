use super::core_particles::build_particle_instances;
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

    let center = 16usize;
    assert!(output[center] > 0);
    assert!(output[center] < 255);
}

// ============================================================================
// Material System Tests
// ============================================================================

#[test]
fn test_material_create_and_retrieve() {
    use super::types::{Material3D, MaterialType};
    let mut renderer = make_renderer();
    let mat = Material3D {
        material_type: MaterialType::Pbr,
        ..Default::default()
    };
    let id = renderer.create_material(mat);
    assert_ne!(id, 0);
    let retrieved = renderer.get_material(id).expect("material should exist");
    assert_eq!(retrieved.material_type, MaterialType::Pbr);
}

#[test]
fn test_material_update_and_remove() {
    use super::types::{Material3D, MaterialType};
    let mut renderer = make_renderer();
    let id = renderer.create_material(Material3D::default());
    let updated = Material3D {
        material_type: MaterialType::Unlit,
        shininess: 64.0,
        ..Default::default()
    };
    assert!(renderer.update_material(id, updated));
    assert!(!renderer.update_material(9999, Material3D::default()));
    assert!(renderer.remove_material(id));
    assert!(!renderer.remove_material(id));
}

#[test]
fn test_object_material_binding() {
    use super::types::Material3D;
    let mut renderer = make_renderer();
    let obj_id = renderer.create_primitive(PrimitiveCreateInfo {
        primitive_type: PrimitiveType::Cube,
        width: 1.0,
        height: 1.0,
        depth: 1.0,
        segments: 1,
        texture_id: 0,
    });
    let mat_id = renderer.create_material(Material3D::default());
    assert!(renderer.set_object_material(obj_id, mat_id));
    assert_eq!(renderer.get_object_material(obj_id), Some(mat_id));
    assert!(!renderer.set_object_material(9999, mat_id));
}

// ============================================================================
// PBR Properties Tests
// ============================================================================

#[test]
fn test_pbr_properties_defaults() {
    use super::types::PbrProperties;
    let pbr = PbrProperties::default();
    assert!((pbr.metallic - 0.0).abs() < f32::EPSILON);
    assert!((pbr.roughness - 0.5).abs() < f32::EPSILON);
    assert!((pbr.ao - 1.0).abs() < f32::EPSILON);
}

// ============================================================================
// Skeletal Mesh Tests
// ============================================================================

#[test]
fn test_skinned_mesh_create_and_remove() {
    use super::types::Skeleton3D;
    let mut renderer = make_renderer();
    // 16 floats per vertex, 1 vertex
    let vertices = vec![0.0f32; 16];
    let skeleton = Skeleton3D::new();
    let id = renderer.create_skinned_mesh(vertices, skeleton);
    assert_ne!(id, 0);
    assert!(renderer.remove_skinned_mesh(id));
    assert!(!renderer.remove_skinned_mesh(id));
}

#[test]
fn test_skinned_mesh_transform() {
    use super::types::Skeleton3D;
    let mut renderer = make_renderer();
    let vertices = vec![0.0f32; 16];
    let id = renderer.create_skinned_mesh(vertices, Skeleton3D::new());
    assert!(renderer.set_skinned_mesh_position(id, 1.0, 2.0, 3.0));
    assert!(renderer.set_skinned_mesh_rotation(id, 45.0, 90.0, 0.0));
    assert!(renderer.set_skinned_mesh_scale(id, 2.0, 2.0, 2.0));
    assert!(!renderer.set_skinned_mesh_position(9999, 0.0, 0.0, 0.0));
}

#[test]
fn test_skeleton_bone_count() {
    use super::types::{Bone3D, Skeleton3D};
    let mut skel = Skeleton3D::new();
    assert_eq!(skel.bone_count(), 0);
    skel.bones.push(Bone3D {
        name: "root".to_string(),
        parent_index: -1,
        inverse_bind_matrix: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    });
    assert_eq!(skel.bone_count(), 1);
}

// ============================================================================
// Post-Processing Pipeline Tests
// ============================================================================

#[test]
fn test_postprocess_pipeline_add_and_remove() {
    use super::types::{BloomPass, PostProcessPipeline};
    let mut pipeline = PostProcessPipeline::new();
    assert_eq!(pipeline.pass_count(), 0);
    pipeline.add_pass(Box::new(BloomPass::default()));
    assert_eq!(pipeline.pass_count(), 1);
    assert!(pipeline.remove_pass(0));
    assert_eq!(pipeline.pass_count(), 0);
    assert!(!pipeline.remove_pass(0));
}

#[test]
fn test_postprocess_pipeline_processes_data() {
    use super::types::{ColorGradePass, PostProcessPipeline};
    let mut pipeline = PostProcessPipeline::new();
    pipeline.add_pass(Box::new(ColorGradePass {
        exposure: 2.0,
        contrast: 1.0,
        saturation: 1.0,
        enabled: true,
    }));
    // 2x2 image, gray pixels
    let mut data = vec![128u8; 16];
    pipeline.run(2, 2, &mut data);
    // Exposure of 2.0 should brighten the pixels
    assert!(data[0] > 128);
}

// ============================================================================
// Scene Management Tests
// ============================================================================

#[test]
fn test_scene_create_and_destroy() {
    let mut renderer = make_renderer();
    let s1 = renderer.create_scene("level1");
    let s2 = renderer.create_scene("level2");
    assert_ne!(s1, s2);
    assert!(renderer.destroy_scene(s1));
    assert!(!renderer.destroy_scene(s1)); // already destroyed
    assert!(renderer.destroy_scene(s2));
}

#[test]
fn test_scene_set_and_get_current() {
    let mut renderer = make_renderer();
    assert_eq!(renderer.get_current_scene(), None);
    let s = renderer.create_scene("main");
    assert!(renderer.set_current_scene(s));
    assert_eq!(renderer.get_current_scene(), Some(s));
    renderer.clear_current_scene();
    assert_eq!(renderer.get_current_scene(), None);
}

#[test]
fn test_scene_destroy_clears_current() {
    let mut renderer = make_renderer();
    let s = renderer.create_scene("temp");
    renderer.set_current_scene(s);
    renderer.destroy_scene(s);
    assert_eq!(renderer.get_current_scene(), None);
}

#[test]
fn test_scene_add_object_validates_existence() {
    let mut renderer = make_renderer();
    let s = renderer.create_scene("s");
    // Non-existent object should fail.
    assert!(!renderer.add_object_to_scene(s, 9999));
    // Create an object and add it.
    let obj = renderer.create_primitive(PrimitiveCreateInfo {
        primitive_type: PrimitiveType::Cube,
        width: 1.0,
        height: 1.0,
        depth: 1.0,
        segments: 1,
        texture_id: 0,
    });
    assert!(renderer.add_object_to_scene(s, obj));
    assert!(renderer.remove_object_from_scene(s, obj));
    assert!(!renderer.remove_object_from_scene(s, obj));
}

#[test]
fn test_scene_add_light_validates_existence() {
    use super::types::Light;
    let mut renderer = make_renderer();
    let s = renderer.create_scene("s");
    assert!(!renderer.add_light_to_scene(s, 9999));
    let l = renderer.add_light(Light::default());
    assert!(renderer.add_light_to_scene(s, l));
    assert!(renderer.remove_light_from_scene(s, l));
}

#[test]
fn test_scene_filtering_limits_rendered_objects() {
    let mut renderer = make_renderer();
    // Create two objects.
    let a = renderer.create_primitive(PrimitiveCreateInfo {
        primitive_type: PrimitiveType::Cube,
        width: 1.0,
        height: 1.0,
        depth: 1.0,
        segments: 1,
        texture_id: 0,
    });
    let _b = renderer.create_primitive(PrimitiveCreateInfo {
        primitive_type: PrimitiveType::Cube,
        width: 1.0,
        height: 1.0,
        depth: 1.0,
        segments: 1,
        texture_id: 0,
    });

    // No scene: both objects rendered (2 draw calls).
    renderer.render(None);
    assert_eq!(renderer.stats().draw_calls, 2);

    // Create scene with only object `a`.
    let s = renderer.create_scene("filtered");
    renderer.add_object_to_scene(s, a);
    renderer.set_current_scene(s);
    renderer.render(None);
    assert_eq!(renderer.stats().draw_calls, 1);

    // Clear scene: both objects rendered again.
    renderer.clear_current_scene();
    renderer.render(None);
    assert_eq!(renderer.stats().draw_calls, 2);
}

/// Regression test for #630: primitives marked static must still render via
/// the static batch path instead of disappearing.
#[test]
fn test_static_primitive_renders_via_batch() {
    let mut renderer = make_renderer();
    let cube = renderer.create_primitive(PrimitiveCreateInfo {
        primitive_type: PrimitiveType::Cube,
        width: 1.0,
        height: 1.0,
        depth: 1.0,
        segments: 1,
        texture_id: 0,
    });
    assert_ne!(cube, 0);

    // Dynamic path: one draw call, one visible object.
    renderer.render(None);
    assert_eq!(renderer.stats().draw_calls, 1);
    assert_eq!(renderer.stats().visible_objects, 1);

    // Mark static: should render via static batch, not dynamic pass.
    assert!(renderer.set_object_static(cube, true));
    renderer.render(None);
    let stats = renderer.stats();
    assert!(
        stats.draw_calls >= 1,
        "static object must produce at least one draw call"
    );
    assert_eq!(
        stats.visible_objects, 0,
        "static object should not appear in dynamic pass"
    );
}
