# Phase 5 — Parallelism & Determinism

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

> **Expansion:** Phase 5 adds ENG2-P5-07 (per-frame frame-arena allocator + extract/prepare/submit split) and raises the H1 gate — see [phase-specs-expansion.md](../phase-specs-expansion.md) §B. Total P5 issues = 7.

---

## Phase 5 — Parallelism & Determinism (W3) — *concurrent with Phases 6, 7*

**Goal:** Multicore engine loop; determinism guarantees Throne Phase 1 requires. 6 issues.

### Batch 5.1 — Send + schedule (serial)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P5-01 | Make World/GoudContext Send; eliminate remaining global-static blockers | — | L | Phase 4 gate |
| ENG2-P5-02 | Route the runtime loop through the existing ParallelSystemStage | — | M | P5-01 |

- **P5-01:** `context.rs:22` documents not-Send; all contexts behind one global Mutex (`registry.rs:346`); Phase 3 removed the component-store blockers — finish the job (thread-local window/immediate state maps at `window/state.rs:361`, `immediate.rs:367`).
- **P5-02:** A working parallel scheduler with access-conflict analysis exists (`schedule/parallel/execution.rs:85` rayon::scope) but `App::run_once` drives sequential `SystemStage` (`ecs/app/mod.rs:163-165`). Wire it; feature-flag for fallback during rollout.

### Batch 5.2 — Jobs, determinism, pacing (3 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P5-03 | Work-stealing job system for engine-internal work (propagation, culling, batch bake) | A | L | P5-01 |
| ENG2-P5-04 | Deterministic iteration order + world-state hash FFI hook | B | M | — |
| ENG2-P5-05 | Frame pacing: target-FPS limiter without vsync + fixed-timestep render interpolation | C | M | — |

- **P5-03:** Replaces Throne's `Parallel.For` stub (throne engine-dependencies.md:17); deterministic scheduling required (Throne phase-1.md:229).
- **P5-04:** Throne Phase 1 gate needs same-seed identical state after 10k ticks across thread schedules (`phases/phase-1.md:224-229`); expose `goud_world_state_hash` so the consumer's determinism test can compare runs.
- **P5-05:** No frame limiter exists (pacing is wgpu present_mode only, `init.rs:73-76`); `physics_world/interpolation.rs` is a 26-LOC stub despite the real accumulator loop in `sdk/game/instance_fixed_timestep.rs:37-48`.

### Batch 5.3 — Scale validation (serial)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P5-06 | H1 headless validation: 100k/1M published tick times, spawn/despawn soak, determinism CI job | — | M | P5-02..04 |

### Phase 5 Gate
- [ ] **H1:** 100k entities headless tick ≤ 5 ms; 1M ≤ 33 ms avg / 50 ms max; report committed.
- [ ] Determinism: 3 seeds × 10k ticks ⇒ identical world hash, parallel stage ON, CI-gated.
- [ ] Parallel propagation+culling ≥ 2.5× speedup on 8 cores vs single thread (bench).
- [ ] Frame limiter holds 60 ± 1 FPS on S1 with vsync off.

**══ SYNC B (throne_ge):** `THR-B-01` [M] — replace the Parallel.For/job stubs with the engine job system; wire `goud_world_state_hash` into `DeterminismTests`; re-run Throne Phase 1 scale gates against engine primitives.

---
