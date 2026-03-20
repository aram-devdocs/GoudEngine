//! Render Benchmark Suite
//!
//! Establishes performance baselines for the rendering pipeline before
//! Phase 0 optimizations. Benchmarks the full CPU pipeline: gather, sort,
//! vertex generation, and batch metric extraction.
//!
//! Run with: `cargo bench --bench render_benchmarks`
//!
//! Generate baseline JSON: `RENDER_BASELINE=1 cargo bench --bench render_benchmarks`
//!
//! ## Scenarios
//!
//! - empty: 0 entities
//! - light: 100 entities
//! - moderate: 1,000 entities
//! - target: 10,000 entities
//!
//! ## Throne Context
//!
//! 373 entities at 13.51ms render, 155-345 draw calls.
//! Target: <3ms, <100 draw calls.

use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};
use goud_engine::assets::loaders::TextureAsset;
use goud_engine::assets::AssetHandle;
use goud_engine::assets::AssetServer;
use goud_engine::core::math::Vec2;
use goud_engine::ecs::components::{Sprite, Transform2D};
use goud_engine::ecs::World;
use goud_engine::rendering::sprite_batch::{SpriteBatch, SpriteBatchConfig, SpriteInstance};
use serde::Serialize;
use std::time::Instant;

#[path = "helpers/null_backend.rs"]
mod null_backend;
use null_backend::NullBackend;

// ================================================================================================
// Render Metrics
// ================================================================================================

/// Captured metrics from a single render frame for baseline comparison.
#[derive(Debug, Clone, Serialize)]
struct RenderFrameMetrics {
    entity_count: usize,
    sprite_count: usize,
    draw_calls: usize,
    batch_count: usize,
    batch_ratio: f32,
    culled_count: usize,
    vertex_count: usize,
    vertex_buffer_bytes: usize,
}

// ================================================================================================
// Helpers
// ================================================================================================

const TEXTURE_COUNT: usize = 8;
const SCENARIO_SIZES: [usize; 4] = [0, 100, 1_000, 10_000];
const SCALING_SIZES: [usize; 3] = [500, 2_000, 5_000];

fn create_render_world(n: usize) -> World {
    let mut world = World::new();
    for i in 0..n {
        let entity = world.spawn_empty();
        let texture: AssetHandle<TextureAsset> = AssetHandle::new((i % TEXTURE_COUNT) as u32, 1);
        world.insert(entity, Sprite::new(texture));
        world.insert(
            entity,
            Transform2D::from_position(Vec2::new(i as f32 * 10.0, (i % 100) as f32)),
        );
    }
    world
}

fn create_render_batch() -> SpriteBatch<NullBackend> {
    SpriteBatch::new(NullBackend::new(), SpriteBatchConfig::default()).unwrap()
}

/// Count draw calls by counting texture transitions in a sorted sprite list.
fn count_draw_calls(sprites: &[SpriteInstance]) -> usize {
    if sprites.is_empty() {
        return 0;
    }
    let mut calls = 1;
    for window in sprites.windows(2) {
        if window[0].texture != window[1].texture {
            calls += 1;
        }
    }
    calls
}

/// Run a full CPU frame and extract metrics.
fn capture_frame_metrics(n: usize) -> RenderFrameMetrics {
    let world = create_render_world(n);
    let mut batch = create_render_batch();
    let mut asset_server = AssetServer::new();

    batch.begin();
    batch.gather_sprites(&world, &mut asset_server).unwrap();
    batch.sort_sprites();

    let draw_calls = count_draw_calls(&batch.sprites);
    let texture_size = Vec2::new(64.0, 64.0);
    let sprites = std::mem::take(&mut batch.sprites);
    for sprite in &sprites {
        batch
            .generate_sprite_vertices(sprite, texture_size)
            .unwrap();
    }
    batch.sprites = sprites;

    let vertex_count = batch.vertices.len();
    let vertex_buffer_bytes =
        vertex_count * std::mem::size_of::<goud_engine::rendering::sprite_batch::SpriteVertex>();

    RenderFrameMetrics {
        entity_count: n,
        sprite_count: batch.sprite_count(),
        draw_calls,
        batch_count: draw_calls,
        batch_ratio: if draw_calls > 0 {
            batch.sprite_count() as f32 / draw_calls as f32
        } else {
            0.0
        },
        culled_count: batch.culled_count(),
        vertex_count,
        vertex_buffer_bytes,
    }
}

// ================================================================================================
// Benchmark Group 1: Full CPU Frame Pipeline
// ================================================================================================

