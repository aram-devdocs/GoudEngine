# Testing v2 Rules (ENG2)

Applies to all ENG2 work. Full strategy: `docs/src/runbook/testing-strategy.md`. Extends `testing.md` (GL-context and unit conventions there still hold).

## Spec-first (TDD)

- Every behavioral issue gets a spec test at `goud_engine/tests/spec/eng2_p<N>_<nn>_<slug>.rs` encoding its Acceptance Criteria as executable assertions. The spec **is** the gate evidence.
- Write the spec RED before the implementation; make it GREEN; then REFACTOR. A reviewer confirms it was red-before-green.
- The issue's Verification section names the spec path. Do not close an issue whose spec test does not exist or does not assert its acceptance criteria.

## Tiers — put each test at the right level

- **Unit** — pure logic, no GPU. Design v2 render/ECS logic (culling, instancing-grouping, sort-key, propagation) to be unit-testable without a GPU context.
- **Isolation** — one subsystem behind its null/provider boundary (`NullBackend`, null net/audio, in-memory VFS). If the seam to mock a dependency is missing, add the null/mock provider.
- **Integration** — cross-layer in-process; assert exact draw/culled/visible counts (generalize `tests/renderer3d_frame_counts.rs`).
- **E2E** — SDK-driven headless engine (ENG2-P0-17); C#/TS/Python boot a real engine, run a scripted game, assert counts/world-hash/flat-RSS.

## Story gallery

- Every render/UI/particle/animation change adds or updates a **story** (ENG2-P0-14): scene + camera + knobs + expected invariants + golden image. CI renders it headless, diffs the golden, checks metric budgets, and publishes the HTML gallery.
- Changing a golden requires the documented golden-update approval — do not silently overwrite baselines.

## Determinism, property, fuzz

- Sim/scheduling changes run the determinism fixture (3 seeds × 10k ticks, hash-compare). Do not introduce nondeterminism (unordered iteration, wall-clock reads, FP drift).
- Component-store and command-decoder changes add/extend `proptest` + `cargo-fuzz` targets (ENG2-P0-18, P3-09).

## Don't game the gate

Never weaken an assertion, widen a tolerance, or skip a scene to make a gate pass. If a gate is wrong, fix the gate in the same PR with justification.
