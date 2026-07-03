# 0001 — Canonical Verify Pipeline

Status: Accepted

## Context

When git hooks and CI maintain separate step lists, the two drift. A check
passes locally and fails in CI, or the reverse, and developers stop trusting
either. The pre-commit, pre-push, and CI paths need one authority for what
"verified" means.

## Decision

One data-driven script, `scripts/verify.sh`, owns the step table. The
pre-commit hook, the pre-push hook, and the CI workflows all route through it
instead of hand-mirroring the steps. "Passes locally" therefore means "passes
CI" by construction.

The staged (pre-commit) run is a strict subset of the full (pre-push / CI) run.
Steps are tagged so the staged set is guaranteed to be a subset of the full set,
not a separately maintained list.

## Consequences

- A new check is added once, in the step table, and every path picks it up.
- Local and CI results agree, so a green local run is trustworthy.
- The step table is a single point of control, so changes to it MUST be
  reviewed with the same care as the checks themselves.
- Scoping (staged subset, per-lane runs) is expressed as tags on steps rather
  than as forked scripts.