fn bench_render_full_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_full_frame");

    for &size in &SCENARIO_SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_render_world(size);
            let mut batch = create_render_batch();
            let mut asset_server = AssetServer::new();

            b.iter(|| {
                batch.begin();
                batch.gather_sprites(&world, &mut asset_server).unwrap();
                batch.sort_sprites();
                let texture_size = Vec2::new(64.0, 64.0);
                let sprites = std::mem::take(&mut batch.sprites);
                for sprite in &sprites {
                    batch
                        .generate_sprite_vertices(sprite, texture_size)
                        .unwrap();
                }
                let count = sprites.len();
                batch.sprites = sprites;
                black_box((count, batch.vertices.len()));
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Benchmark Group 2: Metrics Capture
// ================================================================================================

fn bench_render_metrics_capture(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_metrics_capture");

    for &size in &SCENARIO_SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_render_world(size);
            let mut batch = create_render_batch();
            let mut asset_server = AssetServer::new();

            b.iter(|| {
                batch.begin();
                batch.gather_sprites(&world, &mut asset_server).unwrap();
                batch.sort_sprites();
                let dc = count_draw_calls(&batch.sprites);
                black_box((batch.sprite_count(), dc, batch.culled_count()));
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Benchmark Group 3: Frame Percentiles (iter_custom for P95/P99)
// ================================================================================================

fn bench_render_frame_percentiles(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_frame_percentiles");

    for &size in &SCENARIO_SIZES {
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_render_world(size);
            let mut batch = create_render_batch();
            let mut asset_server = AssetServer::new();

            b.iter_custom(|iters| {
                let mut total = std::time::Duration::ZERO;
                for _ in 0..iters {
                    let start = Instant::now();
                    batch.begin();
                    batch.gather_sprites(&world, &mut asset_server).unwrap();
                    batch.sort_sprites();
                    let texture_size = Vec2::new(64.0, 64.0);
                    let sprites = std::mem::take(&mut batch.sprites);
                    for sprite in &sprites {
                        batch
                            .generate_sprite_vertices(sprite, texture_size)
                            .unwrap();
                    }
                    let count = sprites.len();
                    batch.sprites = sprites;
                    black_box(count);
                    total += start.elapsed();
                }
                total
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Benchmark Group 4: Scaling Curve
// ================================================================================================

fn bench_render_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_scaling");

    for &size in &SCALING_SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_render_world(size);
            let mut batch = create_render_batch();
            let mut asset_server = AssetServer::new();

            b.iter(|| {
                batch.begin();
                batch.gather_sprites(&world, &mut asset_server).unwrap();
                batch.sort_sprites();
                let texture_size = Vec2::new(64.0, 64.0);
                let sprites = std::mem::take(&mut batch.sprites);
                for sprite in &sprites {
                    batch
                        .generate_sprite_vertices(sprite, texture_size)
                        .unwrap();
                }
                let count = sprites.len();
                batch.sprites = sprites;
                black_box((count, batch.vertices.len()));
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Criterion Configuration
// ================================================================================================

criterion_group!(full_frame_benches, bench_render_full_frame,);

criterion_group!(metrics_benches, bench_render_metrics_capture,);

criterion_group!(percentile_benches, bench_render_frame_percentiles,);

criterion_group!(scaling_benches, bench_render_scaling,);

// ================================================================================================
// Custom Main — supports RENDER_BASELINE=1 for JSON baseline generation
// ================================================================================================

fn generate_baseline() {
    println!("Generating render baseline...");

    let all_sizes: Vec<usize> = SCENARIO_SIZES
        .iter()
        .chain(SCALING_SIZES.iter())
        .copied()
        .collect();

    let metrics: Vec<RenderFrameMetrics> = all_sizes
        .iter()
        .map(|&size| capture_frame_metrics(size))
        .collect();

    #[derive(Serialize)]
    struct Baseline {
        generated_at: String,
        commit: String,
        scenarios: Vec<RenderFrameMetrics>,
    }

    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let baseline = Baseline {
        generated_at: format!("unix:{now}"),
        commit,
        scenarios: metrics,
    };

    let json = serde_json::to_string_pretty(&baseline).unwrap();

    let baselines_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("benches/baselines");
    std::fs::create_dir_all(&baselines_dir).unwrap();
    let path = baselines_dir.join("render_baseline.json");
    std::fs::write(&path, &json).unwrap();

    println!("Baseline written to {}", path.display());
    println!("{json}");
}

fn main() {
    if std::env::var("RENDER_BASELINE").is_ok() {
        generate_baseline();
        return;
    }

    // Run criterion benchmarks
    full_frame_benches();
    metrics_benches();
    percentile_benches();
    scaling_benches();
}
