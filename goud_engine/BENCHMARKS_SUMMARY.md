# GoudEngine Benchmark Results Summary

**Date:** 2026-01-05
**Version:** 0.0.808
**Rust Version:** 1.70+
**Hardware:** Development Machine

---

## Quick Start

Run all benchmarks:
```bash
cargo bench
```

Run specific benchmark suite:
```bash
cargo bench --bench ecs_benchmarks
cargo bench --bench asset_benchmarks
```

View HTML reports:
```bash
open target/criterion/report/index.html
```

---

## Implemented Benchmarks

### ✅ ECS Benchmarks (`ecs_benchmarks.rs`)

**Entity Operations:**
- ✅ Entity spawn (single, batch 100/1K/10K)
- ✅ Entity despawn (single, batch 100/1K/10K)

**Component Operations:**
- ✅ Component add (single to empty, multiple, archetype transition)
- ✅ Component remove (single, to empty)

**Query Operations:**
- ✅ Query iteration (single component, two components, sparse components)
- ✅ Tested with 100, 1K, 10K entity counts

**Archetype Operations:**
- ✅ Archetype find/create
- ✅ Component add transition
- ✅ Component remove transition

**System Operations:**
- ✅ System execution overhead with query iteration

### ✅ Asset Benchmarks (`asset_benchmarks.rs`)

**Storage Operations:**
- ✅ Asset insertion (small, large 1MB)
- ✅ Asset get (immutable, mutable)
- ✅ Asset removal
- ✅ Batch insertion (100/1K/10K assets)

**Loader Operations:**
- ✅ Asset loading (1KB, 100KB, 1MB)
- ✅ AssetServer handle reservation

### ⚠️ Physics Benchmarks (`physics_benchmarks.rs`)

**Status:** Disabled (API dependencies)
- ❌ Collision detection (circle-circle, box-box, circle-box)
- ❌ AABB computation
- ❌ Spatial hash operations
- ❌ Collision response

**Note:** Physics benchmarks require public API exports for `ColliderShape`, `aabb` module, and collision functions. Will be enabled after Phase 5 completion.

---

## Preliminary Results

### ECS Performance

| Operation | Target | Actual | Status | Notes |
|-----------|--------|--------|--------|-------|
| Entity spawn | <100ns | ~101ns | ⚠️ | Within 1% of target |
| Component add | <150ns | TBD | ⏳ | Needs full benchmark run |
| Query iteration | <50ns/entity | TBD | ⏳ | Needs full benchmark run |
| System overhead | <1μs | TBD | ⏳ | Needs full benchmark run |

**Legend:**
- ✅ Target met or exceeded
- ⚠️ Within 20% of target
- ❌ Below target
- ⏳ Awaiting full benchmark run

---

## Running Full Benchmarks

For accurate results, run on dedicated hardware with:

```bash
# Disable CPU frequency scaling (Linux)
sudo cpupower frequency-set -g performance

# Run full benchmark suite
cargo bench --no-fail-fast

# Results saved to target/criterion/
```

**Recommendations:**
- Close all other applications
- Run on battery power or disable power management
- Wait for system to cool down between runs
- Use `--save-baseline` for regression tracking

---

## Next Steps

1. **Complete Phase 5:** Export physics APIs for benchmarking
2. **Run Full Suite:** Complete benchmark run on dedicated hardware
3. **Baseline Establishment:** Save first full run as baseline
4. **CI Integration:** Add GitHub Actions workflow
5. **Performance Tracking:** Set up continuous benchmark monitoring

---

## Known Limitations

### Current Implementation

- **Physics Benchmarks Disabled:** Require public API exports
- **Path-based Asset Benchmarks Skipped:** AssetPath API needs stabilization
- **Handle Allocation Benchmarks Omitted:** Internal API not exposed
- **No Parallel System Benchmarks:** Requires parallel system execution

### Future Additions

- Graphics rendering benchmarks (sprite batching, draw call overhead)
- Audio system benchmarks (spatial audio, mixing)
- Input system benchmarks (action mapping, buffering)
- Memory allocation benchmarks
- Parallel system execution benchmarks
- Real-world game scenario benchmarks

---

## Benchmark Configuration

All benchmarks use [Criterion.rs](https://github.com/bheisler/criterion.rs) with:

- **Statistical Analysis:** Outlier detection, confidence intervals
- **HTML Reports:** Visual graphs and regression detection
- **Sample Size:** 100 iterations (adjustable with `--sample-size`)
- **Warm-up:** 3 seconds per benchmark
- **Measurement:** 5 seconds per benchmark

Configuration in `Cargo.toml`:
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "ecs_benchmarks"
harness = false
```

---

## Contributing

To add new benchmarks:

1. Add to appropriate benchmark file in `benches/`
2. Follow existing patterns (group, throughput, iter_batched)
3. Document performance targets in comments
4. Update this summary with new benchmarks
5. Run full suite and verify no regressions

See `BENCHMARKS.md` for detailed guidelines.

---

**Last Updated:** 2026-01-05
**Status:** Phase 6.4.3 Complete
**Next Review:** After Phase 5 completion
