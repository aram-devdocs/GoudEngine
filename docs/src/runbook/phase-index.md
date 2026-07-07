# ENG2 Phase Index

The GoudEngine v2 rebuild is 10 phases + a continuous lane. Full per-issue specs (with file:line evidence) are in [phase-specs.md](phase-specs.md); the perf-hardening + testing/infra delta is in [phase-specs-expansion.md](phase-specs-expansion.md). Live issue numbers are tracked in the pinned master issue (**ENG2 master #810**).

| Phase | Milestone | Focus | Depends on | Runbook |
|---|---|---|---|---|
| 0 | `eng2-p0-truth` | Instrumentation, Truth & Test Foundation | — | [phase-0](phases/phase-0.md) |
| 1 | `eng2-p1-sdk-truth` | SDK Single Source of Truth | P0 gate | [phase-1](phases/phase-1.md) |
| 2 | `eng2-p2-render-core` | Render Core v2 (O(visible)) | P0 gate · ‖P1 | [phase-2](phases/phase-2.md) |
| 3 | `eng2-p3-data-core` | Data Core v2 (ECS + store) | P1 gate | [phase-3](phases/phase-3.md) |
| 4 | `eng2-p4-ffi-v2` | FFI v2 Convergence | P2 + P3 gates | [phase-4](phases/phase-4.md) |
| 5 | `eng2-p5-parallel` | Parallelism & Determinism | P4 gate | [phase-5](phases/phase-5.md) |
| 6 | `eng2-p6-render-2d` | 2D Render v2 | P4 gate · ‖P5,P7 | [phase-6](phases/phase-6.md) |
| 7 | `eng2-p7-services` | Runtime Services | P4 gate · ‖P5,P6 | [phase-7](phases/phase-7.md) |
| 8 | `eng2-p8-capabilities` | Capability Gaps | P5+P6+P7 gates | [phase-8](phases/phase-8.md) |
| 9 | `eng2-p9-authoring` | Authoring, Platforms, Examples | P8 gate | [phase-9](phases/phase-9.md) |
| — | `eng2-continuous` | Dependency health (no gate) | ongoing | — |

## Concurrency

```
P0  →  (P1 ‖ P2)  →  P3  →  P4 [SYNC A]  →  (P5 ‖ P6 ‖ P7) [SYNC B]  →  P8 [SYNC C]  →  P9
```

Within a phase: batches are sequential; groups within a batch run in parallel (`/gh-issue <n> --worktree`, one per group). All groups in a batch merge before the next batch.

## Gate summaries (super-high-FPS ambition — full checklists in each phase doc and [perf-dod.md](perf-dod.md))

- **P0** — GPU + CPU phase counters non-zero; `bench-gate.py` required in CI; scene harness S1–S5; story gallery; alloc-budget gate; E2E/spec/determinism/fuzz fixtures; Docker CI parity; RFC-0005 + OpenGL-removal ADR merged.
- **P1** — clean-room codegen green; compiled JNI symbols == manifest (167-method drift closed); per-SDK parity gate for all 10 SDKs; wasm coverage gated; C#/TS/Python integration tier in CI.
- **P2** — S1 BeginFrame+EndFrame ≤2 ms & ≥120 FPS; S2 (100k objects / 30k visible / shadows ON) ≥120 FPS avg, p99 ≤12 ms; 40k CreatePlane tiles ≤64 draws; cull_scaling ≤10%; #677/#678/#679 closed with captures.
- **P3** — zero global component statics; single lock per component op (bench-asserted); 100k-entity dirty propagation ≤0.5 ms; flat-RSS churn soak; pointer-escape fixed + fuzz green.
- **P4** — S3 per-frame op stream ≤2 FFI crossings, ≤1 ms engine-side, 0 steady-state allocs; fn-table entry point live; all 10 SDKs regenerate green. → **Sync A** (Throne).
- **P5** — H1: 100k tick ≤3 ms, 1M ≤16 ms avg/25 ms max; 3-seed determinism CI; parallel speedup ≥2.5×; frame limiter holds 60±1 FPS. → **Sync B**.
- **P6** — S4: 100k sprites / 30k visible ≥120 FPS, ≤16 draws, CPU submit ≤1 ms; exactly one sprite renderer remains.
- **P7** — async loads no frame >20 ms; mip chains + trilinear; one audio stack, one 2D physics; UI 1k-widget single-change relayout ≤0.3 ms; two windows.
- **P8** — nav 10k agents repath ≤5 ms; every capability ships FFI + 10-SDK parity + integration test + docs page; save/load round-trips to identical world hash; cgmath gone. → **Sync C**.
- **P9** — inspector edits + saves scene JSON; Android/iOS CI required; console typecheck lane; ≥2 example games build in CI; migration guide + agents/skills v2.

## Named perf scenes

Defined once (ENG2-P0-07), referenced by every perf gate — see [perf-dod.md](perf-dod.md): **S1** throne-baseline, **S2** throne-target (the prime-directive scene), **S3** ffi-churn, **S4** sprite-storm, **S5** stress (500k objects), **H1** headless-scale.
