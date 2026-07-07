# Phase 3 — Data Core v2 (ECS + store unification)

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

---

## Phase 3 — Data Core v2: ECS & Store Unification (W2-store) — *requires Phase 1 gate; may overlap Phase 2 Batches 2.2+*

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
