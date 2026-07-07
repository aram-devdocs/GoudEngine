# Phase 6 — 2D Render v2

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

---

## Phase 6 — 2D Render v2 (W4) — *concurrent with Phases 5, 7*

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
