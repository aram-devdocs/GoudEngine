# Performance Definition of Done

The failure mode this program exists to prevent: shipping a "perf" feature and closing its milestone without proving it against a consumer-scale scene — while the instrumentation that should have caught the regression reports zero. (That is exactly how sprite batching, materials/shadows, and metrics all shipped "done" while Throne still ran at 1–17 FPS.)

## The rule

A performance issue closes **only** when BOTH hold:

1. **The named scripted scene hits its numeric target**, captured via `scripts/perf-capture` (ENG2-P0-07), 3-run median, on the Apple M-series dev machine (Metal) — or the CI ratio-gate (`scripts/bench-gate.py`) passes for CI-runnable benches.
2. **A phase counter attributes the cost.** The relevant `gpu_*`/CPU phase timing must be non-zero and account for the frame. **A counter that reads zero means broken instrumentation, never "free."**

The closing PR posts the capture (scene, numbers, phase breakdown, bench-gate delta) on the issue.

## Named scenes

| Scene | Definition | Gate target (super-high-FPS) |
|---|---|---|
| **S1 throne-baseline** | ~6,400 terrain planes + ~1,431 model instances (~55k submeshes) + ~300 dynamic entities, shadows OFF (Throne field data) | BeginFrame+EndFrame ≤ **2 ms**; ≥ **120 FPS** (baseline today ≈ 42.5 ms / 15 FPS) |
| **S2 throne-target** | 200×200 map (40k tiles) as a tilemap primitive, 100k total scene objects, **30k visible**, 2k dynamic transforms/frame, shadows ON, 1 directional light | ≥ **120 FPS avg**; p99 ≤ **12 ms**; no frame > 20 ms |
| **S3 ffi-churn** | 100k entities × 4 components; per frame 50k reads, 10k writes, 1k spawn, 1k despawn via FFI | ≤ **2 FFI crossings**; ≤ **1 ms** engine-side; **0 steady-state heap allocs** |
| **S4 sprite-storm** | 2D: 100k sprites across 16 textures, **30k visible**, scrolling camera | ≥ **120 FPS**; ≤ **16 draws**; CPU submit ≤ **1 ms** |
| **S5 stress** | 500k objects / 50k visible | Informational nightly trend (self-hosted M-series runner, ENG2-P9-10); no crash, bounded RSS, frame documented — promoted to a hard gate in a later revision |
| **H1 headless-scale** | Headless: 100k and 1M entity tick, no GPU | 100k ≤ **3 ms**; 1M ≤ **16 ms** avg / **25 ms** max |
| **cull_scaling** | Fixed 5k visible, total 10k→100k, culling ON | frame-time growth ≤ **10%** |

## Allocation & upload budgets (CI-gated via ENG2-P0-15)

- Steady-state render frame (no scene mutation, D=0) performs **0 heap allocations** in the render path.
- Per-frame bytes-uploaded budget asserted per scene; the per-frame allocation inventory (material-sort clone, 4KB/draw uniform ring, per-instance vertex clones, UI string clones) is a tracked burndown checklist.

## Why the numbers are credible

Apple M-series sustains multi-GB/s buffer bandwidth and hundreds of thousands of instanced draws/frame under Metal when submission is GPU-friendly (persistent instance buffers, one draw per (mesh, material), per-frame constants uploaded once). 120 FPS is an 8.3 ms budget; the targets above leave real headroom rather than sitting on the 60 FPS floor. S5 is deliberately informational until the architecture proves out.
