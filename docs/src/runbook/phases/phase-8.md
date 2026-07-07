# Phase 8 — Capability Gaps

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

---

## Phase 8 — Capability Gaps (W7) — *requires Phases 5+6+7*

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
