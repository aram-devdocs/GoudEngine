# Profiling

GoudEngine has optional Tracy and Puffin instrumentation for local engine profiling. It is off by default and is not part of the release build surface unless a profiling feature is enabled.

## Run With Profiler Spans

Start a local engine process with one profiler backend enabled:

```bash
cargo run -p sandbox --features goud_engine/profiling-tracy
cargo run -p sandbox --features goud_engine/profiling-puffin
```

Use one backend feature per profiling run. The CI all-features path prefers Tracy when both feature flags are present, but local captures should choose one backend so the capture is unambiguous.

For Tracy, open the Tracy viewer while the sandbox is running and connect to the process. For Puffin, attach `puffin_viewer` through an app-side `puffin_http` sink, or use an in-app `puffin_egui` view. Puffin scope sites turn capture on, and WGPU `end_frame` advances Puffin frames; headless tools that profile ECS without a rendered frame should advance the app-side Puffin frame sink themselves. The viewer transport stays outside engine core so the engine does not bind a TCP port.

Both backends emit spans for the wgpu frame path and ECS schedule execution, including:

- `wgpu.begin_frame`
- `wgpu.end_frame`
- `wgpu.uniform_upload`
- `wgpu.shadow_pass`
- `wgpu.render_pass`
- `wgpu.gpu_submit`
- `ecs.run_system`
- `ecs.system_stage`
- `ecs.system`
- `ecs.parallel_stage`
- `ecs.parallel_system`

The feature is intentionally separate from the fixed frame phase counters. Use Tracy to investigate local call paths and flamegraphs; use the ENG2 phase counters and `scripts/perf-capture` reports for gate evidence.

For compile-only validation, run both feature paths:

```bash
cargo check -p goud-engine-core --features profiling-tracy
cargo check -p sandbox --features goud_engine/profiling-tracy
cargo check -p goud-engine-core --features profiling-puffin
cargo check -p sandbox --features goud_engine/profiling-puffin
```

## Default Build Cost

Without a profiling feature, the span sites are removed at compile time and the default feature graph does not activate the Tracy backend (`tracy-client` or `tracy-client-sys`) or the Puffin backend (`puffin`). The lockfile can still contain the generic `profiling` crate because existing transitive dependencies use it.
