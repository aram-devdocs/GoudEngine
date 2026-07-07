# Phase 4 — FFI v2 Convergence

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

---

## Phase 4 — FFI v2 Convergence (W2-ffi) — *requires Phase 2 AND Phase 3 gates*

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
