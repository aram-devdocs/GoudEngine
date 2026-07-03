# 0003 — Canonical Layer Numbering

Status: Accepted

## Context

The architecture is a layered dependency hierarchy. When multiple documents
each describe the layers in their own words, the numbering and names diverge,
and no one can say which description is authoritative when they disagree.

## Decision

There is one canonical five-layer model, and `tools/lint_layers.rs` is its
source of truth:

1. **Foundation** — `core/`
2. **Libs** — `libs/`
3. **Services** — `ecs/`, `assets/`
4. **Engine** — `sdk/`, `rendering/`, `component_ops/`, `context_registry/`
5. **FFI** — `ffi/`, `wasm/`

Dependencies flow downward only. The linter scans imports under
`goud_engine/src/` and fails on any upward dependency. Documentation and every
nested `AGENTS.md` MUST match this model; where prose and the linter disagree,
the linter wins.

## Consequences

- The layer model is enforced by a gate (`cargo run -p lint-layers`), not by
  convention, so violations fail the build rather than accumulating.
- Docs are downstream of the linter. When the model changes, change
  `tools/lint_layers.rs` first, then reconcile the prose.
- A new module MUST be placed in a layer, and its imports MUST respect the
  downward-only rule, or the linter rejects it.
