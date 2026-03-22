//! Pool & Arena Performance Benchmarks
//!
//! Benchmarks for EntityPool and FrameArena operations including:
//! - Pool acquire/release (single and batch)
//! - Pool vs raw World::spawn_empty comparison
//! - Arena allocation and reset cycles
//!
//! Run with: `cargo bench --bench pool_benchmarks`
//!
//! ## Performance Targets
//!
//! - Pool acquire: O(1), zero allocation
//! - Pool release: O(1), zero allocation
//! - Pool 10K iteration: <1ms
//! - Arena alloc+reset cycle (1000 items): <10μs

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use goud_engine::core::arena::FrameArena;
use goud_engine::core::pool::EntityPool;
use goud_engine::ecs::World;

// ================================================================================================
// Pool Acquire/Release Benchmarks
// ================================================================================================

fn bench_pool_acquire_release(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_acquire_release");

    // Acquire a single entity from a pre-populated pool
    group.bench_function("acquire_single", |b| {
        b.iter_batched(
            || {
                let mut pool = EntityPool::new(10_000);
                for i in 0..10_000 {
                    pool.set_slot_entity(i, (i as u64 + 1) * 100);
                }
                pool
            },
            |mut pool| {
                let result = pool.acquire();
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Release a single entity back to the pool
    group.bench_function("release_single", |b| {
        b.iter_batched(
            || {
                let mut pool = EntityPool::new(10_000);
                for i in 0..10_000 {
                    pool.set_slot_entity(i, (i as u64 + 1) * 100);
                }
                let (slot, _) = pool.acquire().unwrap();
                (pool, slot)
            },
            |(mut pool, slot)| {
                let result = pool.release(slot);
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Acquire then release in a tight loop (simulates rapid reuse)
    group.bench_function("acquire_release_cycle", |b| {
        let mut pool = EntityPool::new(10_000);
        for i in 0..10_000 {
            pool.set_slot_entity(i, (i as u64 + 1) * 100);
        }
        // Acquire one to set up a slot for cycling
        let (slot, _) = pool.acquire().unwrap();
        pool.release(slot);

        b.iter(|| {
            let (slot, eid) = pool.acquire().unwrap();
            black_box(eid);
            pool.release(slot);
        });
    });

    group.finish();
}

// ================================================================================================
// Pool vs Raw Spawn Comparison
// ================================================================================================

fn bench_pool_vs_raw_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_vs_raw_spawn");

    // Pool acquire (pre-allocated)
    group.bench_function("pool_acquire", |b| {
        let mut pool = EntityPool::new(100_000);
        for i in 0..100_000 {
            pool.set_slot_entity(i, (i as u64 + 1) * 100);
        }
        // Pre-acquire and release one to warm up
        let (slot, _) = pool.acquire().unwrap();
        pool.release(slot);

        b.iter(|| {
            let (slot, eid) = pool.acquire().unwrap();
            black_box(eid);
            pool.release(slot);
        });
    });

    // World::spawn_empty (allocating)
    group.bench_function("world_spawn_empty", |b| {
        let mut world = World::new();
        b.iter(|| {
            let entity = world.spawn_empty();
            black_box(entity);
        });
    });

    group.finish();
}

// ================================================================================================
// Batch Acquire Benchmarks
// ================================================================================================

fn bench_pool_batch_acquire(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_batch_acquire");

    for size in [100, 1_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("batch", size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let mut pool = EntityPool::new(10_000);
                    for i in 0..10_000 {
                        pool.set_slot_entity(i, (i as u64 + 1) * 100);
                    }
                    pool
                },
                |mut pool| {
                    let result = pool.acquire_batch(size);
                    black_box(result);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ================================================================================================
// Pool 10K Iteration Benchmark
// ================================================================================================

fn bench_pool_10k_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_10k_iteration");

    // Acquire 10K entities and check active status (simulates iteration)
    group.bench_function("acquire_10k_check_active", |b| {
        b.iter_batched(
            || {
                let mut pool = EntityPool::new(10_000);
                for i in 0..10_000 {
                    pool.set_slot_entity(i, (i as u64 + 1) * 100);
                }
                pool
            },
            |mut pool| {
                // Acquire all 10K
                let batch = pool.acquire_batch(10_000);
                // Iterate and check active status
                let mut count = 0usize;
                for &(slot, _) in &batch {
                    if pool.is_active(slot) {
                        count += 1;
                    }
                }
                black_box(count);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ================================================================================================
// Arena Alloc + Reset Benchmarks
// ================================================================================================

#[derive(Clone, Copy)]
#[repr(C)]
struct BenchData {
    _x: f32,
    _y: f32,
    _z: f32,
    _w: f32,
}

fn bench_arena_alloc_reset(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_alloc_reset");

    // Allocate 1000 items then reset
    group.bench_function("alloc_1000_reset", |b| {
        let mut arena = FrameArena::new();

        b.iter(|| {
            for i in 0..1_000 {
                let val = arena.alloc(BenchData {
                    _x: i as f32,
                    _y: i as f32 * 2.0,
                    _z: i as f32 * 3.0,
                    _w: i as f32 * 4.0,
                });
                black_box(val);
            }
            arena.reset();
        });
    });

    // Single allocation (measures raw alloc speed)
    group.bench_function("alloc_single", |b| {
        let arena = FrameArena::new();

        b.iter(|| {
            let val = arena.alloc(42u64);
            black_box(val);
        });
        // Note: arena grows but never resets here; acceptable for micro-bench
    });

    // Slice copy allocation
    group.bench_function("alloc_slice_1000", |b| {
        let mut arena = FrameArena::with_capacity(1024 * 1024);
        let data: Vec<f32> = (0..1_000).map(|i| i as f32).collect();

        b.iter(|| {
            let slice = arena.alloc_slice_copy(&data);
            black_box(slice);
            arena.reset();
        });
    });

    group.finish();
}

// ================================================================================================
// Criterion Configuration
// ================================================================================================

criterion_group!(
    benches,
    bench_pool_acquire_release,
    bench_pool_vs_raw_spawn,
    bench_pool_batch_acquire,
    bench_pool_10k_iteration,
    bench_arena_alloc_reset,
);
criterion_main!(benches);
