# Phase 2 — Render Core v2

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

> **Expansion:** Phase 2 adds ENG2-P2-13 (SoA transform store, zero-copy extract) and ENG2-P2-14 (GPU-driven culling / indirect), and raises the gate to super-high-FPS numbers — see [phase-specs-expansion.md](../phase-specs-expansion.md) §A/§B. Total P2 issues = 14.

---

## Phase 2 — Render Core v2 (W1) — *concurrent with Phase 1*

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
