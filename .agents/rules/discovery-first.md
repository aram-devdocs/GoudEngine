---
description: Verify current state before any claim or edit
alwaysApply: true
---

# Discovery First

Claims about this codebase MUST be grounded in what the code actually says right now, not in what any document, memory file, or `AGENTS.md` listing asserts.

## Before You Claim or Edit

- You MUST `grep`/`read` the relevant files before stating how something works or before changing it.
- You MUST cite `file:line` evidence for any non-trivial claim about behavior, structure, or wiring.
- You MUST NOT trust a doc's file list, an `AGENTS.md` table, an auto-memory note, or a prior agent's summary over the actual source. These drift; the source does not.
- You MUST re-check state that a doc describes as fixed — feature flags, layer membership, exported symbols, and default features change without the doc following.

## When Doc and Code Disagree

- The code and the validators (`cargo run -p lint-layers`, `codegen/validate_coverage.py`, `check-agents-md.sh`, `cargo clippy -- -D warnings`) are the source of truth. The doc is not.
- When a doc contradicts what the code or a validator reports, the code/validator wins.
- You MUST fix the stale doc in the SAME change that surfaced the contradiction. Do not defer it, do not open a follow-up, do not leave a `TODO`.

## Evidence Format

- Quote the exact `path:line` you relied on when the precise text is load-bearing (a signature, a guard, a flag).
- Prefer a fresh `grep` over recalling a result from earlier in the session — the tree may have moved under you.
