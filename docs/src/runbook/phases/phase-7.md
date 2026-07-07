# Phase 7 — Runtime Services

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

---

## Phase 7 — Runtime Services (W5) — *concurrent with Phases 5, 6*

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
