# Phase 9 — Authoring, Platforms, Examples & Polish

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

> **Expansion:** Phase 9 adds ENG2-P9-10 (self-hosted M-series nightly perf runner + trend dashboard) — see [phase-specs-expansion.md](../phase-specs-expansion.md) §D. Total P9 issues = 10.

---

## Phase 9 — Authoring, Platforms, Examples & Polish (W8 + W6-platform)

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
