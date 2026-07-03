//! Deterministic draw-call / culled-count assertions for the renderer frame
//! benchmark scenes.
//!
//! These run under `cargo test` (the benchmark target itself uses
//! `harness = false`, so its measurement code cannot host `#[test]` functions).
//! They pin the exact counts produced by the scenes in
//! `benches/helpers/scene3d.rs` so a regression that changes how many draw
//! commands the renderer records is caught immediately.

#[path = "../benches/helpers/scene3d.rs"]
mod scene3d;

use scene3d::DEFAULT_MATERIALS;

/// Dynamic scenes route every object through the per-object visible scan, so
/// `draw_calls == visible_objects == object_count` and nothing is culled.
#[test]
fn dynamic_scene_draw_counts_are_pinned() {
    for &n in &[1_000usize, 10_000] {
        let mut renderer = scene3d::dynamic_scene(n);
        renderer.render(None);
        let stats = renderer.stats();

        assert_eq!(
            stats.draw_calls, n as u32,
            "dynamic scene of {n} objects must record {n} draw calls"
        );
        assert_eq!(stats.total_objects, n as u32);
        assert_eq!(stats.visible_objects, n as u32);
        assert_eq!(
            stats.culled_objects, 0,
            "frustum culling is disabled, nothing should be culled"
        );
    }
}

/// Static scenes are batched into the static VBO grouped by material, so the
/// number of draw calls equals the number of distinct materials, the batched
/// objects are skipped by the per-object scan (`visible_objects == 0`), and the
/// total object count is preserved.
#[test]
fn static_scene_batches_into_one_draw_per_material() {
    let n = 10_000usize;
    let mut renderer = scene3d::static_scene(n);
    renderer.render(None);
    let stats = renderer.stats();

    assert_eq!(
        stats.draw_calls, DEFAULT_MATERIALS as u32,
        "static batch must collapse to one draw per material"
    );
    assert_eq!(stats.total_objects, n as u32);
    assert_eq!(
        stats.visible_objects, 0,
        "static-batched objects are skipped by the per-object scan"
    );
}

/// Material sorting reorders the visible draw list but does not merge draws, so
/// the draw-call count is identical whether sorting is on or off.
#[test]
fn material_sorting_does_not_change_draw_count() {
    let n = 5_000usize;

    let mut sorted = scene3d::dynamic_scene_sorting(n, true);
    sorted.render(None);

    let mut unsorted = scene3d::dynamic_scene_sorting(n, false);
    unsorted.render(None);

    assert_eq!(sorted.stats().draw_calls, n as u32);
    assert_eq!(unsorted.stats().draw_calls, n as u32);
    assert_eq!(sorted.stats().draw_calls, unsorted.stats().draw_calls);
}

/// The shadow scene records the GPU shadow pre-pass (Wgsl backend + directional
/// light) and still draws every object in the main pass. Shadow-pass draw
/// commands are depth-only and are not reflected in `Renderer3DStats`; the main
/// pass count is what we can pin through the public API.
#[test]
fn shadow_scene_main_pass_draws_every_object() {
    let n = 1_400usize;
    let mut renderer = scene3d::shadow_scene(n);
    // Two frames: first warms first-frame allocation, second is steady state.
    renderer.render(None);
    renderer.render(None);
    let stats = renderer.stats();

    assert_eq!(
        stats.draw_calls, n as u32,
        "main pass must draw every object with culling disabled"
    );
    assert_eq!(stats.total_objects, n as u32);
    assert_eq!(stats.visible_objects, n as u32);
}
