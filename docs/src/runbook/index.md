# GoudEngine v2 Rebuild Runbook ("ENG2")

This runbook is the authoritative plan for the GoudEngine v2 rebuild. It replaces the archived `ALPHA_ROADMAP.md`, whose "Phase 0 (v2): Core Performance — COMPLETE" claim was false when written (the perf gate was never validated against a consumer-scale scene, and #677/#678/#679 were open at the time).

## Why this exists

GoudEngine's flagship consumer, Throne (a C# colony-sim), ran at 1–17 FPS on moderate scenes and had to work around the engine (shadows disabled, an 80×80 terrain window, manual native-lib copying). A 12-subsystem, file:line-verified audit found two structural causes:

1. **Render submission is O(total), per-object, un-instanced.** Every frame linearly scans the entire scene-object hash map (main + shadow pass); no spatial index; `CreatePlane`/`CreateCube` never instance; each draw re-uploads a full 4KB uniform block.
2. **The FFI data path takes 3 global mutexes per component op and the component store is implemented three times**; one heap malloc per component; a raw pointer escapes its mutex guard; transform propagation walks the full tree every frame; the engine is single-threaded by construction.

The instrumentation that should have caught this was a facade (GPU timestamps disabled, so `shadow_pass` reported ~0 while frames stalled 300–600 ms; the perf-regression gate wired into zero CI jobs).

## The goal

Run **tens of thousands of visible entities at 120+ FPS** on Apple M-series and **100k+ simulation entities**, with per-frame cost **O(visible), not O(total)** — and become a general-purpose engine able to build any game. The forcing function is Throne, but the architecture targets far beyond it.

## Binding decisions

- **No legacy code kept** where we refactor. Breaking the API is allowed; Throne adopts afterward via filed `throne_ge` issues.
- **Full 10-SDK parity stays** — made cheap (one generated source of truth) and real (per-SDK CI gates), not pruned.
- **All platform targets stay** — made CI-real (Android/iOS gated, console experimental-gated), not pruned. Legacy OpenGL backend is removed per the no-legacy policy (ADR in Phase 0, executed in Phase 2).

## Execution model

- **Phases are sequential** (a phase's gate issue must close before the next starts) — except **Phase 1 ‖ Phase 2** run concurrently, and **Phases 5 ‖ 6 ‖ 7** run concurrently (their file sets are disjoint).
- **Batches within a phase are sequential; groups within a batch run in parallel** — one `/gh-issue <n> --worktree` per group, separate agents/terminals. All groups in a batch merge before the next batch.
- **Every issue is self-contained** for `/gh-issue` execution (Architecture Context, Scope, Acceptance Criteria, Files Likely Touched, Agent Notes with file:line evidence, Verification).
- **Issues are disposable.** If an issue drifts from its phase doc, close it and regenerate from the runbook — never patch a drifted issue.
- **Every `breaking-change` issue names its Throne adoption follow-up** (in `throne_ge`, label `goudengine-adoption`) before it closes.

## The perf definition of done

See [perf-dod.md](perf-dod.md). A performance issue closes only when the named scripted scene (S1–S5) hits its numeric target **and** a phase counter attributes the cost. A counter that reads zero means broken instrumentation, never "free".

## Where things live

- **Phase index + gates:** [phase-index.md](phase-index.md)
- **Full phase specs:** [phases/phase-0.md](phases/phase-0.md) … `phase-9.md`
- **Perf scenes + targets:** [perf-dod.md](perf-dod.md)
- **Testing strategy (unit/isolation/integration/E2E/spec, story gallery, determinism/fuzz):** [testing-strategy.md](testing-strategy.md)
- **Throne adoption sync points:** [throne-sync.md](throne-sync.md)
- **Issue template:** [issue-template.md](issue-template.md)
- **Master tracking issue:** pinned in the GoudEngine repo (ENG2 master).

## Concurrency map

```
P0  →  (P1 ‖ P2)  →  P3  →  P4 [SYNC A]  →  (P5 ‖ P6 ‖ P7) [SYNC B]  →  P8 [SYNC C]  →  P9
```
