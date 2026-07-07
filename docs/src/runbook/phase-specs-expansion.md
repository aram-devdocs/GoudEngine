# ENG2 Roadmap Expansion — Perf Hardening + Testing/Infra (delta on design-technical.md)

User directives driving this delta:
1. "tens of thousands of entities at SUPER HIGH fps" → raise ambition beyond 60fps parity to headroom (120+ FPS on Apple M-series for tens of thousands of visible entities; 100k+ sim).
2. Bake in TDD, spec-driven validation, unit/integration/E2E/isolation testing, a "Storybook for the engine", Docker/reproducible infra.

This delta ADDS issues and RAISES gates. The base 91-issue structure in design-technical.md stands; below are the additions/changes. New engine total: **102 issues** (+11); Throne: **7** (unchanged). Grand total **109**.

---

## A. Recalibrated performance gates (super-high-FPS ambition)

Rationale: Apple M-series (M1/M2/M3) sustains multi-GB/s buffer bandwidth and hundreds of thousands of instanced draws/frame under Metal when submission is GPU-friendly. 60 FPS is a floor, not the goal. Targets stay physically credible (frame budget in ms) but demand real headroom.

| Scene | OLD gate | NEW gate |
|---|---|---|
| **S1 throne-baseline** (55k objects / 10k visible, shadows off) | BeginFrame+EndFrame ≤ 4 ms; ≥ 60 FPS | **BeginFrame+EndFrame ≤ 2 ms; ≥ 120 FPS** |
| **S2 throne-target** (100k objects / **30k** visible, shadows ON, was 10k visible) | ≥ 60 FPS; p99 ≤ 33 ms | **≥ 120 FPS avg; p99 ≤ 12 ms; no frame > 20 ms** |
| **S3 ffi-churn** (100k entities; 50k reads/10k writes/1k spawn/1k despawn per frame) | ≤ 3 crossings; ≤ 2 ms engine-side | **≤ 2 crossings; ≤ 1 ms engine-side; 0 steady-state heap allocs** |
| **S4 sprite-storm** (100k sprites / **30k** visible) | ≥ 60 FPS; ≤ 20 draws | **≥ 120 FPS; ≤ 16 draws; CPU submit ≤ 1 ms** |
| **H1 headless-scale** (100k / 1M entity tick) | 100k ≤ 5 ms; 1M ≤ 33 ms | **100k ≤ 3 ms; 1M ≤ 16 ms avg / 25 ms max** |
| **S5 stress** (NEW: 500k objects / 50k visible) | — | **informational trend (nightly); no crash, no unbounded RSS, frame documented — becomes a hard gate in a later revision** |

Add S5 to `perf-dod.md` scene table (defined in ENG2-P0-07). cull_scaling gate stays ≤ 10% frame growth for fixed-visible/growing-total. Every raised number updates the corresponding gate checklist in the phase doc; the acceptance criteria of ENG2-P2-12 (S1/S2 validation), ENG2-P4-08 (S3), ENG2-P5-06 (H1), ENG2-P6-08 (S4) change to the NEW numbers.

---

## B. New performance issues

### ENG2-P2-13 — SoA transform store shared between ECS and renderer (zero-copy extract)  [L] · milestone p2 · area:rendering,area:ecs,area:performance,breaking-change
The ECS→renderer handoff is the expensive seam: today `SetModelPosition` pushes per-entity transforms across FFI, and `instantiate_model` clones vertices per instance (`core_model_instances.rs:39,77`), and the renderer holds transforms in `Object3D` inside a `FxHashMap`. Design a **structure-of-arrays transform column** (position/rotation/scale + world matrix) owned once, referenced by both ECS `GlobalTransform` reads and the renderer's per-instance buffer upload. Renderer consumes dirty ranges directly (no copy, no per-entity FFI). This is the backbone of the super-high-FPS target — it turns per-object work into per-dirty-range work.
Blocked by: ENG2-P2-01 (dense storage), ENG2-P0-09 (RFC-0005). Feeds ENG2-P2-05 (instance buffers), ENG2-P4-04 (bulk transform upload writes straight into this store). Throne follow-up: THR-A-02.
Agent notes: RFC-0005 (P0-09) must enumerate this handoff design; RFC-0006 (P3-01) must make the unified component store expose a columnar transform view compatible with this. Verify wgpu buffer-mapping strategy in `backend/wgpu_backend/init.rs`.

