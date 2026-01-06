//! ECS Performance Benchmarks
//!
//! Comprehensive benchmarks for Entity-Component-System operations including:
//! - Entity spawn/despawn performance
//! - Component add/remove performance
//! - Query iteration performance
//! - System execution performance
//! - Archetype transition performance
//!
//! Run with: `cargo bench --bench ecs_benchmarks`
//!
//! ## Performance Targets
//!
//! - Entity spawn: <100ns per entity
//! - Component add: <150ns per component
//! - Query iteration: <50ns per entity
//! - System execution: <1μs overhead
//! - Batch spawn (10K): <500μs total

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use goud_engine::ecs::*;
use std::hint::black_box as std_black_box;

// ================================================================================================
// Component Definitions
// ================================================================================================

#[derive(Clone, Copy, Debug)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Clone, Copy, Debug)]
struct Velocity {
    x: f32,
    y: f32,
}
impl Component for Velocity {}

#[derive(Clone, Copy, Debug)]
struct Health {
    current: f32,
    max: f32,
}
impl Component for Health {}

#[derive(Clone, Copy, Debug)]
struct Damage {
    amount: f32,
}
impl Component for Damage {}

// ================================================================================================
// Entity Spawn/Despawn Benchmarks
// ================================================================================================

fn bench_entity_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_spawn");

    // Single entity spawn
    group.bench_function("single", |b| {
        let mut world = World::new();
        b.iter(|| {
            let entity = world.spawn_empty();
            black_box(entity);
        });
    });

    // Batch entity spawn
    for size in [100, 1_000, 10_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("batch", size), &size, |b, &size| {
            let mut world = World::new();
            b.iter(|| {
                let entities = world.spawn_batch(size);
                black_box(entities);
            });
        });
    }

    group.finish();
}

