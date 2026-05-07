use super::*;
use crate::libs::graphics::renderer3d::config::SpatialIndexConfig;

fn populate_grid(renderer: &mut Renderer3D, side: i32, spacing: f32) -> Vec<u32> {
    let mut ids = Vec::with_capacity((side as usize).pow(2));
    for x in 0..side {
        for z in 0..side {
            let id = renderer.create_primitive(PrimitiveCreateInfo {
                primitive_type: PrimitiveType::Cube,
                width: 1.0,
                height: 1.0,
                depth: 1.0,
                segments: 1,
                texture_id: 0,
            });
            assert_ne!(id, 0);
            renderer.set_object_position(id, x as f32 * spacing, 0.0, z as f32 * spacing);
            ids.push(id);
        }
    }
    ids
}

fn aim_camera_at_origin(renderer: &mut Renderer3D) {
    // Camera at (5, 5, 5) looking at origin: a small visible patch covers
    // only the first few cells of the populated grid.
    renderer.set_camera_position(5.0, 5.0, 5.0);
    renderer.set_camera_rotation(-30.0, -135.0, 0.0);
}

fn collect_candidate_ratio(renderer: &mut Renderer3D) -> (u32, u32) {
    renderer.render(None);
    let stats = renderer.stats();
    (stats.spatial_index_candidates, stats.total_objects)
}

#[test]
fn spatial_index_shrinks_candidate_set_at_1k_objects() {
    let mut renderer = make_renderer();
    let _ids = populate_grid(&mut renderer, 32, 4.0); // 1024 objects.
    aim_camera_at_origin(&mut renderer);
    let (candidates, total) = collect_candidate_ratio(&mut renderer);
    assert_eq!(total, 1024);
    assert!(
        candidates < total / 2,
        "expected spatial index to halve the candidate set at 1k objects, \
         got {candidates}/{total}"
    );
}

#[test]
fn spatial_index_stays_sublinear_at_10k_objects() {
    let mut renderer = make_renderer();
    let _ids = populate_grid(&mut renderer, 100, 4.0); // 10_000 objects.
    aim_camera_at_origin(&mut renderer);
    let (candidates, total) = collect_candidate_ratio(&mut renderer);
    assert_eq!(total, 10_000);
    // 10k objects across a 400x400 patch with the camera looking at a
    // narrow region near the origin: <10% of the scene should make it
    // through the spatial-index pre-filter.
    assert!(
        candidates * 10 < total,
        "expected <10% candidate ratio at 10k, got {candidates}/{total}"
    );
    assert!(
        renderer.stats().spatial_index_cells_visited > 0,
        "spatial index must report at least one visited cell"
    );
}

#[test]
fn spatial_index_stays_sublinear_at_30k_objects() {
    let mut renderer = make_renderer();
    let _ids = populate_grid(&mut renderer, 174, 4.0); // 30_276 objects.
    aim_camera_at_origin(&mut renderer);
    let (candidates, total) = collect_candidate_ratio(&mut renderer);
    assert_eq!(total, 30_276);
    assert!(
        candidates * 10 < total,
        "expected <10% candidate ratio at 30k, got {candidates}/{total}"
    );
}

#[test]
fn spatial_index_stays_sublinear_at_50k_objects() {
    let mut renderer = make_renderer();
    let _ids = populate_grid(&mut renderer, 224, 4.0); // 50_176 objects.
    aim_camera_at_origin(&mut renderer);
    let (candidates, total) = collect_candidate_ratio(&mut renderer);
    assert_eq!(total, 50_176);
    // The candidate set must not scale linearly with the total. We
    // require the pre-filter to deliver <5% of the registry.
    assert!(
        candidates * 20 < total,
        "expected <5% candidate ratio at 50k, got {candidates}/{total}"
    );
}

#[test]
fn spatial_index_disabled_falls_back_to_full_scan() {
    let mut renderer = make_renderer();
    let _ids = populate_grid(&mut renderer, 32, 4.0); // 1024 objects.
    aim_camera_at_origin(&mut renderer);

    let mut config = renderer.render_config().clone();
    config.spatial_index = SpatialIndexConfig {
        enabled: false,
        cell_size: config.spatial_index.cell_size,
    };
    renderer.set_render_config(config);

    renderer.render(None);
    let stats = renderer.stats();
    // When the index is disabled, the pre-filter is a no-op so every
    // object is a candidate.
    assert_eq!(stats.total_objects, 1024);
    assert_eq!(stats.spatial_index_candidates, 1024);
    assert_eq!(stats.spatial_index_cells_visited, 0);
}