### ENG2-P2-14 — GPU-driven culling / multi_draw_indirect path (Metal-gated stretch)  [L] · milestone p2 · area:rendering,area:performance
Evaluate and, if Metal-viable under wgpu, implement indirect/compute-driven submission for the 30k+ visible target: a compute pass culls instances and writes a draw-indirect buffer, collapsing CPU submit cost toward O(1). Check `wgpu::Features::MULTI_DRAW_INDIRECT`/`INDIRECT_FIRST_INSTANCE` support on the Metal backend and what `init.rs` requests today. If Metal cannot support it acceptably, deliver the CPU spatial-index path (P2-02) as the shipped answer and document the indirect path as a future GPU-backend capability. RFC-0005 decision item.
Blocked by: ENG2-P2-02, ENG2-P2-04. Acceptance: S2 at 30k visible meets the 120-FPS gate with CPU submit ≤ 1 ms (whichever path achieves it).

### ENG2-P5-07 — Per-frame frame-arena (bump) allocator + extract/prepare/submit split  [L] · milestone p5 · area:memory,area:performance
Introduce a bump allocator reset each frame for all per-frame scratch (visible lists, draw-command buffers, extract staging) so steady-state frames do zero heap allocations. Adopt a bevy-style **extract → prepare → submit** split so sim state is snapshotted into render-owned buffers, enabling the parallel schedule (P5-02/P5-03) to overlap sim(N+1) with render(N). This is what lets the alloc-budget gate (P0-15) pass under load.
Blocked by: ENG2-P5-01 (Send), ENG2-P0-15 (alloc harness). Acceptance: S1 steady-state (D=0) render path = 0 heap allocations, asserted by the P0-15 counter; extract/submit overlap shows ≥ 1.5× throughput on the parallel bench.

RFC scope edits (not new issues):
- **ENG2-P0-09 (RFC-0005)** decision list MUST add: CPU spatial index vs GPU compute culling; instanced draws vs multi_draw_indirect (Metal feature check); SoA transform handoff (P2-13); bind-group layout (per-frame/per-material/per-object dynamic offset); sort-key format.
- **ENG2-P3-01 (RFC-0006)** MUST require the store expose a columnar transform view usable by P2-13 with no copy.
- **ENG2-P4-01 (RFC-0007)** MUST specify bulk transform-upload ops writing directly into the P2-13 SoA store, and a 0-alloc decode path.

---

## C. Testing & validation workstream (new Phase 0 issues — regression net BEFORE the render rewrite)

These land in Phase 0 deliberately: you cannot safely rewrite the render core (P2) or unify the store (P3) without a spec/story/determinism net that catches regressions. All are `area:testing`/`area:infra`.

### ENG2-P0-13 — Docker CI image + `docker compose verify` local/CI parity  [M] · milestone p0 · area:infra
Audit the existing root `Dockerfile` + `docker-compose.yml` (verify what they do today). Build a versioned `Dockerfile.ci`: pinned `rust-toolchain.toml` toolchain + Mesa lavapipe/llvmpipe (for headless GPU + story rendering) + SDK toolchains (dotnet/node/python). Publish to GHCR; cache via sccache or cargo-chef layers. A `docker compose run verify` entrypoint runs the SAME `scripts/verify.sh` gate as CI, so local == CI. Document the Metal-only exception (real-GPU perf capture stays on the M-series host per perf-dod).
Acceptance: `docker compose run verify` reproduces the CI core gate result; image pinned + published; `scripts/check-gate-parity.py` extended to include the container entrypoint.