fn bench_entity_despawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_despawn");

    // Single entity despawn
    group.bench_function("single", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entity = world.spawn_empty();
                (world, entity)
            },
            |(mut world, entity)| {
                let result = world.despawn(entity);
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Batch entity despawn
    for size in [100, 1_000, 10_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("batch", size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let mut world = World::new();
                    let entities = world.spawn_batch(size);
                    (world, entities)
                },
                |(mut world, entities)| {
                    let count = world.despawn_batch(&entities);
                    black_box(count);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ================================================================================================
// Component Add/Remove Benchmarks
// ================================================================================================

fn bench_component_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_add");

    // Add single component to empty entity
    group.bench_function("single_to_empty", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entity = world.spawn_empty();
                (world, entity)
            },
            |(mut world, entity)| {
                world.insert(entity, Position { x: 0.0, y: 0.0 });
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Add multiple components to empty entity
    group.bench_function("multiple_to_empty", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entity = world.spawn_empty();
                (world, entity)
            },
            |(mut world, entity)| {
                world.insert(entity, Position { x: 0.0, y: 0.0 });
                world.insert(entity, Velocity { x: 1.0, y: 1.0 });
                world.insert(entity, Health { current: 100.0, max: 100.0 });
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Add component causing archetype transition
    group.bench_function("archetype_transition", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entity = world.spawn_empty();
                world.insert(entity, Position { x: 0.0, y: 0.0 });
                (world, entity)
            },
            |(mut world, entity)| {
                world.insert(entity, Velocity { x: 1.0, y: 1.0 });
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_component_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_remove");

    // Remove single component
    group.bench_function("single", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entity = world.spawn_empty();
                world.insert(entity, Position { x: 0.0, y: 0.0 });
                world.insert(entity, Velocity { x: 1.0, y: 1.0 });
                (world, entity)
            },
            |(mut world, entity)| {
                let result = world.remove::<Position>(entity);
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Remove to empty archetype
    group.bench_function("to_empty", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entity = world.spawn_empty();
                world.insert(entity, Position { x: 0.0, y: 0.0 });
                (world, entity)
            },
            |(mut world, entity)| {
                let result = world.remove::<Position>(entity);
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ================================================================================================
// Query Benchmarks
// ================================================================================================

fn bench_query_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_iteration");

    for entity_count in [100, 1_000, 10_000] {
        group.throughput(Throughput::Elements(entity_count as u64));

        // Iterate entities with single component
        group.bench_with_input(
            BenchmarkId::new("single_component", entity_count),
            &entity_count,
            |b, &count| {
                let mut world = World::new();
                for i in 0..count {
                    let entity = world.spawn_empty();
                    world.insert(entity, Position { x: i as f32, y: i as f32 });
                }

                b.iter(|| {
                    let query = Query::<&Position>::new(&world);
                    let mut sum = 0.0;
                    for pos in query.iter(&world) {
                        sum += pos.x + pos.y;
                    }
                    black_box(sum);
                });
            },
        );

        // Iterate entities with two components
        group.bench_with_input(
            BenchmarkId::new("two_components", entity_count),
            &entity_count,
            |b, &count| {
                let mut world = World::new();
                for i in 0..count {
                    let entity = world.spawn_empty();
                    world.insert(entity, Position { x: i as f32, y: i as f32 });
                    world.insert(entity, Velocity { x: 1.0, y: 1.0 });
                }

                b.iter(|| {
                    let query = Query::<(&Position, &Velocity)>::new(&world);
                    let mut sum = 0.0;
                    for (pos, vel) in query.iter(&world) {
                        sum += pos.x + vel.x;
                    }
                    black_box(sum);
                });
            },
        );

        // Iterate entities with sparse component distribution
        group.bench_with_input(
            BenchmarkId::new("sparse_components", entity_count),
            &entity_count,
            |b, &count| {
                let mut world = World::new();
                for i in 0..count {
                    let entity = world.spawn_empty();
                    world.insert(entity, Position { x: i as f32, y: i as f32 });
                    // Only half have Velocity (sparse distribution)
                    if i % 2 == 0 {
                        world.insert(entity, Velocity { x: 1.0, y: 1.0 });
                    }
                }

                b.iter(|| {
                    // Query both components (tests sparse iteration)
                    let query = Query::<(&Position, &Velocity)>::new(&world);
                    let mut sum = 0.0;
                    for (pos, vel) in query.iter(&world) {
                        sum += pos.x + vel.x;
                    }
                    black_box(sum);
                });
            },
        );
    }

    group.finish();
}

// ================================================================================================
// Archetype Benchmarks
// ================================================================================================

fn bench_archetype_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("archetype_transitions");

    // Benchmark archetype graph lookups
    group.bench_function("find_or_create_archetype", |b| {
        let mut world = World::new();

        b.iter(|| {
            // Create entity with unique component combination
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 0.0, y: 0.0 });
            world.insert(entity, Velocity { x: 1.0, y: 1.0 });
            world.despawn(entity);
        });
    });

    // Benchmark component add causing transition
    group.bench_function("add_component_transition", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entity = world.spawn_empty();
                world.insert(entity, Position { x: 0.0, y: 0.0 });
                (world, entity)
            },
            |(mut world, entity)| {
                world.insert(entity, Velocity { x: 1.0, y: 1.0 });
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Benchmark component remove causing transition
    group.bench_function("remove_component_transition", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entity = world.spawn_empty();
                world.insert(entity, Position { x: 0.0, y: 0.0 });
                world.insert(entity, Velocity { x: 1.0, y: 1.0 });
                (world, entity)
            },
            |(mut world, entity)| {
                world.remove::<Velocity>(entity);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ================================================================================================
// System Benchmarks
// ================================================================================================

fn movement_system(world: &mut World) {
    // Simulate physics update using mutable query
    let query = Query::<(&Position, &Velocity)>::new(world);

    // Collect velocities to apply (read-only)
    let updates: Vec<_> = query
        .iter(world)
        .map(|(pos, vel)| (pos.x, pos.y, vel.x, vel.y))
        .collect();

    // Benchmark overhead: in a real system this would use iter_mut
    // For now we just verify the query iteration works
    std_black_box(updates.len());
}

fn bench_system_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_execution");

    for entity_count in [100, 1_000, 10_000] {
        group.throughput(Throughput::Elements(entity_count as u64));

        group.bench_with_input(
            BenchmarkId::new("movement_system", entity_count),
            &entity_count,
            |b, &count| {
                let mut world = World::new();
                for i in 0..count {
                    let entity = world.spawn_empty();
                    world.insert(entity, Position { x: i as f32, y: i as f32 });
                    world.insert(entity, Velocity { x: 1.0, y: 1.0 });
                }

                b.iter(|| {
                    movement_system(&mut world);
                });
            },
        );
    }

    group.finish();
}

// ================================================================================================
// Criterion Configuration
// ================================================================================================

criterion_group!(
    entity_benches,
    bench_entity_spawn,
    bench_entity_despawn,
);

criterion_group!(
    component_benches,
    bench_component_add,
    bench_component_remove,
);

criterion_group!(
    query_benches,
    bench_query_iteration,
);

criterion_group!(
    archetype_benches,
    bench_archetype_transitions,
);

criterion_group!(
    system_benches,
    bench_system_execution,
);

criterion_main!(
    entity_benches,
    component_benches,
    query_benches,
    archetype_benches,
    system_benches,
);
