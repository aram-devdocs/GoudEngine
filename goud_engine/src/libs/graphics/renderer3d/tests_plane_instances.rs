//! Tests for the plane-instance pool path (`instantiate_plane`, #679).
//!
//! These live in their own file so the main `tests.rs` stays under the
//! 500-line cap (`scripts/check-rs-line-limit.sh`).

use super::{PrimitiveCreateInfo, PrimitiveType, Renderer3D};
use crate::libs::graphics::backend::null::NullBackend;

fn make_renderer() -> Renderer3D {
    Renderer3D::new(Box::new(NullBackend::new()), 800, 600).expect("renderer should initialize")
}

fn make_unit_plane(renderer: &mut Renderer3D) -> u32 {
    renderer.create_primitive(PrimitiveCreateInfo {
        primitive_type: PrimitiveType::Plane,
        width: 1.0,
        height: 0.0,
        depth: 1.0,
        segments: 1,
        texture_id: 0,
    })
}

/// Perf regression for #679: 10 000 plane instances created via
/// `instantiate_plane` must render through a single instanced draw call,
/// instead of one per primitive.
#[test]
fn test_instantiate_plane_collapses_many_tiles_to_single_draw_call() {
    let mut renderer = make_renderer();
    let source_plane = make_unit_plane(&mut renderer);
    assert_ne!(source_plane, 0);

    let tile_count: usize = 10_000;
    let mut instance_ids = Vec::with_capacity(tile_count);
    for i in 0..tile_count {
        let id = renderer
            .instantiate_plane(source_plane)
            .expect("instantiate_plane must succeed");
        let x = (i % 100) as f32;
        let z = (i / 100) as f32;
        renderer.set_object_position(id, x, 0.0, z);
        instance_ids.push(id);
    }

    renderer.render(None);
    let stats = renderer.stats();
    // 1 source-plane object draw + 1 instanced draw for the pool, regardless
    // of the 10 000 tiles. The pre-existing per-object pass still draws the
    // source plane itself once (not in any scene); the instancing collapse is
    // the win on top of that.
    assert_eq!(
        stats.instanced_draw_calls, 1,
        "all instances of one source plane should batch into a single instanced draw"
    );
    assert_eq!(
        stats.active_instances, tile_count as u32,
        "pool must report every instance"
    );
    // Without `instantiate_plane`, 10 000 tiles via `create_primitive` would
    // produce 10 000 visible objects in the dynamic pass. Here the source
    // plane is the only object visible to the per-object pass.
    assert_eq!(stats.visible_objects, 1, "no per-tile objects should exist");
}

/// Regression for #679: per-instance transforms must round-trip through the
/// instancing path -- moving an instance moves only that slot.
#[test]
fn test_instantiate_plane_per_instance_transform_updates_pool_only() {
    let mut renderer = make_renderer();
    let source_plane = make_unit_plane(&mut renderer);
    let a = renderer.instantiate_plane(source_plane).unwrap();
    let b = renderer.instantiate_plane(source_plane).unwrap();
    assert_ne!(a, b);

    assert!(renderer.set_object_position(a, 5.0, 0.0, 5.0));
    assert!(renderer.set_object_scale(b, 2.0, 1.0, 2.0));
    assert!(renderer.set_object_rotation(a, 0.0, 90.0, 0.0));

    renderer.render(None);
    assert_eq!(renderer.stats().instanced_draw_calls, 1);
    assert_eq!(renderer.stats().active_instances, 2);

    assert!(renderer.remove_object(a));
    renderer.render(None);
    assert_eq!(renderer.stats().instanced_draw_calls, 1);
    assert_eq!(renderer.stats().active_instances, 1);

    assert!(renderer.remove_object(b));
    renderer.render(None);
    // Removing the last instance tears down the pool -- no instanced draw left.
    assert_eq!(renderer.stats().instanced_draw_calls, 0);
    assert_eq!(renderer.stats().active_instances, 0);
}

/// Regression for #679: two source planes with different textures should
/// produce exactly two instanced draw calls regardless of how many instances
/// each pool holds (matches the "9 materials -> 9 draw calls" expectation).
#[test]
fn test_instantiate_plane_two_source_planes_two_draw_calls() {
    let mut renderer = make_renderer();
    let src_a = make_unit_plane(&mut renderer);
    let src_b = make_unit_plane(&mut renderer);
    for _ in 0..1_000 {
        renderer.instantiate_plane(src_a).unwrap();
    }
    for _ in 0..2_000 {
        renderer.instantiate_plane(src_b).unwrap();
    }

    renderer.render(None);
    let stats = renderer.stats();
    assert_eq!(
        stats.instanced_draw_calls, 2,
        "one instanced draw per source plane"
    );
    assert_eq!(stats.active_instances, 3_000);
}

/// Regression for #679: removing the source plane must cascade-tear-down its
/// pool so the source-plane id cannot route future instances into a stale
/// pool. The instance handles are invalidated too.
#[test]
fn test_remove_source_plane_cascades_pool_teardown() {
    let mut renderer = make_renderer();
    let source_plane = make_unit_plane(&mut renderer);
    let inst_a = renderer.instantiate_plane(source_plane).unwrap();
    let inst_b = renderer.instantiate_plane(source_plane).unwrap();

    renderer.render(None);
    assert_eq!(renderer.stats().instanced_draw_calls, 1);
    assert_eq!(renderer.stats().active_instances, 2);

    assert!(renderer.remove_object(source_plane));
    renderer.render(None);
    assert_eq!(
        renderer.stats().instanced_draw_calls,
        0,
        "pool must be torn down when its source plane is destroyed"
    );

    assert!(
        !renderer.set_object_position(inst_a, 1.0, 2.0, 3.0),
        "instance handle should be invalid after source teardown"
    );
    assert!(
        !renderer.remove_object(inst_b),
        "instance handle should be unknown after source teardown"
    );
}

/// Regression for #679: when a current scene is set, plane-instance pools
/// should only draw if their source plane was added to the scene.
#[test]
fn test_instantiate_plane_respects_current_scene() {
    let mut renderer = make_renderer();
    let source_plane = make_unit_plane(&mut renderer);
    for _ in 0..10 {
        renderer.instantiate_plane(source_plane).unwrap();
    }

    let s = renderer.create_scene("level");
    renderer.set_current_scene(s);
    renderer.render(None);
    assert_eq!(
        renderer.stats().instanced_draw_calls,
        0,
        "source plane is not in the current scene -- pool must not draw"
    );

    assert!(renderer.add_object_to_scene(s, source_plane));
    renderer.render(None);
    assert_eq!(
        renderer.stats().instanced_draw_calls,
        1,
        "source plane in scene -- pool draws once"
    );
    assert_eq!(renderer.stats().active_instances, 10);
}