### ENG2-P0-14 — Scene story gallery MVP ("Storybook for the engine")  [L] · milestone p0 · area:testing,area:rendering
Build `goud_gallery`: a declarative **story** = scene setup + camera + knobs (args) + expected invariants (draw calls, instance counts, phase-timing budgets) + golden-image tag. A headless runner renders each story to a texture on the wgpu backend (no window), captures framebuffer + metrics via the existing MCP `capture_frame`/`get_metrics_trace`, perceptually diffs against a committed golden (evaluate `dssim`/`image-compare`), and checks metric budgets. CI emits a static **HTML gallery** artifact per PR (image, diff-vs-golden, metrics table, pass/fail) — the Storybook browsing experience. Reuse `benches/helpers/scene3d.rs` scene builders and the `tests/renderer3d_frame_counts.rs` count-pinning pattern. MVP catalog: a handful of core stories (sprite batch, one instancing path, one shadow config); every later render/UI/particle/animation issue ADDS stories as an acceptance item.
Acceptance: gallery runs on lavapipe in CI + real Metal locally with per-backend tolerance; golden-update protocol documented (who approves a changed golden; baselines versioned in-repo, small PNGs, or LFS if size demands). Interaction stories (scripted input via MCP `inject_input`/`replay`, assert end-state) supported as the "play function" analog.

### ENG2-P0-15 — Allocation-budget harness + steady-state zero-alloc gate  [M] · milestone p0 · area:testing,area:performance
Add a global-allocator counter (custom `GlobalAlloc` wrapper or `dhat` feature) usable in benches/tests to assert allocations-per-frame and bytes-uploaded-per-frame budgets. Wire into `bench-gate.py` so budgets are CI-gated. Establish the contract: **steady-state render frame (no scene mutation, D=0) performs 0 heap allocations**; document current baseline (expected-fail today, becomes gated as P2/P5 land). Inventory the per-frame allocation sites (audit found: material sort clone `render/mod.rs:264-282`, uniform ring 4KB/draw, per-instance vertex clones, UI string clones per frame, `finalize_batch` to_vec) as the burndown checklist.
Acceptance: alloc counter available to benches; budget assertions in CI; the per-frame-allocation inventory committed as a tracked checklist.

### ENG2-P0-16 — Spec-test convention + TDD gate  [M] · milestone p0 · area:testing
Define the issue→spec mapping: each ENG2 issue with observable behavior gets a spec test at `goud_engine/tests/spec/eng2_p<N>_<nn>_<slug>.rs`, tagged, that encodes its Acceptance Criteria as executable assertions and is the gate evidence. Extend the `tdd-workflow` skill for RED-GREEN-REFACTOR against spec tests; add to the issue template's Verification section a required "spec test path" line; add a reviewer check (spec-reviewer) that the spec exists and was red-before-green. A CI job asserts every closed `roadmap:eng2` issue with `area:performance`/behavioral scope has a corresponding spec file.
Acceptance: convention documented in the runbook; template updated; `tdd-workflow` skill extended; example spec test committed for one Phase 0 issue.

### ENG2-P0-17 — E2E SDK-driven headless harness (C#/TS/Python)  [L] · milestone p0 · area:testing,area:sdk-csharp
A harness where each first-class SDK boots a REAL headless engine, runs a scripted mini-game N frames, and asserts draw counts / world-state hash / no-crash / flat RSS. Lives under `sdks/*/e2e/` + a shared scene script format. Runs in the Docker CI image (P0-13) on lavapipe. This is the missing tier: today secondary SDK tests never touch a running engine (Swift EnumTests 36 LOC), and `validate_coverage.py` checks only manifest parity. Minimum: C#, TypeScript, Python; other SDKs keep scaffolds + the P1-04 parity gate.
Acceptance: three SDKs run a scripted headless game in required CI; assertions include a cross-SDK identical world-state hash (ties to determinism P5-04) once available.

