
# GoudEngine v2 Technical Roadmap — Phases, Batches, Groups, Gates, and Issue Decomposition

**Status:** Design for approval. On approval this becomes `docs/runbook/v2/phase-index.md` + `docs/runbook/v2/phases/phase-N.md` in aram-devdocs/GoudEngine, one GitHub milestone per phase, one issue per ENG2 ID below, a new master tracking issue (superseding #475/#114), and 7 issues in aram-devdocs/throne_ge at the marked sync points.

**Prime directive (from binding user decisions):** Throne (colony-sim, C#) at 60 fps with 10k+ visible entities and 100k+ sim entities; engine per-frame cost O(visible), not O(total). Full 10-SDK parity kept but made cheap and real. No legacy code retained where we refactor; breaking changes allowed, Throne adopted afterward via filed throne_ge issues. All platform targets kept and made CI-real.

**Total issue count: 98** — 91 GoudEngine issues across 10 phases + 7 throne_ge adoption issues at 4 sync points. (Within the 60–100 target.)

---

## 1. Structure and Concurrency Model

The runbook mirrors the throne_ge convention (`docs/runbook/phase-index.md`): **phases → sequential batches → parallel groups**, all-merge-before-next-batch, per-issue self-contained specs (Architecture Context / Scope checklist / Acceptance Criteria / Files Likely Touched / Agent Notes with file:line evidence), and explicit `/gh-issue <n> --worktree` run commands per batch.

One deliberate extension over the throne_ge convention: **two concurrency-eligible pairs of phases**, because genuine parallelism exists between file-disjoint workstreams and forcing a linear chain would idle agent capacity:

```
Phase 0  Instrumentation & Truth            (W0 + W8-docs + RFCs)     ── everything blocks on this
   │
   ├── Phase 1  SDK Single Source of Truth   (W6-core)   ─┐  CONCURRENT PAIR A
   └── Phase 2  Render Core v2               (W1)        ─┘  (disjoint: codegen/ sdks/ src/jni src/wasm  vs  libs/graphics/ ffi/renderer3d/)
   │
   Phase 3  Data Core v2 (ECS unification)   (W2-store)  — requires Phase 1 gate; MAY overlap Phase 2 Batches 2.2+
   │                                                       (disjoint: ecs/ ffi/component* context_registry vs render files)
   Phase 4  FFI v2 Convergence               (W2-ffi)    — requires Phase 2 AND Phase 3 gates
   │        ══ SYNC A: Throne adoption (4 throne_ge issues) — the prime-directive validation ══
   │
   ├── Phase 5  Parallelism & Determinism    (W3)        ─┐
   ├── Phase 6  2D Render v2                 (W4)        ─┼─ CONCURRENT TRIPLE B (pairwise-disjoint:
   └── Phase 7  Runtime Services             (W5)        ─┘  ecs/schedule vs 2D render stack vs assets/ui/audio/platform)
   │        ══ SYNC B: Throne job-system/determinism adoption (1 throne_ge issue) ══
   │
   Phase 8  Capability Gaps                  (W7)        — requires 5+6+7 (nav needs jobs+spatial; 2D particles need sprite core; streaming needs async pipeline)
   │        ══ SYNC C: Throne capability evaluation (1 throne_ge issue) ══
   │
   Phase 9  Authoring, Platforms, Examples   (W8 + W6-platform)
```

Sequencing decision for the W1/W2 overlap (required by the task): **render core (Phase 2) starts first and owns the critical path**; data core (Phase 3) starts as soon as the SDK-truth gate (Phase 1) passes — codegen must be healthy *before* any FFI-surface rewrites, or every W2 deliverable widens the 167-method JNI drift and the 62/682 wasm gap. The two touchpoint items that need both (bulk transform upload, command-buffer draining renderer ops) are deliberately pulled out of both phases into **Phase 4: Convergence**.

**Within a concurrency pair, each phase still runs its own batches sequentially.** No batch in any phase starts until Phase 0's gate passes.

### Issue conventions
- **ID scheme:** `ENG2-P<phase>-<nn>` (GoudEngine), `THR-<sync>-<nn>` (throne_ge). GitHub titles: `ENG2-P2-08: Auto-instance primitives by (mesh,material) identity`.
- **Effort:** S ≈ small focused PR, M ≈ one full `/gh-issue` agent run, L ≈ one large run or two chained PRs. No XL issues remain (all split).
- **Milestones:** `v2-phase-0` … `v2-phase-9` in GoudEngine. Labels: existing `type:*`/`area:*`/`priority:*` plus new `phase:v2-p{0..9}`; legacy label dedup handled in P0-12.
- **Every breaking-change issue names its Throne adoption follow-up** in its body (the THR issue ID from the sync tables below).
- **Issue body template (all issues):** Architecture Context → Scope (checklist) → Acceptance Criteria (numeric where perf) → Files Likely Touched → Agent Notes (the file:line evidence bullets below go here verbatim).

### Perf gate measurement procedure
CI runners cannot measure Metal. Two enforcement layers:
1. **CI (blocking):** `scripts/bench-gate.py` ratio-normalized regression gate on the tracked bench set (P0-05); blocking lavapipe GPU-test lane (P0-06); count-pinning tests (draw calls, culled counts).
2. **Phase gate sign-off (manual, scripted):** `scripts/perf-capture` harness (P0-07) runs the named scene on the Apple M-series dev machine, 3-run median, and the capture report is posted on the phase tracking issue before the gate box is checked.

### Named scripted scenes (defined once, in P0-07; referenced by every perf gate)

| Scene | Definition | Purpose |
|---|---|---|
| **S1 throne-baseline** | Replica of Throne field data: 6,400 terrain planes + 1,431 model instances (~55k submeshes) + 300 dynamic entities, shadows OFF (per `profiling/profile-GameWorld-20260420-224800.md` + throne_ge `Game3DSceneRenderer.Terrain.cs:8-16`) | Regression anchor; today: BeginFrame 21.7 ms + EndFrame 20.8 ms + 41 ms overhead ≈ 15 FPS |
| **S2 throne-target** | 200×200 map (40k tiles) as tilemap primitive, 100k total scene objects, 10k visible, 2k dynamic transform updates/frame, shadows ON, 1 directional light | The prime-directive scene |
| **S3 ffi-churn** | 100k entities × 4 components; per frame: 50k component reads, 10k writes, 1k spawns, 1k despawns via FFI | Data/FFI core gate |
| **S4 sprite-storm** | 2D: 100k sprites across 16 textures, 10k visible, scrolling camera | 2D core gate |
| **H1 headless-scale** | Headless: 100k and 1M entity tick, no GPU | Parallelism/determinism gate; feeds Throne Phase 1 |

---

## 2. Phase 0 — Instrumentation & Truth (W0)

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

## 3. Phase 1 — SDK Single Source of Truth (W6-core) — *concurrent with Phase 2*

**Goal:** Parity across 10 SDKs becomes cheap (one generated source of truth) and real (CI-gated per SDK, integration-tested). Must gate before Phase 3 so FFI rewrites propagate cleanly. 10 issues.

### Batch 1.1 — Fix the broken generators (2 groups)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P1-01 | Fix JNI codegen dual-file bug; add CI assert compiled-JNI-symbols == manifest | A | L | — |
| ENG2-P1-02 | Upgrade jni crate 0.21 → 0.22 (#657) | B | S | — |

- **P1-01:** `jni/mod.rs:10-11` compiles ONLY `generated.g.rs` (421 symbols, frozen 2026-04-02) while `codegen/gen_jni.py:21,2275` writes orphaned `generated.rs` (588 symbols) — Kotlin/Android ships missing 167 methods incl. all frame-timing FFI. Point generator at the compiled file, delete the orphan, fix `scripts/check-rs-line-limit.sh:25` + `tools/lint_layers.rs:286` which reference the dead file, and add the CI symbol-parity assert.
- **P1-02:** Straight dependency upgrade; do after P1-01 so regen is trustworthy.

### Batch 1.2 — Single source of truth (2 groups)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P1-03 | Generate ffi_mapping.json from the Rust manifest; delete the 30-entry `_TYPE_ALIASES` drift table | A | M | — |
| ENG2-P1-04 | Per-SDK export-parity CI gate: diff each generated SDK against the manifest | A | M | P1-03 |
| ENG2-P1-05 | Bring wasm/web bridge under manifest codegen + coverage gate; wasm-pack modernization (#425) | B | L | P1-03 |

- **P1-03:** Today three sources: auto-extracted `ffi_manifest.json` (669 fns) vs hand-maintained `ffi_mapping.json` + 430KB `goud_sdk.schema.json`; `validate_coverage.py` papers over drift with a hardcoded 30-entry alias map. Normalize types at extraction, make the manifest the sole root.
- **P1-04:** `validate_coverage.py` never inspects generated SDK outputs — 671 C# DllImports vs 669 manifest fns is ungated. Promote the `sdk-parity-check` skill concept into `codegen.sh` + required CI for all 10 SDKs.
- **P1-05:** `src/wasm/*.rs` (3.9k LOC) is hand-written, 62/682 exports covered; `gen_ts_web.py` already exists. Generate the wasm bridge from the manifest, enforce coverage (explicit exclusion list for GPU-inapplicable exports), and replace deprecated wasm-pack tooling (#425).

### Batch 1.3 — SDK reality & release cost (5 groups)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P1-06 | Engine-integration test tier: C#/TS/Python E2E against headless engine (absorbs #137) | A | L | — |
| ENG2-P1-07 | Decouple 9-registry publishing from patch bumps; per-ecosystem dispatch | B | M | — |
| ENG2-P1-08 | Make the Rust SDK real (replace 10-line published stub) | C | M | — |
| ENG2-P1-09 | Decision + execute: #[goud_api] macros as generator of no_mangle surface, or archive | D | M | P1-03 |
| ENG2-P1-10 | Dependency health: #704 security deps + RUSTSEC ignored-advisory burndown + dep dedup | E | M | — |

- **P1-06:** Secondary SDK tests are scaffold-only (Swift EnumTests 36 LOC; Kotlin type tests) — never touch a running engine. Add an integration tier (headless engine, spawn/query/draw-count assertions) for C#/TS/Python minimum; other SDKs keep scaffolds + parity gate.
- **P1-07:** `release.yml` is 801 lines publishing to 9 registries on every patch (version 0.0.841); `release-please-config.json` pins 20 files. Publish only on explicit tag/dispatch per ecosystem.
- **P1-08:** `sdks/rust` is a 10-line lib.rs published to crates.io. Wire it to the real native `sdk/` module surface with tests, or re-export the engine crate SDK — no stub publishing.
- **P1-09:** `goud_engine_macros` `#[goud_api]` is a third codegen mechanism across 21 files. Decision record + execution: either it *generates* the `#[no_mangle]` surface feeding the manifest (reducing hand-written FFI), or it's archived. Cannot stay ambiguous into Phase 4's fn-table work.
- **P1-10:** 8 ignored RUSTSEC advisories in deny.toml incl. anyhow UB; #704's dependency list; duplicate crate versions.

### Phase 1 Gate
- [ ] Clean-room `./codegen.sh` green; compiled JNI symbol set == manifest (drift = 0, was 167).
- [ ] Per-SDK parity gate required in CI for all 10 SDKs; C# DllImport count == manifest count.
- [ ] wasm/web coverage == manifest minus documented exclusions, CI-gated (was 62/682).
- [ ] C#/TS/Python integration tests run a headless engine in required CI.
- [ ] A patch version bump publishes zero registries without explicit dispatch.
- [ ] RUSTSEC ignore list ≤ 2 entries, each with a written justification.

---

## 4. Phase 2 — Render Core v2 (W1) — *concurrent with Phase 1*

**Goal:** Per-frame cost O(visible); instancing by default; shadows usable. Closes #677/#678/#679. This is the highest-impact phase in the roadmap. 12 issues.

### Batch 2.1 — Foundations from RFC-0005 (2 groups)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P2-01 | Dense scene storage: slot arrays + generational handles replacing FxHashMap iteration | A | L | P0-09 |
| ENG2-P2-02 | Scene spatial index; move culling upstream of encode so culled objects pay zero per-frame cost | A | L | P2-01 |
| ENG2-P2-03 | Remove the legacy glfw/OpenGL backend (execute ADR from P0-10) | B | L | P0-10 |

- **P2-01:** `renderer3d/core/mod.rs:54-93` — objects/instanced_meshes/lights/materials/models all `FxHashMap<u32, …>`; frame scan iterates hash maps. Replace with dense Vec-backed slots + generation-checked handles per RFC-0005.
- **P2-02:** Consumer evidence: `SetFrustumCullingEnabled` is ON yet Throne still needs the 80×80 terrain window because culled objects still pay BeginFrame/EndFrame cost (throne `Game3DSceneRenderer.Terrain.cs:9-14`; overhead ~41 ms regardless of zoom). Cull against the spatial index BEFORE transform/uniform/encode work. Primary fix for #678; validated by the cull_scaling bench (P0-03).
- **P2-03:** Delete the GL backend impl + glfw platform path per ADR; the `RenderBackend` trait seam stays. Frees all later work (instancing, dynamic offsets, timestamps) from dual-backend constraints. Names Throne follow-up: none (Throne is on wgpu/Metal).

### Batch 2.2 — Upload rearchitecture (2 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P2-04 | Uniform v2: per-frame constants uploaded once; per-object data in dynamic-offset slots; batched buffer writes | A | L | — |
| ENG2-P2-05 | Persistent dirty-tracked per-instance transform buffers | A | M | P2-04 |
| ENG2-P2-06 | Material sort caching + retained draw list (re-sort only on scene mutation) | B | M | — |
| ENG2-P2-07 | Incremental static batching: no full re-bake on change; no CPU vertex pre-transform | B | L | — |

- **P2-04:** `backend/wgpu_backend/uniforms.rs:78-92` — every draw command appends the full 4KB shader staging block into a ring and `write_buffer`s it (`uniforms.rs:188,211`). Split per-frame constants (camera, lights — uploaded once) from per-object data (dynamic-offset bind group slots); coalesce writes into one upload per frame.
- **P2-05:** `render_instanced.rs:28` clones `instanced_uniforms` per frame; instance buffers rebuilt wholesale. Persist per-instance buffers keyed by handle, upload only dirty ranges.
- **P2-06:** Frame scan re-sorts materials each frame; retain sorted draw list, invalidate on add/remove/material-change only.
- **P2-07:** `core_static_batch.rs:27-35` `rebuild_static_batch` re-bakes everything when `static_batch_dirty`; `core/mod.rs:159` pre-bakes *CPU-transformed vertices* into one VBO. Move to per-chunk static batches with GPU transforms; incremental add/remove.

### Batch 2.3 — Instancing everywhere & shadows (2 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P2-08 | Auto-instancing by (mesh,material) identity incl. CreatePlane/CreateCube; make instancing_enabled real or remove; stop per-instance vertex clones (#679) | A | L | P2-04 |
| ENG2-P2-09 | Instanced shadow pre-pass | B | M | P2-08 |
| ENG2-P2-10 | Persistent shadow target + caster caching; kill the 300–600 ms Metal stall (#677) | B | L | P0-01 |

- **P2-08:** `ffi/renderer3d/primitives.rs:47-176` creates one discrete object per call while `core_primitives.rs:58` `create_instanced_primitive` exists unused by that path; `renderer3d/config.rs:52` `instancing_enabled` defaults true but changes nothing for primitives. Key draw submission on (mesh id, material id) and batch automatically. Throne follow-up: THR-A-02.
- **P2-09:** Shadow pass currently draws casters individually (`shadow_pass.rs:348` per-object write_buffer). Render instanced groups instanced in the shadow pre-pass too.
- **P2-10:** Consumer note (throne `Game3DSceneRenderer.cs:221-230`): shadows disabled engine-wide due to 300–600 ms/frame stalls at 1.4k instances; root-cause with the new gpu_shadow timings (P0-01), keep the shadow RT/pipelines persistent across frames, cache the caster list with dirty invalidation. Throne follow-up: THR-A-04.

### Batch 2.4 — Scene-scale primitives & gate validation (2 groups)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P2-11 | Chunked/scrollable 3D tilemap-grid primitive (windowless terrain) | A | L | P2-07, P2-08 |
| ENG2-P2-12 | O(visible) validation on S1/S2; fix residual per-object frame costs; close #678 | B | M | all P2 |

- **P2-11:** Throne destroys/recreates thousands of planes on every 12-tile recenter (`Terrain.cs:45-78`) because no grid primitive exists. One handle = one tile grid (N×M, per-tile material/UV), chunked static batches, scroll/rebind without destroy/create. Throne follow-up: THR-A-03.
- **P2-12:** Run S1/S2 + cull_scaling; attribute any remaining O(total) cost with the P0 counters; fix stragglers (per-object hashmap touches, per-frame allocs). Closing PR posts the capture report on #678.

### Phase 2 Gate (M-series, `scripts/perf-capture`, 3-run median)
- [ ] **S1:** BeginFrame+EndFrame ≤ 4 ms combined (baseline ~42.5 ms); sustained ≥ 60 FPS.
- [ ] **S2** (100k objects, 10k visible, shadows ON): ≥ 60 FPS average; p99 frame ≤ 33 ms; no frame > 50 ms.
- [ ] **cull_scaling** (CI-gated from now on): fixed 5k visible, total 10k→100k ⇒ frame-time increase ≤ 10%.
- [ ] **Draw-call bench:** 40k CreatePlane tiles ⇒ ≤ 64 draw calls (baseline ~11k+).
- [ ] **Shadows:** S2 shadows-ON vs OFF total-frame delta ≤ 25%; `gpu_shadow` ≤ 4 ms and non-zero.
- [ ] #677, #678, #679 closed with capture links satisfying the perf definition-of-done.

---

## 5. Phase 3 — Data Core v2: ECS & Store Unification (W2-store) — *requires Phase 1 gate; may overlap Phase 2 Batches 2.2+*

**Goal:** One World-owned component store; one lock; sound FFI reads; dirty transforms. 9 issues.

### Batch 3.1 — Decision record

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P3-01 | RFC-0006: Unified World-owned component store + locking model | — | M | Phase 1 gate |

- Today three stores: typed World SparseSets (`ecs/world/mod.rs:118-146`), `ffi/component/storage.rs`, and `component_ops/storage.rs:265` global `CONTEXT_COMPONENT_STORAGE` — two near-identical ~400-LOC unsafe byte stores, entities in World but FFI component bytes in a disjoint global side-table. RFC decides: type-erased columnar view over World SparseSets vs World-owned byte columns; lock granularity; FFI pointer contract.

### Batch 3.2 — Store unification (serial chain + soundness group)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P3-02 | Contiguous strided Vec\<u8\> component storage (replace Vec<*mut u8>, one malloc per component) | A | L | P3-01 |
| ENG2-P3-03 | World-owned store: delete CONTEXT_COMPONENT_STORAGE + COMPONENT_TYPE_REGISTRY globals | A | L | P3-02 |
| ENG2-P3-04 | Single context-borrow locking; batch-first internals; batch/single liveness-validation parity | A | M | P3-03 |
| ENG2-P3-05 | Fix component_get/get_mut raw-pointer-escapes-mutex soundness (copy-out or pinned arena) | B | M | P3-03 |

- **P3-02:** `component_ops/storage.rs:38` `data: Vec<*mut u8>` with `alloc(layout)` per insert (:123-136) — 100k entities × N types = 100k+ mallocs, pointer-chasing, and `goud_component_get_all` returns scattered pointers (`ffi/component/query.rs:247-261`). Dense strided column ⇒ base pointer + stride for bulk reads.
- **P3-03:** `single_ops.rs:100-114` writes bytes into the global map keyed by `context_key` (`helpers.rs:17`) after checking liveness on the World — dual source of truth; context destroy orphans storage. Store moves into `GoudContext`/World; purge paths become structural.
- **P3-04:** 3 global Mutex acquisitions per single op (`single_ops.rs:70,92,106`, repeated at :182-323) = 120k lock ops/frame at 40k entities; batch path skips liveness entirely (`batch_ops.rs:84-101`). One borrow per op, one per batch; identical validation semantics both paths.
- **P3-05:** `ffi/component/access.rs:206` returns `*const u8` into mutex-owned memory after the guard drops — use-after-free class. Per RFC: copy-out into caller buffer, or stable-address arena + debug-enforced single-thread contract.

### Batch 3.3 — Systems & hardening (3 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P3-06 | Dirty-tracked transform propagation via existing changed_ticks; iterative subtree walks | A | M | — |
| ENG2-P3-07 | Extract physics/collision/broad_phase out of ecs/; dedupe 2D/3D transform module pairs | B | L | — |
| ENG2-P3-08 | Despawn/teardown purge verification + leak canary CI (absorbs #273) | C | M | P3-03 |
| ENG2-P3-09 | FFI soundness hardening: unsafe audit (#270) + fuzz harness (#267) over the new store | C | M | P3-05 |

- **P3-06:** `propagate3d.rs:58-128` walks EVERY archetype/entity every frame, allocating scratch Vecs, recursing subtrees (`:140-169`) — O(total) regardless of culling, despite `added_ticks/changed_ticks` existing (`sparse_set/core.rs:103-106`). Matches the BeginFrame-scales-with-total complaint from the ECS side.
- **P3-07:** ecs/ bundles collision (2906 LOC), broad_phase (1193), physics_world (1237) plus near-parallel transform/transform2d, global_transform/2d, propagate2d/3d pairs. Extract to sibling modules; unify propagation over a dimension-generic core.
- **P3-08:** The audit **refuted** the despawn-leak claim (FFI single+batch despawn purge both stores) — verify the remaining suspects only: recursive despawn and context-teardown purge of the (now World-owned) store; add churn canary (10k frames × 1k spawn/despawn, assert flat RSS) to CI.
- **P3-09:** `RawComponentStorage`'s hand-written `unsafe impl Send/Sync` (`storage.rs:51-56`) was justified by the global mutex being deleted here — re-audit all unsafe in the new store; add cargo-fuzz targets for component add/get/remove/batch and entity lifecycle FFI.

### Phase 3 Gate
- [ ] Zero global component statics remain (`CONTEXT_COMPONENT_STORAGE`, `COMPONENT_TYPE_REGISTRY` deleted); lint/grep gate added.
- [ ] `ffi_component_benchmarks`: single component op takes exactly 1 lock (bench-asserted); `component_get_all` at 100k returns base-pointer+stride, ≥ 2x baseline throughput.
- [ ] Transform propagation: 100k entities, 1% dirty ⇒ ≤ 0.5 ms (new bench, CI-gated).
- [ ] Soak: 10k frames × 1k spawn/despawn ⇒ RSS flat ±1%; teardown leaves no orphaned storage.
- [ ] Pointer-escape fixed with regression test; fuzz targets running in nightly CI with zero findings; miri/asan lane green on component tests.

---

## 6. Phase 4 — FFI v2 Convergence (W2-ffi) — *requires Phase 2 AND Phase 3 gates*

**Goal:** The FFI stops being a per-object, per-call surface: command buffer + fn-table + bulk APIs, propagated to all 10 SDKs. 8 issues.

### Batch 4.0 — Decision record

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P4-01 | RFC-0007: Command-buffer FFI + versioned fn-table entry point | — | M | P2/P3 gates |

- 682 flat `#[no_mangle]` exports (+1347 generated JNI fns) with no versioning or bulk submission (`ffi/` audit). RFC decides opcode format, shared-memory ownership, drain semantics (one lock per frame), fn-table versioning (GDExtension-style), and the deprecation path for per-object exports (binding: no compat shims long-term).

### Batch 4.1 — Core mechanisms (2 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P4-02 | Command-buffer FFI core: packed opcode buffer, engine drains under one lock per frame | A | L | P4-01 |
| ENG2-P4-03 | Versioned fn-table entry point; flat exports become generated veneer over it | B | L | P4-01 |

- **P4-02:** Target 10k–50k ops per frame with O(1) FFI crossings. Ops: component write/read-request, transform set, spawn/despawn, draw-submit. Batch APIs (`component/batch.rs:72,145-172`) already prove the amortization pattern.
- **P4-03:** Single `goud_get_interface(version)` returning a fn-pointer struct; codegen (healthy after Phase 1) regenerates SDK bindings against it; ABI version negotiation test.

### Batch 4.2 — Bulk surfaces (3 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P4-04 | Bulk transform-upload API: SoA handle/position/rotation arrays in one call, renderer-side dirty integration | A | M | P4-02 |
| ENG2-P4-05 | Batch-first surface: CreatePlane/CubeBatch, batched RPC drain, demote per-object hot calls | B | M | P4-02 |
| ENG2-P4-06 | Spawn/despawn churn fast path + cross-FFI object pooling | C | M | P3 gate |

- **P4-04:** Throne calls `SetModelPosition`/`SetModelRotation` per entity per frame (throne `Entities.cs:56,287`); batch equivalents exist but were never the primary path. One call updates N instances; feeds P2-05 persistent instance buffers. Throne follow-up: THR-A-02.
- **P4-05:** SDK shape encourages the 40k-call pattern (`GoudGame.g.cs:676,682` CreateCube/CreatePlane 1:1). Add batched creates; networking drain returns all pending messages per call (consumer audit: "RPC drain 1 msg/call"); per-object exports marked deprecated in manifest metadata.
- **P4-06:** Engine-side freelist/pool for entity+component slots across FFI churn — replaces Throne's ConcurrentBag stub (throne engine-dependencies.md:15).

### Batch 4.3 — Propagation & validation (serial)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P4-07 | Propagate FFI v2 through codegen to all 10 SDKs + integration tests | — | M | P4-02..06 |
| ENG2-P4-08 | S3 ffi-churn gate validation + locks/allocs-per-op CI budget benches | — | M | P4-07 |

### Phase 4 Gate
- [ ] **S3:** the full per-frame op stream (50k reads, 10k writes, 1k spawn, 1k despawn) executes in ≤ 3 FFI crossings and ≤ 2 ms engine-side; RSS flat over 10k frames.
- [ ] fn-table entry point live; version-negotiation test passes; flat exports generated from the same manifest.
- [ ] All 10 SDKs regenerate green; parity gates pass; C#/TS/Python integration tier exercises command buffer + bulk transforms.
- [ ] locks-per-op and allocs-per-op budgets asserted in CI benches.

### ══ SYNC A — Throne adoption (filed in throne_ge; this validates the prime directive) ══

| ID | Title | Effort | Blocked by |
|---|---|---|---|
| THR-A-01 | Bump GoudEngine to v2 surface; migrate breaking API changes; re-profile (supersedes GE#596; maps throne_ge#1040 and GE#590–593 content) | M | Phase 4 gate |
| THR-A-02 | Adopt batch spawn + bulk transform upload + command-buffer submission; delete per-entity SetModelPosition/SetModelRotation loops | M | THR-A-01 |
| THR-A-03 | Replace the 80×80 terrain window with the engine chunked tilemap-grid primitive (full 200×200 map) | M | THR-A-01 |
| THR-A-04 | Re-enable shadows; validate the prime-directive gate in Throne itself | S | THR-A-02, THR-A-03 |

**Sync A acceptance (posted on both repos' tracking issues):** Throne GameWorld, full 200×200 map, no terrain window, shadows ON: ≥ 60 FPS on M-series; BeginFrame+EndFrame ≤ 4 ms; grep gate shows zero per-entity transform-push loops.

---

## 7. Phase 5 — Parallelism & Determinism (W3) — *concurrent with Phases 6, 7*

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

## 8. Phase 6 — 2D Render v2 (W4) — *concurrent with Phases 5, 7*

**Goal:** Four sprite renderers become one instanced core; 2D camera and culling move engine-side. 8 issues.

### Batch 6.0 — Decision record

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P6-01 | RFC-0008: Unified instanced 2D sprite core | — | S | Phase 4 gate |

- Four parallel renderers today: immediate FFI (~15 GL-state calls/sprite, `ffi/renderer/draw/internal.rs:107-132`), batched FFI (full rebuild/sort/upload per call, `draw/batch.rs:171-343`), retained ECS SpriteBatch (unreachable from FFI, `rendering/sprite_batch/batch.rs`), WASM batcher (`wasm/sprite_renderer/renderer_core.rs`). Zero GPU instancing anywhere — every sprite CPU-expanded to 4 vertices.

### Batch 6.1 — Core

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P6-02 | Instanced sprite core: static unit quad + dirty-tracked per-instance buffer (incl. flip flags, #424) | — | L | P6-01 |

- Per-instance {transform, size, uv-rect, color, tex-index, flags}; persistent buffers, changed-only uploads (replaces `batch.rs:234-242` CPU corner rotation + `batch.rs:355-417` per-frame vertex gen). Reuses Phase 2 instancing patterns behind the backend trait.

### Batch 6.2 — Path unification (3 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P6-03 | Route FFI sprite paths through the core; delete immediate + rebuild-per-call paths | A | M | P6-02 |
| ENG2-P6-04 | Unify retained ECS SpriteBatch + WASM batcher onto the core; delete duplicates | B | M | P6-02 |
| ENG2-P6-05 | First-class 2D camera (view+proj in shader) + engine-side 2D culling + coordinate-origin option (#554) | C | M | — |

- **P6-05:** FFI batch sets only `u_viewport` (`draw/batch.rs:355`); Throne does world→screen math + visibility in C# per sprite (`GoudRenderPort.cs:275`). Camera uniform + grid/quadtree culling before submission; #554's configurable origin lands in the camera.

### Batch 6.3 — 2D scale features (3 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P6-06 | Chunked tilemap renderer from TileLayer gids | A | M | P6-02 |
| ENG2-P6-07 | Auto atlas/texture-array integration with the sprite core | B | M | P6-02 |
| ENG2-P6-08 | Shaped-text run cache + S4 sprite-storm gate validation | C | M | P6-03 |

- **P6-06:** `assets/loaders/tiled_map/layer.rs:22` TileLayer is raw `Vec<u32>` gids with no renderer — the direct cause of per-tile DrawSprite in 2D consumers.
- **P6-07:** ShelfPacker + full atlas FFI exist (`rendering/texture_atlas/packer.rs`, `ffi/renderer/atlas/ffi.rs`) but nothing auto-packs; collapse many textures into few draws without consumer bookkeeping.
- **P6-08:** `ffi/renderer/text/draw_impl.rs:45` re-shapes every string every frame (atlas is cached, shaping is not); cache runs keyed (string, font, size).

### Phase 6 Gate
- [ ] **S4:** 100k sprites / 10k visible ≥ 60 FPS; ≤ 20 draw calls; CPU submit ≤ 2 ms.
- [ ] Exactly one sprite renderer remains (lint gate: immediate path + duplicate batchers deleted; wasm routes through the core).
- [ ] 2D example runs with zero consumer-side world→screen math.
- [ ] 200×200 tile layer renders in ≤ 4 draw calls.

---

## 9. Phase 7 — Runtime Services (W5) — *concurrent with Phases 5, 6*

**Goal:** The built-but-unwired services become real: async assets, mipmaps, one audio stack, one 2D physics, retained UI, multi-window. 6 issues.

### Batch 7.1 — Assets & media (3 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P7-01 | Wire async asset pipeline into the runtime; background decode incl. RGB→RGBA off main thread | A | M | Phase 4 gate |
| ENG2-P7-02 | Mipmap generation + trilinear sampling in the wgpu texture path | B | M | — |
| ENG2-P7-03 | Unify the dual rodio audio stacks onto the provider architecture | C | M | — |

- **P7-01:** `assets/server/async_operations.rs:44-104` (load_async + process_loads) has ZERO non-test native callers; the live path is blocking `operations.rs:14-53` incl. per-pixel RGB→RGBA on the main thread (`texture.rs:59-70`) — violates assets/CLAUDE.md's own rule. Call `process_loads()` once per frame in the instance runtime; decode on rayon.
- **P7-02:** `mip_level_count: 1` everywhere (`texture.rs:41`, `init.rs:160`, sdl/xbox init) with `MipmapFilterMode::Nearest` and no chain — aliasing + wasted bandwidth at distance.
- **P7-03:** `assets/audio_manager/` (2832 LOC) vs `libs/providers/impls/rodio_audio.rs:118-138` — two full rodio wrappers, two unsafe Send/Sync blocks; keep the provider-trait one, bridge asset data through it.

### Batch 7.2 — Physics, UI, windowing (3 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P7-04 | Consolidate 2D physics on rapier2d; delete the custom ECS solver | A | M | P3-07 |
| ENG2-P7-05 | UI v2: retained command buffer, dirty-subtree relayout, string interning; absolute positioning (#553) | B | L | — |
| ENG2-P7-06 | Multi-window support (WindowId-keyed window collection) | C | M | — |

- **P7-04:** Custom solver (`ecs/physics_world/simulation.rs`, 397 LOC) coexists with the rapier2d provider — divergence risk; rapier2d becomes canonical ("no legacy code kept").
- **P7-05:** `manager/render.rs:12-20` rebuilds the full command Vec + clones label/font/path strings per frame; single global `layout_dirty` flag relayouts the whole tree (`layout.rs:16-33`); HashMap node storage (`manager.rs:44`). Retain commands, dirty subtrees, interned strings/slotmap; ship #553's absolute-positioning mode as part of the layout rework.
- **P7-06:** `winit_platform.rs:37-48` holds one `Option<Arc<Window>>`; needed for Phase 9 editor/tools.

### Phase 7 Gate
- [ ] Load burst (100 × 2048² textures) during S1 ⇒ no frame > 20 ms.
- [ ] Textures upload with full mip chains (mip_level_count > 1 asserted in test); trilinear default.
- [ ] Exactly one audio stack and one 2D physics backend in-tree.
- [ ] UI: 1k-widget tree, single label change ⇒ relayout+rebuild ≤ 0.3 ms (bench).
- [ ] Two windows render simultaneously in an example.

---

## 10. Phase 8 — Capability Gaps (W7) — *requires Phases 5+6+7*

**Goal:** "Any game" credibility: navigation, particles, save/load, procedural primitives, terrain. Every capability ships FFI + 10-SDK parity + integration test + docs page (per the Phase 1 gates, now cheap). 11 issues.

### Batch 8.1 — Highest-demand capabilities (4 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P8-01 | Grid A* + flow fields reusing ecs/spatial_grid; nav FFI surface (#546) | A | L | P5-03 |
| ENG2-P8-02 | Particles: FFI exposure of the existing 3D emitter + new 2D emitter on the sprite core | B | M | P6-02 |
| ENG2-P8-03 | World snapshot save/load (serde over the component registry) | C | L | P3 gate |
| ENG2-P8-04 | Seedable RNG + noise crate with FFI exports (#548) | D | S | — |

- **P8-01:** No navigation module exists anywhere (grep-verified; confirms #546); reuse `ecs/spatial_grid/core.rs`; flow fields align with Throne's hierarchical-pathfinding plan (throne phase-1.md:138-143).
- **P8-02:** `ParticleEmitter` fully implemented (`libs/graphics/renderer3d/types.rs:107-278`, rendered in `render_instanced.rs:13`) but `ffi/renderer3d/` has no particles module and zero SDK surface.
- **P8-03:** Only network-delta + one-way scene JSON exist (`core/serialization/`, `ffi/scene_loading.rs:47`); reuse the delta code's component walk.
- **P8-04:** No `rand`/`noise` in engine deps; cross-language determinism requires an engine-owned seeded PRNG (ties to P5-04 hashing).

### Batch 8.2 — Gameplay services (4 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P8-05 | Engine event bus with FFI subscription (#547) | A | M | — |
| ENG2-P8-06 | Isometric projection/tile utils (#550) | B | S | P6-05 |
| ENG2-P8-07 | 3D animation blend trees + 2-bone IK (extend the 2D controller pattern) | C | L | — |
| ENG2-P8-08 | cgmath → core/math migration (deprecated dep, 36 files) | D | M | — |

- **P8-07:** 2D has a real state machine with crossfade (`ecs/systems/animation_controller/system.rs:74-192`); 3D is raw clip playback only, zero IK hits repo-wide.
- **P8-08:** Deliberately late: Phases 2/3/6 already deleted much cgmath-using code; migrate the remainder onto `core/math`, drop the advisory-ignored dep.

### Batch 8.3 — Big-world (2 groups)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P8-09 | Navmesh generation/query for 3D navigation | A | L | P8-01 |
| ENG2-P8-10 | Chunked heightmap terrain component | B | L | P2-11 |
| ENG2-P8-11 | Terrain LOD + texture/mesh streaming | B | L | P8-10, P7-01 |

### Phase 8 Gate
- [ ] Nav bench: 10k agents repathing within ≤ 5 ms/frame budget; A*-parity correctness tests.
- [ ] Every new capability: FFI exports + all-SDK parity gate green + integration test + mdBook page (checklist per issue).
- [ ] Save/load: S1-scale world round-trips to identical world hash (P5-04 hook).
- [ ] cgmath absent from goud_engine Cargo.toml.

**══ SYNC C (throne_ge):** `THR-C-01` [S] — evaluate engine A*/flow-field, event bus, and RNG/noise against Throne's C# implementations; adopt where they beat the stubs; report findings on the engine-dependencies table.

---

## 11. Phase 9 — Authoring, Platforms, Examples & Polish (W8 + W6-platform)

**Goal:** Every kept platform is CI-real; developers can see and author scenes; examples prove the v2 API. 9 issues.

### Batch 9.1 — Platforms & tooling (4 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P9-01 | Scene inspector MVP (egui or web client over the MCP relay): entity tree + live component editing | A | L | P7-06 |
| ENG2-P9-02 | Inspector: scene/prefab authoring round-trip to the JSON scene format | A | M | P9-01 |
| ENG2-P9-03 | Mobile CI real: Android NDK cross-build + on-device smoke, iOS build job (#134) | B | L | Phase 1 gate |
| ENG2-P9-04 | Console targets: explicit experimental feature-gate, dedupe 3×323-line init copies, strategy record (#135) | C | M | — |
| ENG2-P9-05 | Networking truth: rename fake WebRTC to udp_datachannel or adopt a real crate; networking-tier decision record | D | M | — |

- **P9-01/02:** The MCP debugger is real (20 tools incl. inspect_entity/capture_frame, `tools/goudengine-mcp/src/server/tools.rs`) but there is no authoring tool at all — scenes are code/JSON only (`ffi/scene_loading.rs:47`; prefabs at `context_registry/scene/prefab.rs`). Build inspector on the relay; then save-back to scene JSON.
- **P9-03:** CI has no aarch64-linux-android build; jniLibs contain only .gitkeep; JNI is validated only on desktop JVM (`Cargo.toml:172` jni_smoke). Keep-all-targets decision ⇒ make it gated.
- **P9-04:** `switch_vulkan_platform.rs:52-57`/`xbox_gdk_platform.rs:55-56` are honest PoC stubs; the three wgpu init files are byte-near-identical. Extract shared init, gate behind `experimental-console`, CI-typecheck lane.
- **P9-05:** `providers/impls/webrtc_network/mod.rs:1-17` is custom UDP+STUN labeled WebRTC, `net-webrtc=[]` pulls no crate; ~7-8k LOC networking has zero production consumers — decision record on tiering (default-feature status) + honest naming.

### Batch 9.2 — Examples, docs, agent configs (4 groups, parallel)

| ID | Title | Grp | Effort | Blocked by |
|---|---|---|---|---|
| ENG2-P9-06 | Compiled Rust examples in the engine crate (currently zero .rs examples) | A | M | — |
| ENG2-P9-07 | Example games refresh: 2–3 games validating the v2 API (consolidates #333/#336/#340/#342/#345/#348/#139) | B | L | P9-06 |
| ENG2-P9-08 | Agents/skills v2 update: perf-capture skill, render-v2 patterns, perf-work rules in .agents/ | C | M | — |
| ENG2-P9-09 | mdBook v2 architecture rewrite + breaking-changes migration guide | D | M | — |

### Phase 9 Gate
- [ ] Inspector connects to a running S1, edits a transform live, saves scene JSON that reloads identically.
- [ ] Android + iOS CI jobs required; console typecheck lane explicit experimental; networking naming/tiering resolved.
- [ ] ≥ 2 example games + Rust examples build in required CI against the v2 API.
- [ ] Migration guide published; .agents/ + skills reference v2 patterns and the perf-capture procedure.

---

## 12. Existing open-issue disposition map (requirement 7)

| Issue | Disposition |
|---|---|
| #704 security deps | → ENG2-P1-10 |
| #679 CreatePlane bypasses instancing | → ENG2-P2-08 (closed at Phase 2 gate) |
| #678 frame cost scales with total | → ENG2-P2-02 + ENG2-P2-12 (closed at Phase 2 gate) |
| #677 shadow Metal stall | → ENG2-P2-10 (closed at Phase 2 gate) |
| #670 NuGet targets | → verify+path-fix+close in ENG2-P0-12; consumer removal in THR-S0-01 |
| #657 jni 0.21→0.22 | → ENG2-P1-02 |
| #596 V2-P08 Throne bump | close, superseded by THR-A-01 |
| #593/#592/#591/#590 target:throne | migrate to throne_ge (THR-A-01..04), close GE-side in ENG2-P0-12 |
| #554 coordinate origin | → ENG2-P6-05 |
| #553 UiManager absolute positioning | → ENG2-P7-05 |
| #550 isometric utils | → ENG2-P8-06 |
| #548 noise | → ENG2-P8-04 |
| #547 event bus | → ENG2-P8-05 |
| #546 A* | → ENG2-P8-01 |
| #475 ALPHA-002 master / #114 ALPHA-001 | close, superseded by the new v2 master tracking issue (ENG2-P0-12) |
| #425 wasm-pack deprecation | → ENG2-P1-05 |
| #424 flipX/flipY | → ENG2-P6-02 |
| #423 drawText web | close as invalid after verify (exists at `wasm/rendering.rs:239`) — ENG2-P0-12 |
| #348/#345/#342/#340/#336/#333/#139 example games | → ENG2-P9-07 |
| #273 leak detection | → ENG2-P3-08 |
| #270 unsafe audit / #267 FFI fuzz | → ENG2-P3-09 |
| #256 perf regression detection | → ENG2-P0-05 |
| #137 F22 testing | → ENG2-P1-06 |
| #136 F21 debugger | close (shipped); authoring successor ENG2-P9-01 |
| #135 console strategy | → ENG2-P9-04 |
| #134 mobile | → ENG2-P9-03 |
| Milestones 1–9 | close as superseded after migration (ENG2-P0-12) |

## 13. Issue count summary

| Phase | Engine issues | Throne issues |
|---|---|---|
| P0 Instrumentation & Truth | 12 | 1 (Sync 0) |
| P1 SDK Source of Truth | 10 | — |
| P2 Render Core v2 | 12 | — |
| P3 Data Core v2 | 9 | — |
| P4 FFI v2 Convergence | 8 | 4 (Sync A) |
| P5 Parallelism & Determinism | 6 | 1 (Sync B) |
| P6 2D Render v2 | 8 | — |
| P7 Runtime Services | 6 | — |
| P8 Capability Gaps | 11 | 1 (Sync C) |
| P9 Authoring/Platforms/Polish | 9 | — |
| **Total** | **91** | **7** → **98 overall** |

Effort mix: 9 S / 47 M / 35 L / 0 XL engine-side — every issue is sized for a single `/gh-issue <n> --worktree` agent run producing one focused PR. Each phase doc in `docs/runbook/v2/phases/` will list the run commands per batch (issue numbers filled at creation), e.g. `Batch 2.3 Group A: /gh-issue <P2-08#> --worktree`.

### Critical Files for Implementation
- /Users/aramhammoudeh/dev/game/GoudEngine/goud_engine/src/libs/graphics/renderer3d/core/mod.rs (scene storage — heart of Phase 2)
- /Users/aramhammoudeh/dev/game/GoudEngine/goud_engine/src/libs/graphics/backend/wgpu_backend/frame.rs (phase timing + frame path — Phase 0/2)
- /Users/aramhammoudeh/dev/game/GoudEngine/goud_engine/src/component_ops/storage.rs (global component store to be deleted — Phase 3)
- /Users/aramhammoudeh/dev/game/GoudEngine/codegen/validate_coverage.py (source-of-truth/parity gates — Phase 1)
- /Users/aramhammoudeh/dev/game/throne_ge/docs/runbook/phase-index.md (runbook convention to mirror)
