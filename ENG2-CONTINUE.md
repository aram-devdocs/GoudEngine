# ENG2 — Continue the GoudEngine v2 Rebuild (agent hand-off)

This file is the entry point for a coding agent (Codex, Claude, etc.) continuing the
GoudEngine v2 performance rebuild. Read it, then read the runbook, then execute.

## Paste-into-Codex goal prompt

> Copy the block below into Codex as the session goal.

```
GOAL: Execute the GoudEngine v2 rebuild ("ENG2"), one issue at a time, until the
phase gates are met.

START by reading, in order:
1. ENG2-CONTINUE.md (this file)
2. docs/src/runbook/index.md and docs/src/runbook/phase-index.md
3. The pinned GitHub master tracking issue #810 (gh issue view 810), which maps every
   ENG2-P<phase>-<nn> ID to its live issue number and shows the phase/gate structure.
4. docs/src/runbook/perf-dod.md and docs/src/runbook/testing-strategy.md.

EXECUTION MODEL (strict):
- Work phases in order. Phase 0 has no dependencies — start there. Phases 1 and 2 may
  run concurrently after Phase 0's gate passes; Phases 5/6/7 are concurrent after Phase 4.
- Within a phase, do batches in order; issues within a batch (groups) are independent.
  Respect each issue's "Blocked By" (listed by ENG2 ID in the issue body).
- Do NOT start a later phase until the prior phase's gate issue (type:gate) is closed
  with evidence. Gates are in docs/src/runbook/phases/phase-<N>.md and the master issue.

FOR EACH ISSUE:
- Pull the issue body (it is a self-contained spec: Architecture Context, Scope,
  Acceptance Criteria, Files Likely Touched, Agent Notes with file:line evidence).
- Write a spec test FIRST at goud_engine/tests/spec/eng2_p<N>_<nn>_<slug>.rs that encodes
  the Acceptance Criteria (RED), then implement (GREEN), then refactor. See
  .agents/rules/testing-v2.md.
- Follow the layer architecture (tools/lint_layers.rs — downward-only deps) and the
  rules in .agents/rules/ that match your change area (perf-work.md, render-v2.md,
  testing-v2.md, ffi-patterns.md, ecs-patterns.md, graphics-patterns.md, etc.).
- Any FFI change: rerun ./codegen.sh and keep all 10 SDKs in parity (a hard CI gate).
- Breaking changes are allowed (no legacy code kept), but every breaking-change issue
  must file/point to its Throne adoption follow-up in aram-devdocs/throne_ge
  (milestone goudengine-v2-adoption) before it closes.

DEFINITION OF DONE (do not skip):
- Verify: cargo check && cargo fmt --all -- --check && cargo clippy -- -D warnings &&
  cargo test && ./codegen.sh && git diff --exit-code (generated-artifact drift gate).
- For any area:performance issue, meet docs/src/runbook/perf-dod.md: the named scene
  (S1-S5/H1) hits its numeric target AND a phase counter attributes the cost. A counter
  that reads zero means broken instrumentation, never "free." Attach the capture.
- Open ONE pull request per issue on a branch off main. Do NOT merge without green CI.
  Reference the issue it closes.

CONSTRAINTS:
- main is branch-protected; work on branches and open PRs.
- Do not force-merge past red CI. Do not weaken a gate/assertion to make it pass; if a
  gate is wrong, fix the gate in the same PR with justification.
- Prefer the highest-impact critical path first: Phase 0 instrumentation, then Phase 2
  render core (closes #677/#678/#679), which is the biggest FPS win.

Begin with Phase 0. Report which issue you're taking, implement it end to end, open the
PR, then take the next unblocked issue.
```

## Context you need

**What this program is.** A 10-phase rebuild of GoudEngine's performance-critical cores
so the engine runs tens of thousands of visible entities at 120+ FPS (O(visible), not
O(total)) and 100k+ sim entities — driven by a full-engine audit. Full narrative:
`docs/src/runbook/index.md`.

**Where the plan lives.**
- Master tracking issue: **#810** (pinned) — the authoritative ID→issue-number map.
- Runbook: `docs/src/runbook/` — `phase-index.md`, `perf-dod.md`, `testing-strategy.md`,
  `throne-sync.md`, `issue-template.md`, and `phases/phase-0.md … phase-9.md`
  (plus `phase-specs.md` / `phase-specs-expansion.md` for the full per-issue detail).
- 102 engine issues titled `ENG2-P<phase>-<nn>: …` across milestones `eng2-p0-truth`
  … `eng2-p9-authoring` + `eng2-continuous`. 7 Throne adoption issues in throne_ge.

**The two root causes to fix (highest impact first).**
1. Render submission is O(total), per-object, un-instanced (Phase 2). Every frame scans
   the whole scene hash map (main + shadow); no spatial index; `CreatePlane`/`CreateCube`
   never instance; each draw re-uploads a 4KB uniform block. Issues #677/#678/#679.
2. The FFI component store is triple-implemented behind 3 global mutexes with a
   pointer-escape soundness bug; single-threaded by construction (Phase 3).
   Instrumentation is a facade — GPU timestamps disabled, bench-gate wired to no CI
   (Phase 0 fixes this FIRST, so later perf work is measurable).

**Current state (2026-07-07).**
- Roadmap issues + milestones + master #810 created; legacy issues/milestones closed.
- Runbook merged to main (PR #811).
- Early perf PRs already merged: #684 (shadow-pass allocs, toward #677) and #685
  (InstantiatePlane instancing, toward #679). Main compiles.
- Still open / needs work by you: PR #686 (spatial-index frustum cull for #678) has
  merge conflicts — rebase it onto main and finish it; it maps to ENG2-P2-02. A
  dependabot rust-patch group PR (#706) has failing CI. The release PR (#681) is left
  for a deliberate release.

**Guardrails that will block a careless PR (all are CI gates):** layer-boundary lint,
10-SDK codegen parity + drift, clippy -D warnings, AGENTS.md 55-line cap +
stale-pattern check, and (once Phase 0 lands) the bench-gate, story gallery, and
GPU-test lane. Read `.agents/rules/*.md` for the area you touch.

Start with Phase 0. It unblocks everything else.
