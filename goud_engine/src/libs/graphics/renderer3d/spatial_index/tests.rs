use super::*;
use cgmath::Vector3;

fn collect(idx: &mut SpatialIndex, min: Vector3<f32>, max: Vector3<f32>) -> Vec<u32> {
    let mut out = Vec::new();
    idx.query_aabb(min, max, |id| out.push(id));
    out.sort_unstable();
    out
}

#[test]
fn empty_index_yields_no_candidates() {
    let mut idx = SpatialIndex::new(16.0);
    let got = collect(
        &mut idx,
        Vector3::new(-100.0, -100.0, -100.0),
        Vector3::new(100.0, 100.0, 100.0),
    );
    assert!(got.is_empty());
    assert_eq!(idx.last_query_visited_cells(), 0);
    assert_eq!(idx.last_query_visited_candidates(), 0);
}

#[test]
fn insert_then_query_returns_object() {
    let mut idx = SpatialIndex::new(16.0);
    idx.insert(7, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
    let got = collect(
        &mut idx,
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(8.0, 8.0, 8.0),
    );
    assert_eq!(got, vec![7]);
    assert_eq!(idx.last_query_visited_candidates(), 1);
}

#[test]
fn query_outside_skips_object() {
    let mut idx = SpatialIndex::new(16.0);
    idx.insert(7, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
    let got = collect(
        &mut idx,
        Vector3::new(100.0, 100.0, 100.0),
        Vector3::new(120.0, 120.0, 120.0),
    );
    assert!(got.is_empty());
    assert_eq!(idx.last_query_visited_cells(), 0);
}

#[test]
fn object_spanning_cells_dedupes_to_one_visit() {
    let mut idx = SpatialIndex::new(8.0);
    // AABB straddles four cells in the XY plane: (0,0,0), (1,0,0),
    // (0,1,0), (1,1,0).
    idx.insert(42, Vector3::new(7.0, 7.0, 0.5), Vector3::new(9.0, 9.0, 0.5));
    let got = collect(
        &mut idx,
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(20.0, 20.0, 1.0),
    );
    assert_eq!(got, vec![42]);
    assert_eq!(idx.last_query_visited_candidates(), 1);
    assert!(
        idx.last_query_visited_cells() >= 4,
        "expected at least 4 cells visited, got {}",
        idx.last_query_visited_cells()
    );
}

#[test]
fn update_moves_object_between_cells() {
    let mut idx = SpatialIndex::new(16.0);
    idx.insert(7, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
    idx.update(
        7,
        Vector3::new(100.0, 100.0, 100.0),
        Vector3::new(101.0, 101.0, 101.0),
    );
    // Old location should yield nothing.
    assert!(collect(
        &mut idx,
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(8.0, 8.0, 8.0)
    )
    .is_empty());
    // New location should yield the object.
    assert_eq!(
        collect(
            &mut idx,
            Vector3::new(99.0, 99.0, 99.0),
            Vector3::new(110.0, 110.0, 110.0)
        ),
        vec![7]
    );
}

#[test]
fn remove_drops_object_from_all_cells() {
    let mut idx = SpatialIndex::new(8.0);
    idx.insert(42, Vector3::new(7.0, 7.0, 0.5), Vector3::new(9.0, 9.0, 0.5));
    assert!(idx.remove(42));
    assert_eq!(idx.object_count(), 0);
    assert_eq!(idx.cell_count(), 0);
    let got = collect(
        &mut idx,
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(20.0, 20.0, 1.0),
    );
    assert!(got.is_empty());
}

#[test]
fn remove_missing_id_returns_false() {
    let mut idx = SpatialIndex::new(16.0);
    assert!(!idx.remove(99));
}

#[test]
fn many_objects_in_one_cell_survive_partial_removal() {
    let mut idx = SpatialIndex::new(16.0);
    for id in 0..32 {
        idx.insert(id, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
    }
    // Remove evens.
    for id in (0..32).step_by(2) {
        assert!(idx.remove(id));
    }
    let got = collect(
        &mut idx,
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(8.0, 8.0, 8.0),
    );
    let expected: Vec<u32> = (1..32).step_by(2).collect();
    assert_eq!(got, expected);
}

#[test]
fn reinsert_with_same_aabb_is_idempotent() {
    let mut idx = SpatialIndex::new(16.0);
    idx.insert(5, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
    idx.insert(5, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
    let got = collect(
        &mut idx,
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(8.0, 8.0, 8.0),
    );
    assert_eq!(got, vec![5]);
    assert_eq!(idx.object_count(), 1);
}

#[test]
fn world_aabb_from_sphere_handles_scale() {
    let (min, max) = world_aabb_from_sphere(
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        2.0,
        Vector3::new(3.0, 1.0, 1.0),
    );
    // World center is (10, 1, 0); scaled radius is 2 * max(3,1,1) = 6.
    assert!((min.x - 4.0).abs() < f32::EPSILON);
    assert!((max.x - 16.0).abs() < f32::EPSILON);
    assert!((min.y - (-5.0)).abs() < f32::EPSILON);
    assert!((max.y - 7.0).abs() < f32::EPSILON);
}

#[test]
fn clear_drops_everything_but_keeps_cell_size() {
    let mut idx = SpatialIndex::new(16.0);
    idx.insert(1, Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
    idx.clear();
    assert_eq!(idx.object_count(), 0);
    assert_eq!(idx.cell_count(), 0);
    assert!((idx.cell_size() - 16.0).abs() < f32::EPSILON);
}

#[test]
fn negative_world_coords_route_to_negative_cells() {
    let mut idx = SpatialIndex::new(8.0);
    idx.insert(
        1,
        Vector3::new(-9.0, -9.0, -9.0),
        Vector3::new(-1.0, -1.0, -1.0),
    );
    let got = collect(
        &mut idx,
        Vector3::new(-16.0, -16.0, -16.0),
        Vector3::new(0.0, 0.0, 0.0),
    );
    assert_eq!(got, vec![1]);
}

#[test]
fn stamp_wraparound_preserves_correctness() {
    let mut idx = SpatialIndex::new(16.0);
    idx.insert(3, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
    // Force the stamp counter to wrap.
    idx.next_stamp = u64::MAX - 1;
    for _ in 0..4 {
        let got = collect(
            &mut idx,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(8.0, 8.0, 8.0),
        );
        assert_eq!(got, vec![3]);
    }
}

#[test]
fn cell_size_floor_clamp_protects_against_zero() {
    let idx = SpatialIndex::new(0.0);
    assert!(idx.cell_size() >= 0.5);
}

/// Forces the `query_aabb` occupied-cell-scan branch by populating just a
/// handful of cells in a tiny region but querying a huge AABB whose cell
/// sweep would cost orders of magnitude more lookups than the populated
/// cell count. The query must still report exactly the populated objects
/// and the visited-cells stat must not exceed the populated cell count.
#[test]
fn occupied_cell_scan_used_for_huge_aabb() {
    let mut idx = SpatialIndex::new(8.0);
    // Populate a small cluster: three objects in a few cells at ~origin.
    idx.insert(1, Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
    idx.insert(2, Vector3::new(8.0, 0.0, 0.0), Vector3::new(9.0, 1.0, 1.0));
    idx.insert(3, Vector3::new(0.0, 8.0, 0.0), Vector3::new(1.0, 9.0, 1.0));
    let occupied = idx.cell_count() as u32;

    // Query a far-plane-sized AABB; sweep cost would be ~(20000/8)^3
    // > 10^10, way more than 3 occupied cells, so the implementation
    // must switch to occupied-cell scan.
    let got = collect(
        &mut idx,
        Vector3::new(-10000.0, -10000.0, -10000.0),
        Vector3::new(10000.0, 10000.0, 10000.0),
    );
    assert_eq!(got, vec![1, 2, 3]);
    assert!(
        idx.last_query_visited_cells() <= occupied,
        "occupied-cell scan should not visit more than {} cells, got {}",
        occupied,
        idx.last_query_visited_cells()
    );
}
