//! Engine Tick Benchmark Suite
//!
//! Drives a headless engine tick: build a [`World`], spawn entities arranged in
//! 3-level `Transform2D` hierarchies (root → child → grandchild), and run the
//! transform-propagation schedule once per iteration.
//!
//! Run with: `cargo bench --bench engine_tick_benchmarks`
//!
//! ## Why this exists
//!
//! `engine_tick/tick_10k` is the **ratio-normalization denominator** for the
//! bench gate (`scripts/bench-gate.py`). Renderer benchmarks are compared as a
//! ratio against this tick so results survive runner-to-runner variance: if the
//! whole machine is 20% slower, both the renderer bench and this tick slow down
//! together and the ratio is unchanged.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use goud_engine::core::math::Vec2;
use goud_engine::ecs::components::propagation::propagate_transforms_2d;
use goud_engine::ecs::components::{Children, GlobalTransform2D, Parent, Transform2D};
use goud_engine::ecs::World;

/// Builds a world of `entity_count` entities arranged as 3-level hierarchies.
///
/// Every group of three entities forms a root → child → grandchild chain. Each
/// entity carries a `Transform2D` and a `GlobalTransform2D`; children carry a
/// `Parent` and parents a `Children` list, which is exactly what
/// [`propagate_transforms_2d`] traverses.
fn build_hierarchy_world(entity_count: usize) -> World {
    let mut world = World::new();
    let triples = entity_count / 3;

    for i in 0..triples {
        let root = world.spawn_empty();
        world.insert(root, Transform2D::from_position(Vec2::new(i as f32, 0.0)));
        world.insert(root, GlobalTransform2D::IDENTITY);

        let child = world.spawn_empty();
        world.insert(
            child,
            Transform2D::from_position(Vec2::new(1.0, (i % 10) as f32)),
        );
        world.insert(child, GlobalTransform2D::IDENTITY);
        world.insert(child, Parent::new(root));

        let grandchild = world.spawn_empty();
        world.insert(grandchild, Transform2D::from_position(Vec2::new(0.5, 0.5)));
        world.insert(grandchild, GlobalTransform2D::IDENTITY);
        world.insert(grandchild, Parent::new(child));

        let mut root_children = Children::new();
        root_children.push(child);
        world.insert(root, root_children);

        let mut child_children = Children::new();
        child_children.push(grandchild);
        world.insert(child, child_children);
    }

    world
}

fn bench_engine_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_tick");

    for &n in &[10_000usize, 50_000] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(format!("tick_{}", label(n)), |b| {
            let mut world = build_hierarchy_world(n);
            b.iter(|| {
                propagate_transforms_2d(black_box(&mut world));
            });
        });
    }

    group.finish();
}

/// Compact size label: 10000 -> "10k", 50000 -> "50k".
fn label(n: usize) -> String {
    if n.is_multiple_of(1000) {
        format!("{}k", n / 1000)
    } else {
        n.to_string()
    }
}

criterion_group!(engine_tick_benches, bench_engine_tick);
criterion_main!(engine_tick_benches);
