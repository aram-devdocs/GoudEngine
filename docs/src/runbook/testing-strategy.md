# Testing & Validation Strategy (ENG2)

You cannot safely rewrite the render core (Phase 2) or unify the component store (Phase 3) without a regression net that catches what phase counters and ad-hoc benches miss. This strategy is built in Phase 0, **before** the rewrites, and every later issue plugs into it. The foundations already exist in the repo — 5,109 tests, a null-backend, the MCP capture tools, a lavapipe lane — they are just not wired into anything that gates a merge.

## The tiers

1. **Unit** — pure logic, colocated `#[cfg(test)]`. The v2 render/ECS cores are designed so culling, instancing-grouping, sort-key, and propagation logic are testable **without a GPU** (math-only), per `.agents/rules/testing.md`. Testability is a design constraint, not an afterthought.

2. **Isolation** — one subsystem behind the provider/null boundary. Renderer *logic* over `NullBackend`; ECS over a stub store; asset server over an in-memory VFS; networking/audio over null providers. Each layer gets an "isolation contract": what it must do correctly with its dependency mocked. New null/mock providers are added where a seam is missing.

3. **Integration** — cross-layer, in-process. FFI → store → renderer command-stream assertions; the count-pinning pattern in `tests/renderer3d_frame_counts.rs` generalized (assert exact draw calls / culled / visible for a scene).

4. **E2E** — SDK-driven headless engine (ENG2-P0-17). Each first-class SDK (C#, TypeScript, Python) boots a real headless engine, runs a scripted mini-game N frames, and asserts draw counts / world-state hash / no-crash / flat RSS. Runs in the Docker CI image on lavapipe. This is the tier that today does not exist — secondary SDK "tests" never touch a running engine.

5. **Spec / acceptance** (ENG2-P0-16) — each behavioral issue gets a spec test at `goud_engine/tests/spec/eng2_p<N>_<nn>_<slug>.rs` encoding its Acceptance Criteria as executable assertions. The spec **is** the gate evidence. The `tdd-workflow` skill drives RED-GREEN-REFACTOR against it; the issue template's Verification section names the spec path; a reviewer confirms it was red-before-green.

## Story gallery — "Storybook for the engine" (ENG2-P0-14)

A declarative catalog of isolated visual scenarios with visual + metric regression, `goud_gallery`:

- A **story** = scene setup + camera + knobs (args) + expected invariants (draw calls, instance counts, phase-timing budgets) + golden-image tag. One story per feature: each widget kind, sprite batching mode, instancing path, shadow config, post-process pass, animation state, UI layout, particle config.
- A **headless runner** renders each story to a texture on the wgpu backend (no window), captures framebuffer + metrics (MCP `capture_frame`/`get_metrics_trace`), perceptually diffs against a committed golden, and checks metric budgets.
- CI emits a **static HTML gallery** artifact per PR (image, diff-vs-golden, metrics table, pass/fail) — the Storybook browsing experience.
- **Interaction stories** (the "play function" analog) drive scripted input via the MCP `inject_input`/`replay` verbs and assert end-state.
- **Authoring**: every render/UI/particle/animation issue must add or update stories as an acceptance item; the golden-update protocol (who approves a changed golden, how baselines are versioned) is documented with the runner.

## Determinism, property & fuzz (ENG2-P0-18)

- **Determinism** fixture: run the headless sim 3 seeds × 10k ticks, compare world hashes. Consumed by ENG2-P5-04 and Throne's determinism gate.
- **Property** tests (`proptest`): culling visible-set vs brute force; the command-buffer decoder (Phase 4); instancing grouping.
- **Fuzz** (`cargo-fuzz`): component store add/get/remove/batch and the command-buffer decoder. Nightly lane with a time budget.

## Infrastructure

- **Docker CI image** (ENG2-P0-13): pinned toolchain + lavapipe + SDK toolchains; `docker compose run verify` reproduces the CI core gate locally. Metal-only real-GPU perf capture stays on the M-series host.
- **CI restructure** (ENG2-P0-19): fast core gate (fmt/clippy/unit/isolation, <10 min) always-on; heavier tiers (integration, E2E, story gallery, GPU lane, SDK matrix) staged by path filters + merge queue.
- **Self-hosted M-series nightly perf runner** (ENG2-P9-10): scenes S1–S5 + benches on real Metal; trend dashboard + regression alerting.

## What gates a merge

The core gate (fmt, clippy -D warnings, unit + isolation, lint-layers) is always required. `bench-gate.py`, the blocking GPU lane, per-SDK parity, and the story gallery become required as their Phase-0/Phase-1 issues land. No perf issue closes outside the [perf definition of done](perf-dod.md).
