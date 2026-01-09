# GoudEngine Performance Benchmarks

Comprehensive performance benchmarking suite using Criterion for the GoudEngine.

## Table of Contents

- [Overview](#overview)
- [Running Benchmarks](#running-benchmarks)
- [Benchmark Categories](#benchmark-categories)
- [Performance Targets](#performance-targets)
- [Interpreting Results](#interpreting-results)
- [CI Integration](#ci-integration)
- [Adding New Benchmarks](#adding-new-benchmarks)

---

## Overview

The GoudEngine benchmarking suite uses [Criterion](https://github.com/bheisler/criterion.rs), the industry-standard benchmarking framework for Rust. Criterion provides:

- **Statistical rigor**: Outlier detection, confidence intervals
- **HTML reports**: Visual graphs and detailed analysis
- **Regression detection**: Automatic detection of performance changes
- **Cross-platform**: Works on Windows, macOS, Linux

All benchmarks are located in `goud_engine/benches/` and organized into three main categories:

1. **ECS Benchmarks** (`ecs_benchmarks.rs`)
2. **Asset Benchmarks** (`asset_benchmarks.rs`)
3. **Physics Benchmarks** (`physics_benchmarks.rs`)

---

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

This runs all benchmarks and generates HTML reports in `target/criterion/`.

### Run Specific Benchmark Suite

```bash
# ECS benchmarks only
cargo bench --bench ecs_benchmarks

# Asset system benchmarks only
cargo bench --bench asset_benchmarks

# Physics benchmarks only
cargo bench --bench physics_benchmarks
```

### Run Specific Benchmark

```bash
# Run only entity spawn benchmarks
cargo bench --bench ecs_benchmarks -- entity_spawn

# Run only query iteration benchmarks
cargo bench --bench ecs_benchmarks -- query_iteration
```

### Baseline Comparison

Save current performance as baseline:

```bash
cargo bench -- --save-baseline my-baseline
```

Compare against baseline:

```bash
cargo bench -- --baseline my-baseline
```

### Quick Run (Reduced Samples)

For faster iteration during development:

```bash
cargo bench -- --quick
```

---

## Benchmark Categories

### 1. ECS Benchmarks (`ecs_benchmarks.rs`)

Tests Entity-Component-System performance:

#### Entity Operations
- **entity_spawn/single**: Single entity spawn (target: <100ns)
- **entity_spawn/batch**: Batch spawn 100/1K/10K entities
- **entity_despawn/single**: Single entity despawn
- **entity_despawn/batch**: Batch despawn 100/1K/10K entities

#### Component Operations
- **component_add/single_to_empty**: Add component to empty entity (target: <150ns)
- **component_add/multiple_to_empty**: Add 3 components to entity
- **component_add/archetype_transition**: Component add causing archetype change
- **component_remove/single**: Remove single component
- **component_remove/to_empty**: Remove last component

#### Query Operations
- **query_iteration/single_component**: Iterate 100/1K/10K entities with 1 component (target: <50ns/entity)
- **query_iteration/two_components**: Iterate with 2 components
- **query_iteration/with_filter**: Iterate with component filter

#### Archetype Operations
- **archetype_transitions/find_or_create_archetype**: Archetype lookup/creation
- **archetype_transitions/add_component_transition**: Add component transition
- **archetype_transitions/remove_component_transition**: Remove component transition

#### System Execution
- **system_execution/movement_system**: Full system with 100/1K/10K entities (target: <1μs overhead)

### 2. Asset Benchmarks (`asset_benchmarks.rs`)

Tests asset loading and management:

#### Handle Operations
- **handle_allocation/single**: Single handle allocation (target: <50ns)
- **handle_allocation/batch**: Batch allocate 100/1K/10K handles
- **handle_deallocation/single**: Single handle deallocation
- **handle_deallocation/batch**: Batch deallocate 100/1K/10K handles
- **handle_validation/valid_handle**: Validate alive handle (target: <10ns)
- **handle_validation/invalid_handle**: Validate dead handle

#### Storage Operations
- **asset_storage/insert_small**: Insert small asset (target: <100ns)
- **asset_storage/insert_large**: Insert large asset (1MB)
- **asset_storage/get**: Get asset reference (target: <10ns)
- **asset_storage/get_mut**: Get mutable asset reference
- **asset_storage/remove**: Remove asset
- **asset_storage/batch_insert**: Batch insert 100/1K/10K assets

#### Path Operations
- **asset_storage_paths/insert_with_path**: Insert with path deduplication
- **asset_storage_paths/get_by_path_hit**: Path lookup (cache hit)
- **asset_storage_paths/get_by_path_miss**: Path lookup (cache miss)

#### Loader Operations
- **asset_loader/load_1kb**: Load 1KB asset
- **asset_loader/load_100kb**: Load 100KB asset
- **asset_loader/load_1mb**: Load 1MB asset (target: <10ms)

### 3. Physics Benchmarks (`physics_benchmarks.rs`)

Tests physics simulation performance:

#### Collision Detection
- **collision_circle_circle/overlapping**: Circle-circle overlap (target: <50ns)
- **collision_circle_circle/separated**: Circle-circle separated
- **collision_circle_circle/batch**: Batch check 100/1K/10K pairs
- **collision_box_box/aabb_overlapping**: AABB-AABB overlap
- **collision_box_box/obb_rotated**: OBB-OBB with rotation (target: <200ns)
- **collision_box_box/batch_aabb**: Batch AABB checks 100/1K/10K pairs
- **collision_circle_box/circle_aabb**: Circle vs AABB
- **collision_circle_box/circle_obb**: Circle vs OBB (rotated)

#### AABB Operations
- **aabb_computation/circle**: Circle AABB (target: <20ns)
- **aabb_computation/aabb**: AABB shape
- **aabb_computation/obb**: OBB shape
- **aabb_computation/polygon**: Polygon AABB
- **aabb_operations/overlaps**: AABB overlap test
- **aabb_operations/intersection**: AABB intersection
- **aabb_operations/merge**: AABB merge
- **aabb_operations/contains_point**: Point containment
- **aabb_operations/raycast**: AABB raycast

#### Broad Phase
- **spatial_hash/insert**: Insert 100/1K/10K entities into grid
- **spatial_hash/query_pairs**: Query all collision pairs (target: <100μs for 10K)
- **spatial_hash/query_aabb**: AABB region query

#### Collision Response
- **collision_response/resolve_impulse**: Impulse resolution
- **collision_response/position_correction**: Position correction (Baumgarte)

---

## Performance Targets

### ECS Performance

| Operation | Target | Actual | Status |
|-----------|--------|--------|--------|
| Entity spawn | <100ns | TBD | ⏳ |
| Component add | <150ns | TBD | ⏳ |
| Query iteration | <50ns/entity | TBD | ⏳ |
| System overhead | <1μs | TBD | ⏳ |
| Batch spawn (10K) | <500μs | TBD | ⏳ |

### Asset Performance

| Operation | Target | Actual | Status |
|-----------|--------|--------|--------|
| Handle allocation | <50ns | TBD | ⏳ |
| Asset lookup | <10ns | TBD | ⏳ |
| Asset insertion | <100ns | TBD | ⏳ |
| Texture load (1024²) | <10ms | TBD | ⏳ |
| Batch load (100) | <100ms | TBD | ⏳ |

### Physics Performance

| Operation | Target | Actual | Status |
|-----------|--------|--------|--------|
| Circle-circle | <50ns | TBD | ⏳ |
| Box-box SAT | <200ns | TBD | ⏳ |
| AABB computation | <20ns | TBD | ⏳ |
| Spatial hash query | <1μs (1K) | TBD | ⏳ |
| Broad phase | <100μs (10K) | TBD | ⏳ |

**Legend:**
- ✅ Target met or exceeded
- ⚠️ Within 20% of target
- ❌ Below target
- ⏳ Awaiting first benchmark run

---

## Interpreting Results

### Criterion Output

Criterion produces detailed statistical analysis for each benchmark:

```
entity_spawn/single     time:   [45.231 ns 45.892 ns 46.623 ns]
                        change: [-5.2341% -3.1234% -0.8912%] (p = 0.01 < 0.05)
                        Performance has improved.
```

**Key metrics:**
- **time**: Median time with 95% confidence interval
- **change**: Performance change vs. previous run
- **p-value**: Statistical significance (p < 0.05 = significant)

### HTML Reports

Open `target/criterion/report/index.html` for interactive reports with:
- Performance graphs over time
- Distribution plots (violin plots)
- Outlier detection
- Regression analysis

### Performance Analysis

**What to look for:**

1. **Regressions**: Look for "Performance has regressed" messages
2. **High variance**: Wide confidence intervals indicate unstable benchmarks
3. **Outliers**: Check if outliers are legitimate or measurement noise
4. **Throughput**: Compare operations/second across different sizes

**Common issues:**

- **Thermal throttling**: Run benchmarks after system cooldown
- **Background processes**: Close other applications during benchmarking
- **Power management**: Disable CPU frequency scaling for consistency
- **Compiler optimizations**: Ensure release mode (`--release`)

---

## CI Integration

### GitHub Actions

Add to `.github/workflows/benchmarks.yml`:

```yaml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo bench --no-fail-fast

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
```

### Continuous Benchmarking

For tracking performance over time, consider:

- [bencher.dev](https://bencher.dev) - Continuous benchmark tracking
- [codspeed.io](https://codspeed.io) - Performance monitoring for PRs
- Store Criterion baseline in git for regression detection

---

## Adding New Benchmarks

### 1. Choose Benchmark Category

Add to existing file or create new `[[bench]]` in `Cargo.toml`:

```toml
[[bench]]
name = "my_benchmarks"
harness = false
```

### 2. Write Benchmark Function

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_operation(c: &mut Criterion) {
    c.bench_function("my_operation", |b| {
        // Setup code here (not timed)

        b.iter(|| {
            // Code to benchmark (timed)
            let result = my_operation();
            black_box(result); // Prevent compiler optimization
        });
    });
}

criterion_group!(my_benches, bench_my_operation);
criterion_main!(my_benches);
```

### 3. Use `iter_batched` for Setup/Teardown

```rust
b.iter_batched(
    || {
        // Setup (not timed)
        setup_data()
    },
    |data| {
        // Benchmark (timed)
        operation(data)
    },
    criterion::BatchSize::SmallInput,
);
```

### 4. Parameterized Benchmarks

```rust
use criterion::{BenchmarkId, Throughput};

let mut group = c.benchmark_group("my_group");

for size in [100, 1_000, 10_000] {
    group.throughput(Throughput::Elements(size as u64));
    group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
        b.iter(|| operation(size));
    });
}

group.finish();
```

### 5. Best Practices

- **Use `black_box()`**: Prevents dead code elimination
- **Minimize setup cost**: Use `iter_batched` to exclude setup time
- **Control inputs**: Use consistent data across runs
- **Document targets**: Add performance target comments
- **Group related benches**: Use `benchmark_group!` for organization
- **Set throughput**: Use `Throughput::Elements()` or `Throughput::Bytes()`

---

## Troubleshooting

### Benchmarks Fail to Compile

```bash
# Check benchmark dependencies
cargo check --benches

# Run specific benchmark
cargo bench --bench ecs_benchmarks --no-run
```

### Unstable Results

- Run on dedicated hardware (no VMs if possible)
- Disable CPU frequency scaling
- Close background applications
- Increase sample size: `cargo bench -- --sample-size 200`

### Missing HTML Reports

Ensure Criterion HTML feature is enabled in `Cargo.toml`:

```toml
criterion = { version = "0.5", features = ["html_reports"] }
```

### Benchmark Takes Too Long

Use `--quick` for development:

```bash
cargo bench -- --quick
```

Or reduce sample sizes in code:

```rust
let mut group = c.benchmark_group("my_group");
group.sample_size(10); // Default is 100
```

---

## Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Benchmarking in Rust](https://doc.rust-lang.org/book/ch11-03-test-organization.html#benchmark-tests)
- [Writing Reliable Benchmarks](https://easyperf.net/blog/2018/01/07/Microbenchmarks)

---

**Last Updated:** 2026-01-05
**Criterion Version:** 0.5
**Rust Version:** 1.70+
