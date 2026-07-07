# Phase 1 — SDK Single Source of Truth

_Authoritative spec. See also the [phase index](../phase-index.md), [perf-dod](../perf-dod.md), and [testing-strategy](../testing-strategy.md). Live issue numbers: pinned master #810._

---

## Phase 1 — SDK Single Source of Truth (W6-core) — *concurrent with Phase 2*

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
| ENG2-P1-03 | Generate ffi_mapping.json from the Rust manifest; delete the 30-entry _TYPE_ALIASES drift table | A | M | — |
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
