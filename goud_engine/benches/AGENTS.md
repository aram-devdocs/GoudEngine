# benches/ — Benchmarks

## Purpose

Performance benchmarks using the `criterion` crate. Focus on hot paths.

## Files

- `ecs_benchmarks.rs` — Entity creation, component iteration, system execution
- `asset_benchmarks.rs` — Asset loading, caching, handle resolution
- `sprite_batch_benchmarks.rs` — Sprite batch gather, sort, vertex generation, full CPU pipeline
- `physics_benchmarks.rs.disabled` — Physics benchmarks (currently disabled)

## Running

```bash
cargo bench              # Run all benchmarks
cargo bench -- ecs       # Run ECS benchmarks only
cargo bench -- asset     # Run asset benchmarks only
```

## Patterns

- Use `criterion` groups for related benchmarks
- Benchmark hot paths: component queries, system dispatch, asset lookups
- Include setup/teardown outside the measured section
- Compare against baseline with `cargo bench -- --save-baseline <name>`

## Adding a Benchmark

1. Create `new_benchmarks.rs` in this directory
2. Add to `Cargo.toml` under `[[bench]]`
3. Use `criterion_group!` and `criterion_main!` macros
4. Focus on operations that run per-frame or per-entity