#[test]
fn spatial_index_visible_count_matches_linear_scan() {
    // Parity check: enabling the spatial index must not change which
    // objects survive the frustum sphere test compared to the legacy
    // linear scan — the index is a *pre-filter*, not a replacement.
    let make_scene = || {
        let mut r = make_renderer();
        populate_grid(&mut r, 24, 4.0); // 576 objects across 92x92 units.
        aim_camera_at_origin(&mut r);
        r
    };

    let mut spatial = make_scene();
    spatial.render(None);
    let spatial_visible = spatial.stats().visible_objects;

    let mut linear = make_scene();
    let mut config = linear.render_config().clone();
    config.spatial_index.enabled = false;
    linear.set_render_config(config);
    linear.render(None);
    let linear_visible = linear.stats().visible_objects;

    assert_eq!(
        spatial_visible, linear_visible,
        "spatial-indexed visible count {spatial_visible} != linear-scan visible count {linear_visible}"
    );
}

#[test]
fn spatial_index_tracks_object_movement() {
    // Move an object out of view, render, move it back, render. The
    // spatial index must keep its cell membership in sync so a moved
    // object that re-enters the frustum is rendered again.
    let mut renderer = make_renderer();
    let id = renderer.create_primitive(PrimitiveCreateInfo {
        primitive_type: PrimitiveType::Cube,
        width: 1.0,
        height: 1.0,
        depth: 1.0,
        segments: 1,
        texture_id: 0,
    });
    // Camera at (0,0,-5) with yaw=0 looks down +Z, putting the origin in
    // front of the camera.
    renderer.set_camera_position(0.0, 0.0, -5.0);
    renderer.set_camera_rotation(0.0, 0.0, 0.0);

    renderer.render(None);
    assert_eq!(
        renderer.stats().visible_objects,
        1,
        "object at origin should be visible from camera at (0,0,-5) facing +Z"
    );

    // Move object far behind the camera.
    assert!(renderer.set_object_position(id, 0.0, 0.0, -500.0));
    renderer.render(None);
    assert_eq!(
        renderer.stats().visible_objects,
        0,
        "object behind camera should be culled after move"
    );

    // Move back into view.
    assert!(renderer.set_object_position(id, 0.0, 0.0, 0.0));
    renderer.render(None);
    assert_eq!(
        renderer.stats().visible_objects,
        1,
        "object should be visible again after move-back"
    );
}

#[test]
fn spatial_index_tracks_object_scale_growth() {
    // Place a small object far enough off to one side that its tight
    // bounding sphere never reaches the frustum, then scale it up so the
    // sphere swells into view. The spatial index must refresh on the
    // scale change so the now-overlapping object becomes a candidate.
    let mut renderer = make_renderer();
    let id = renderer.create_primitive(PrimitiveCreateInfo {
        primitive_type: PrimitiveType::Cube,
        width: 0.5,
        height: 0.5,
        depth: 0.5,
        segments: 1,
        texture_id: 0,
    });
    // Camera at origin looking down +Z; object at (0, 0, 80) is in front.
    renderer.set_camera_position(0.0, 0.0, -5.0);
    renderer.set_camera_rotation(0.0, 0.0, 0.0);
    // Park the object far out to the side at +X so a unit-scale bounding
    // sphere does not intersect the camera's central frustum.
    assert!(renderer.set_object_position(id, 200.0, 0.0, 50.0));

    renderer.render(None);
    assert_eq!(
        renderer.stats().visible_objects,
        0,
        "tiny far-side object should be culled at unit scale"
    );

    // Scale it up so its world-space sphere swells through the frustum.
    assert!(renderer.set_object_scale(id, 800.0, 800.0, 800.0));
    renderer.render(None);
    assert_eq!(
        renderer.stats().visible_objects,
        1,
        "scaled-up object should be picked up by the spatial index after \
         the scale change refreshes its cell membership"
    );
}