### ENG2-P0-18 — Determinism, property & fuzz fixtures  [M] · milestone p0 · area:testing
Reusable fixtures: (a) determinism harness — run the headless sim 3 seeds × 10k ticks, compare world hashes (fixture consumed by P5-04 and Throne's determinism gate); (b) `proptest` generators for culling (visible-set correctness vs brute force), the command-buffer decoder (P4-02), and instancing grouping; (c) `cargo-fuzz` targets for the component store add/get/remove/batch and the command-buffer decoder (P4). Nightly CI lane runs fuzz with a time budget.
Acceptance: determinism fixture + at least one proptest + one fuzz target running in nightly CI, zero findings; documented so P3/P4/P5 issues plug into them.

### ENG2-P0-19 — CI restructure: tiered gate + path filters + merge queue  [M] · milestone p0 · area:infra
Today ~20 jobs (incl. 10-SDK fanout) gate every merge; shipping one fix is slow. Restructure into: **fast core gate** (fmt, clippy -D warnings, unit + isolation tests, lint-layers) always-on, < 10 min; **staged tiers** (integration, E2E P0-17, story gallery P0-14, blocking GPU lane P0-06, SDK matrix) triggered by path filters + a merge queue. New Phase 0 required gates (bench-gate P0-05, GPU lane P0-06, parity P1-04) slot into the right tier without doubling wall-clock. Coordinates with release.yml decoupling (P1-07).
Acceptance: core gate < 10 min p50; heavy tiers path-filtered; merge queue enabled; documented job graph.

### Phase 0 gate additions (append to the P0 gate checklist)
- [ ] `docker compose run verify` == CI core gate result.
- [ ] Story gallery runs in CI with ≥ 5 committed golden stories; HTML gallery artifact published per PR.
- [ ] Alloc-budget counter available and gated; per-frame allocation inventory committed.
- [ ] Spec-test convention live; issue template + tdd-workflow updated; ≥ 1 example spec committed.
- [ ] E2E harness boots C#/TS/Python against a headless engine in required CI.
- [ ] Determinism + proptest + fuzz fixtures running (nightly for fuzz), zero findings.
- [ ] CI core gate < 10 min; tiered graph + merge queue live.

---

## D. New infra issue in Phase 9

### ENG2-P9-10 — Self-hosted M-series nightly perf runner + trend dashboard  [M] · milestone p9 · area:infra,area:performance
GitHub-hosted runners are too noisy for absolute perf numbers and can't run Metal. Register the M-series dev machine as a labeled self-hosted runner (security-scoped: nightly-only, no fork PRs). Nightly pipeline runs scenes S1–S5 + benches on real Metal, captures via perf-capture (P0-07), publishes a trend dashboard (simple JSON → Pages chart) and alerts on regression beyond the bench-gate ratio threshold. This is where the "super high FPS" numbers are actually measured and defended over time.
Acceptance: nightly job runs on the self-hosted M-series runner; S1–S5 trend chart published; regression alert fires on a seeded regression test.

Phase 9 gate addition:
- [ ] Nightly M-series perf runner live; S1–S5 trend dashboard published; regression alerting verified.

---

## E. Revised issue-count table

| Phase | Milestone | Engine issues | Δ | Throne |
|---|---|---|---|---|
| P0 Instrumentation, Truth & Test Foundation | eng2-p0-truth (#10) | 19 | +7 | 1 (Sync 0) |
| P1 SDK Source of Truth | eng2-p1-sdk-truth (#11) | 10 | — | — |
| P2 Render Core v2 | eng2-p2-render-core (#12) | 14 | +2 | — |
| P3 Data Core v2 | eng2-p3-data-core (#13) | 9 | — | — |
| P4 FFI v2 Convergence | eng2-p4-ffi-v2 (#14) | 8 | — | 4 (Sync A) |
| P5 Parallelism & Determinism | eng2-p5-parallel (#15) | 7 | +1 | 1 (Sync B) |
| P6 2D Render v2 | eng2-p6-render-2d (#16) | 8 | — | — |
| P7 Runtime Services | eng2-p7-services (#17) | 6 | — | — |
| P8 Capability Gaps | eng2-p8-capabilities (#18) | 11 | — | 1 (Sync C) |
| P9 Authoring/Platforms/Polish | eng2-p9-authoring (#19) | 10 | +1 | — |
| Continuous | eng2-continuous (#20) | (carried #704/#657 + dep-dedup lives in P1-10) | — | — |
| **Total** | | **102** | **+11** | **7** → **109** |

## F. Runbook doc deltas
- `perf-dod.md`: add S5 scene; raise all gate numbers per section A; add the alloc/frame + bytes-uploaded/frame budget definitions.
- New runbook page `testing-strategy.md`: the five tiers (unit/isolation/integration/E2E/spec) + story gallery + determinism/property/fuzz, with the issue→spec mapping rule.
- Issue template Verification section: add required "Spec test:" line and "Stories added/updated:" line for render/UI issues.
- New `.agents/rules/testing-v2.md`: the spec-test convention + story-gallery authoring as agent law; extend the `tdd-workflow` and add a `story-authoring` skill.
