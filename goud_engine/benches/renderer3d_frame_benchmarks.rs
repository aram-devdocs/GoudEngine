//! Renderer3D Frame Benchmark Suite
//!
//! Drives the real [`Renderer3D::render`] over a headless Wgsl [`NullBackend`]
//! so a later wave of renderer optimizations shows up as measurable deltas in
//! the per-frame CPU pipeline (object scan, frustum handling, material sort,
//! draw-command recording, and the GPU shadow pre-pass recording).
//!
//! Run with: `cargo bench --bench renderer3d_frame_benchmarks`
//!
//! ## Groups
//!
//! - `frame_scan/{static,dynamic}_{10k,30k}` — steady-state frame cost with the
//!   scene unchanged between frames, plus `dynamic_moving_10k` where every
//!   object's transform changes each frame.
//! - `material_sort/{on,off}_30k` — cost of sorting the visible draw list by
//!   material vs. leaving it unsorted.
//! - `cull_scaling/visible_5k_total_*` — fixed 5k visible objects with total
//!   scene size growing from 10k to 100k to keep #678 measurable.
//! - `primitive_draw_calls/{plane,cube}_10k` — per-frame draw-record cost for
//!   the legacy `CreatePlane` / `CreateCube` primitive paths.
//! - `shadow_record/casters_{1400,5k}` — cost of recording the GPU shadow
//!   pre-pass (requires the Wgsl NullBackend + a directional light).
//! The opt-in native-wgpu shadow bench lives in
//! `renderer3d_real_wgpu_shadow_benchmarks.rs` so the default suite stays
//! CPU-safe in CI and headless environments.
//!
//! The scenes are deterministic; the companion assertions that pin the exact
//! draw-call / culled counts live in
//! `goud_engine/tests/renderer3d_frame_counts.rs`.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use goud_engine::libs::graphics::renderer3d::PrimitiveType;

#[path = "helpers/scene3d.rs"]
mod scene3d;

const CULL_VISIBLE_COUNT: usize = 5_000;
const CULL_SCALING_SIZES: [usize; 3] = [10_000, 30_000, 100_000];
const FRAME_SCAN_SIZES: [usize; 2] = [10_000, 30_000];

// ================================================================================================
// Group 1: Frame scan (static vs dynamic, unchanged between frames)
// ================================================================================================

fn bench_frame_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_scan");

    for &n in &FRAME_SCAN_SIZES {
        group.throughput(Throughput::Elements(n as u64));

        // Static objects: batched into the static VBO, so the per-frame cost is
        // dominated by the scan that skips already-batched objects plus the
        // per-group static-batch draws.
        group.bench_function(format!("static_{}", label(n)), |b| {
            let mut renderer = scene3d::static_scene(n);
            // Warm the static-batch rebuild so steady-state frames are measured.
            renderer.render(None);
            b.iter(|| renderer.render(black_box(None)));
        });

        // Dynamic objects: every object flows through the visible scan + sort +
        // per-object draw-record loop.
        group.bench_function(format!("dynamic_{}", label(n)), |b| {
            let mut renderer = scene3d::dynamic_scene(n);
            b.iter(|| renderer.render(black_box(None)));
        });
    }

    // Moving variant: all 10k objects change transform each frame.
    group.throughput(Throughput::Elements(10_000));
    group.bench_function("dynamic_moving_10k", |b| {
        let mut renderer = scene3d::dynamic_scene(10_000);
        let mut frame: u64 = 0;
        b.iter(|| {
            scene3d::advance_dynamic_scene(&mut renderer, 10_000, frame);
            frame = frame.wrapping_add(1);
            renderer.render(black_box(None));
        });
    });

    group.finish();
}

// ================================================================================================
// Group 2: Material sorting on vs off
// ================================================================================================

fn bench_material_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_sort");
    const N: usize = 30_000;
    group.throughput(Throughput::Elements(N as u64));

    for (label, sorting) in [("on_30k", true), ("off_30k", false)] {
        group.bench_function(label, |b| {
            let mut renderer = scene3d::dynamic_scene_sorting(N, sorting);
            b.iter(|| renderer.render(black_box(None)));
        });
    }

    group.finish();
}

// ================================================================================================
// Group 3: Frustum-culling scaling (fixed visible set, growing total)
// ================================================================================================

fn bench_cull_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("cull_scaling");

    for &total in &CULL_SCALING_SIZES {
        group.throughput(Throughput::Elements(total as u64));
        group.bench_function(
            format!(
                "visible_{}k_total_{}",
                CULL_VISIBLE_COUNT / 1000,
                label(total)
            ),
            |b| {
                let mut renderer = scene3d::cull_scaling_scene(total, CULL_VISIBLE_COUNT);
                b.iter(|| renderer.render(black_box(None)));
            },
        );
    }

    group.finish();
}

// ================================================================================================
// Group 4: Legacy primitive draw-call scaling
// ================================================================================================

fn bench_primitive_draw_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitive_draw_calls");
    const N: usize = 10_000;
    group.throughput(Throughput::Elements(N as u64));

    for (label, primitive_type) in [
        ("plane_10k", PrimitiveType::Plane),
        ("cube_10k", PrimitiveType::Cube),
    ] {
        group.bench_function(label, |b| {
            let mut renderer = scene3d::dynamic_primitive_scene(N, primitive_type);
            b.iter(|| renderer.render(black_box(None)));
        });
    }

    group.finish();
}

// ================================================================================================
// Group 5: Shadow pre-pass recording
// ================================================================================================

fn bench_shadow_record(c: &mut Criterion) {
    let mut group = c.benchmark_group("shadow_record");

    for (label, n) in [("casters_1400", 1_400usize), ("casters_5k", 5_000)] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(label, |b| {
            let mut renderer = scene3d::shadow_scene(n);
            // Warm any first-frame allocation so steady-state frames are measured.
            renderer.render(None);
            b.iter(|| renderer.render(black_box(None)));
        });
    }

    group.finish();
}

/// Compact size label used in bench names: 10000 -> "10k", 30000 -> "30k".
fn label(n: usize) -> String {
    if n.is_multiple_of(1000) {
        format!("{}k", n / 1000)
    } else {
        n.to_string()
    }
}

criterion_group!(
    renderer3d_frame_benches,
    bench_frame_scan,
    bench_material_sort,
    bench_cull_scaling,
    bench_primitive_draw_calls,
    bench_shadow_record,
);
criterion_main!(renderer3d_frame_benches);
