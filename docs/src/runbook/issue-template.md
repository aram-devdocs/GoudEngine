## Parent
- **Program:** ENG2 — GoudEngine v2 Rebuild (see the pinned master tracking issue)
- **Phase / Milestone:** {phase name} (`{milestone}`)
- **Batch / Group:** {batch.group if known, else "see phase doc"}
- **Runbook spec:** `docs/src/runbook/phases/{phase-file}.md` (committed with the roadmap)

## Summary
{1–3 sentences naming the exact artifact produced — concrete, not "improve X".}

## Architecture Context
**Layer:** {Foundation | Libs | Services | Engine | FFI} (see `tools/lint_layers.rs` — downward-only deps).
**Modules/types touched:**
- `goud_engine/src/{path}` — {why}

**Boundary constraints (only those that apply):**
- Raw GPU calls stay in `libs/graphics/backend/` — no `wgpu::` outside it.
- Generated `*.g.rs`/`*.g.cs`/`*.g.ts` are never hand-edited; change Rust + FFI + schema, rerun `./codegen.sh`.
- FFI exports: `#[no_mangle] extern "C"`, `#[repr(C)]`, null checks, `// SAFETY:` comments.

**Pattern to follow:** {existing code to mirror, with path.}

## Scope
- [ ] {deliverable 1}
- [ ] {deliverable 2}
- [ ] Tests: {spec test + unit/isolation as applicable}
- [ ] Docs/rules updates if this invalidates `.agents/rules/*` or `docs/src/*`

## Acceptance Criteria
- [ ] {observable, testable}
- [ ] {PERF (if area:performance): numeric target on a named scene S1–S5 per `docs/src/runbook/perf-dod.md`, captured with non-zero phase-counter attribution}
- [ ] `cargo check && cargo fmt --all -- --check && cargo clippy -- -D warnings` clean; `cargo test` green; `./codegen.sh && git diff --exit-code` (drift gate)

## Breaking Change & Throne Follow-up
{If breaking-change: name the changed/removed API, state NO compat shim is kept (user decision), and link the throne_ge adoption issue that must be filed before close. Else: "None — additive/internal."}

## Blocked By
{list ENG2 IDs, e.g. "ENG2-P2-04, ENG2-P0-09" — or "None."}

## Files Likely Touched
- **New/Modified/Generated:** {paths}

## Agent Notes
{Non-obvious context with file:line evidence from the audit — prevents wrong assumptions. Include the specific evidence bullets from design-technical.md / design-expansion.md for this issue.}

## Verification
```bash
cargo check && cargo fmt --all -- --check && cargo clippy -- -D warnings
cargo test
./codegen.sh && git diff --exit-code
# perf issues: python3 scripts/bench-gate.py (per perf-dod.md), scene capture attached
# spec test: goud_engine/tests/spec/{eng2_pN_nn_slug}.rs
```
