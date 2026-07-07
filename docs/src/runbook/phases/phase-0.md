# Phase 0 — Instrumentation, Truth & Test Foundation

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

> **Expansion:** Phase 0 adds the testing/infra foundation (Docker CI, story gallery, alloc-budget gate, spec-test convention, E2E harness, determinism/fuzz fixtures, CI restructure) — see [phase-specs-expansion.md](../phase-specs-expansion.md) §C. Total P0 issues = 19.

---

## Phase 0 — Instrumentation & Truth (W0)

**Goal:** Measure before optimizing. Every later perf claim must be attributable to a non-zero phase counter and gated by a bench. Decision records for the two big redesigns are written here with fresh data. 12 issues.

### Batch 0.1 — Instrumentation core (3 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P0-01 | Add GPU timestamp queries (wgpu QuerySet) for shadow/render/submit phases | A | M | — |
| ENG2-P0-02 | Fix always-zero CPU phase counters and surface both timing families via FFI + MCP | A | M | P0-01 |
| ENG2-P0-03 | Bench suite expansion: cull_scaling, primitive draw-call, real-GPU shadow benches | B | L | — |
| ENG2-P0-04 | Feature-gated tracy/puffin profiler integration | C | M | — |

- **P0-01:** `timestamp_writes: None` at `backend/wgpu_backend/frame.rs:148` and `shadow_pass.rs:76,382` — no GPU timing exists; all phases are `Instant::now()` CPU record-time (`frame.rs:18-127`). Add QuerySet begin/end around shadow, main pass, submit; new `gpu_shadow/gpu_render/gpu_submit` phases distinct from CPU-record phases. This is why shadow_pass reports 0 µs while Metal stalls 300–600 ms (#677 evidence).
- **P0-02:** Consumer profile shows `uniform_upload, render_pass, gpu_submit, surface_present, shadow_build, bone_*` all 0 µs (throne profile :44-52). Wire real values through `frame_timing.rs` → `GetFramePhaseTimings` FFI → MCP `get_metrics_trace` (tools/goudengine-mcp inherits the mis-attribution today).
- **P0-03:** `benches/helpers/scene3d.rs:81` force-disables frustum culling in every bench scene; `renderer3d_frame_benchmarks.rs:29,35` scale only total count — bug #678 is structurally unmeasurable. Add: (a) `cull_scaling` group — fixed 5k visible, total 10k→100k, culling ON; (b) draw-call bench for `CreatePlane`/`CreateCube` (count-pinning test `tests/renderer3d_frame_counts.rs` already documents draw_calls == n); (c) opt-in real-wgpu shadow bench (NullBackend at `helpers/null_backend.rs:46-53` no-ops everything, so `shadow_record/casters_*` measure the wrong thing).
- **P0-04:** No tracy/puffin in any Cargo.toml. Feature-gate `profiling` crate spans in the frame path + ECS schedule so devs get flamegraphs beyond the 14 fixed counters.

### Batch 0.2 — CI wiring & scene harness (4 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P0-05 | Wire bench-gate.py into required CI; fix benchmarks.yml bench set; retire bench-gate.sh (absorbs #256) | A | M | P0-03 |
| ENG2-P0-06 | Promote curated GPU tests to a blocking lavapipe lane | B | M | — |
| ENG2-P0-07 | Scripted consumer-scale scene harness: S1/S2/S3/S4/H1 + `scripts/perf-capture` + committed S1 baseline report | C | L | P0-01, P0-02 |
| ENG2-P0-08 | Move community-stats bot off protected main; revoke admin push token | D | S | — |

- **P0-05:** `scripts/bench-gate.py` (ratio-normalized, baselines at `benches/baselines/criterion_baseline.json`) has ZERO references in `.github/`/`.husky/`/`verify.sh`; `benchmarks.yml:35` runs the wrong bench set, weekly-cron only, comparing nothing. Make it a required PR check over engine_tick + renderer3d_frame + the new P0-03 benches; delete redundant `bench-gate.sh`.
- **P0-06:** 96 `#[ignore]` GPU tests run only in `headless-gpu.yml` with `continue-on-error: true` (line 25) — renderer is untested in gating CI (`ci.yml:174-178` runs only `null`-named tests). Promote a curated subset to required.
- **P0-07:** Builds the five named scenes (table above) as reproducible harness scenarios (headless + capture via MCP `capture_frame`/`get_metrics_trace`), documents the 3-run-median M-series procedure, and commits the S1 baseline numbers as the regression anchor for the Phase 2 gate.
- **P0-08:** `community-stats.yml:5-6,20-25,40-41` — daily cron force-pushes `[skip ci]` commits to protected main with an admin token (122 commits pollute blame/bisect for perf work). Move to a data branch/Pages; drop the token.

### Batch 0.3 — Decision records & repo truth (3 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P0-09 | RFC-0005: Render submission redesign (spatial index, dense storage, instancing, uniform strategy) | A | M | P0-07 |
| ENG2-P0-10 | ADR: Remove legacy glfw/OpenGL backend; verify live backend; reconcile doc contradiction | A | S | — |
| ENG2-P0-11 | Docs truth pass: archive ALPHA_ROADMAP.md, promote performance-roadmap.md, fix ARCHITECTURE.md, adopt perf definition-of-done | B | M | — |
| ENG2-P0-12 | Issue/label/milestone hygiene + verify-close pass (#423, #670, #136, #114, #475; migrate #590–593) | C | M | — |

- **P0-09:** No RFC covers the redesign #677/#678/#679 require (docs/rfcs has only RFC-0001..0004). Must decide: grid vs BVH; dense-slot storage replacing `FxHashMap<u32, Object3D>` (`renderer3d/core/mod.rs:54-93`); dynamic-offset uniform slots; instancing-by-identity keying. Written with fresh P0-07 baseline numbers. Blocks Phase 2 Batch 2.1.
- **P0-10:** ARCHITECTURE.md says wgpu default; ALPHA_ROADMAP.md:1246-1248 says wgpu incomplete/GL bypasses abstraction. User decision: "no legacy code kept" — record removal of the legacy-glfw-opengl path as an ADR (execution is ENG2-P2-03). Confirm the active Metal path in code first.
- **P0-11:** ALPHA_ROADMAP.md:9 falsely claims "Phase 0 (v2) Core Performance — COMPLETE" while #677/#678/#679 are open. Archive it; promote `docs/src/development/performance-roadmap.md` (current, code-verified); write the perf definition-of-done ("closed only when the named scene hits the number AND the phase counter attributes the cost") into CLAUDE.md/agents + PR template.
- **P0-12:** Dedupe legacy labels (bug/enhancement/…) into `type:*`; add `phase:v2-p*`; close milestones 1–9 as superseded; retag #677-679 `area:performance`; verify-close: #423 (drawText EXISTS in wasm — `wasm/rendering.rs:239`, close invalid), #670 (`sdks/csharp/build/GoudEngine.targets` exists and packs — fix its repo-root path-preference fallback, then close), #136 (debugger substantially shipped — close, successor is P9-01), #114/#475 (superseded by new master issue); migrate #590–593 bodies to throne_ge and close GE-side.

### Phase 0 Gate
- [ ] S1 capture: `gpu_shadow`, `gpu_render`, `gpu_submit` report non-zero; CPU counters `uniform_upload/render_pass/surface_present` non-zero; MCP `get_metrics_trace` shows both families.
- [ ] bench-gate.py is a required PR check over the tracked set (engine_tick, renderer3d_frame, cull_scaling, draw-call, ffi_component); baselines committed.
- [ ] cull_scaling baseline recorded and documents the current O(total) defect (expected-fail documented, becomes gated in Phase 2).
- [ ] Blocking lavapipe lane required on PRs; curated GPU tests pass.
- [ ] S1 baseline report committed (re-measured post-#670-fix numbers).
- [ ] RFC-0005 and the OpenGL-removal ADR merged; ALPHA_ROADMAP.md archived; zombie milestones closed.

**Sync 0 (throne_ge, after gate):** `THR-S0-01` [S] — bump GoudEngine, delete the `CopyGoudEngineNativeLib` workaround (`Throne.App.csproj:33-49`), confirm the ~60x load-path overhead is gone, re-profile GameWorld with the fixed phase counters and post the report (this becomes Phase 2's consumer-side "before" evidence). Maps throne_ge #1040 partially.

---
