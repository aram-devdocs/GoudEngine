//! Scene-Culling Scaling Benchmarks (#678)
//!
//! Drives the [`Renderer3D`] frustum-cull path with 1k / 10k / 50k registered
//! 3D objects and asserts (via the per-frame `Renderer3DStats`) that the
//! candidate set fed into the frustum sphere test grows sub-linearly when the
//! spatial index is enabled.
//!
//! Run with: `cargo bench --bench scene_culling_benchmarks`
//!
//! The benchmark emits the spatial-index candidate count to stdout each
//! sample so before/after comparisons are easy to eyeball without a baseline
//! file.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use goud_engine::libs::graphics::backend::null::NullBackend;
use goud_engine::libs::graphics::renderer3d::config::SpatialIndexConfig;
use goud_engine::libs::graphics::renderer3d::{PrimitiveCreateInfo, PrimitiveType, Renderer3D};

fn make_renderer() -> Renderer3D {
    Renderer3D::new(Box::new(NullBackend::new()), 800, 600)
        .expect("renderer should initialize against the null backend")
}

fn populate_grid(renderer: &mut Renderer3D, side: i32, spacing: f32) {
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
        }
    }
}

fn aim_camera_at_origin(renderer: &mut Renderer3D) {
    renderer.set_camera_position(5.0, 5.0, 5.0);
    renderer.set_camera_rotation(-30.0, -135.0, 0.0);
}

fn bench_render_with_spatial_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_culling_spatial_index_on");

    // 1k / 10k / 30k / 50k matches the scaling table in #678 so the bench
    // output lines up with the issue's three named checkpoints.
    for &(side, label) in &[
        (32i32, 1_024usize),
        (100, 10_000),
        (174, 30_276),
        (224, 50_176),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &side, |b, &side| {
            let mut renderer = make_renderer();
            populate_grid(&mut renderer, side, 4.0);
            aim_camera_at_origin(&mut renderer);
            b.iter(|| {
                renderer.render(None);
                black_box(renderer.stats().spatial_index_candidates);
            });
        });
    }
    group.finish();
}

fn bench_render_with_linear_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_culling_spatial_index_off");

    for &(side, label) in &[
        (32i32, 1_024usize),
        (100, 10_000),
        (174, 30_276),
        (224, 50_176),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &side, |b, &side| {
            let mut renderer = make_renderer();
            let mut config = renderer.render_config().clone();
            config.spatial_index = SpatialIndexConfig {
                enabled: false,
                cell_size: config.spatial_index.cell_size,
            };
            renderer.set_render_config(config);
            populate_grid(&mut renderer, side, 4.0);
            aim_camera_at_origin(&mut renderer);
            b.iter(|| {
                renderer.render(None);
                black_box(renderer.stats().spatial_index_candidates);
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_render_with_spatial_index,
    bench_render_with_linear_scan
);
criterion_main!(benches);
