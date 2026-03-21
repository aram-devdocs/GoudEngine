//! Spatial Grid Performance Benchmarks
//!
//! Benchmarks for spatial grid operations including insertion, update,
//! radius queries, and mixed workloads.
//!
//! Run with: `cargo bench --bench spatial_grid_benchmarks`
//!
//! ## Performance Targets
//!
//! - Insert 10K entities: <1ms
//! - Query radius (10K entities): <0.1ms
//! - Update 10K entities: <1ms

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use goud_engine::core::math::Vec2;
use goud_engine::ecs::{Entity, SpatialGrid};

/// Deterministic pseudo-random number generator for reproducible benchmarks.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self, min: f32, max: f32) -> f32 {
        // xorshift64
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        let t = (self.state as f32) / (u64::MAX as f32);
        min + t * (max - min)
    }
}

fn make_grid_with_entities(count: usize, cell_size: f32) -> SpatialGrid {
    let mut grid = SpatialGrid::with_capacity(cell_size, count);
    let mut rng = SimpleRng::new(42);
    for i in 0..count {
        let x = rng.next_f32(0.0, 10000.0);
        let y = rng.next_f32(0.0, 10000.0);
        grid.insert(Entity::new(i as u32, 0), Vec2::new(x, y));
    }
    grid
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_grid_insert");

    for &count in &[1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                let mut grid = SpatialGrid::with_capacity(32.0, count);
                let mut rng = SimpleRng::new(42);
                for i in 0..count {
                    let x = rng.next_f32(0.0, 10000.0);
                    let y = rng.next_f32(0.0, 10000.0);
                    grid.insert(Entity::new(i as u32, 0), Vec2::new(x, y));
                }
                black_box(&grid);
            });
        });
    }

    group.finish();
}

fn bench_query_radius(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_grid_query_radius");

    for &count in &[1_000, 10_000] {
        let grid = make_grid_with_entities(count, 32.0);

        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, _| {
            let mut rng = SimpleRng::new(99);
            b.iter(|| {
                let cx = rng.next_f32(0.0, 10000.0);
                let cy = rng.next_f32(0.0, 10000.0);
                let results = grid.query_radius(Vec2::new(cx, cy), 100.0);
                black_box(&results);
            });
        });
    }

    group.finish();
}

fn bench_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_grid_update");

    for &count in &[1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            let mut grid = make_grid_with_entities(count, 32.0);
            let mut rng = SimpleRng::new(77);
            b.iter(|| {
                for i in 0..count {
                    let x = rng.next_f32(0.0, 10000.0);
                    let y = rng.next_f32(0.0, 10000.0);
                    grid.update(Entity::new(i as u32, 0), Vec2::new(x, y));
                }
                black_box(&grid);
            });
        });
    }

    group.finish();
}

fn bench_mixed_workload(c: &mut Criterion) {
    c.bench_function("spatial_grid_mixed_10k", |b| {
        let mut grid = make_grid_with_entities(10_000, 32.0);
        let mut rng = SimpleRng::new(123);

        b.iter(|| {
            // Simulate one frame: update 10% positions, query 5 radius lookups
            for i in 0..1_000 {
                let x = rng.next_f32(0.0, 10000.0);
                let y = rng.next_f32(0.0, 10000.0);
                grid.update(Entity::new(i as u32, 0), Vec2::new(x, y));
            }
            for _ in 0..5 {
                let cx = rng.next_f32(0.0, 10000.0);
                let cy = rng.next_f32(0.0, 10000.0);
                let results = grid.query_radius(Vec2::new(cx, cy), 100.0);
                black_box(&results);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_insert,
    bench_query_radius,
    bench_update,
    bench_mixed_workload
);
criterion_main!(benches);
