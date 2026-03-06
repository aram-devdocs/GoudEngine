---
name: review-changes
description: Dispatch 5 parallel review agents to analyze pending changes
context: fork
user-invocable: true
---

# Review Changes

Dispatch five specialized review agents in parallel to analyze uncommitted or staged changes.

## When to Use

Run before committing significant changes, after completing a feature, or when requested. Particularly valuable for changes touching FFI boundaries or multiple modules.

## Workflow

### 1. Gather Context

```bash
git diff --stat                    # What files changed
git diff                           # Full diff
git diff --cached                  # Staged changes
```

### 2. Dispatch Five Review Agents (Parallel)

All five agents run simultaneously on the diff output:

#### Security Reviewer
- Scans for `unsafe` blocks without `// SAFETY:` comments
- Checks FFI functions for null pointer validation
- Looks for hardcoded secrets, API keys, credentials
- Validates memory ownership documentation on pointer parameters
- Flags new external crate additions for review

#### Performance Reviewer
- Identifies unnecessary allocations (`.clone()`, `String` where `&str` suffices)
- Checks for missing `#[inline]` on hot-path functions
- Looks for O(n²) patterns in loops
- Flags large stack allocations in FFI boundary code
- Reviews SpriteBatch usage for unnecessary draw calls

#### Architecture Reviewer
- Validates dependency flow (layers flow DOWN only)
- Checks for cross-module boundary violations
- Verifies new files are in correct module directories
- Ensures no upward imports in the 5-layer hierarchy:
  - Layer 1 (Core): `libs/`
  - Layer 2 (Engine): `goud_engine/src/`
  - Layer 3 (FFI): `goud_engine/src/ffi/`
  - Layer 4 (SDKs): `sdks/`
  - Layer 5 (Apps): `examples/`

#### Pattern Reviewer
- Checks against the 16 anti-patterns from AGENTS.md
- Validates FFI patterns (`#[no_mangle]`, `#[repr(C)]`)
- Ensures SDK wrappers remain thin (no logic)
- Verifies error handling uses `Result`, not `unwrap`/`expect` in library code
- Checks component patterns (derive Debug + Clone)

#### Simplicity Reviewer
- Flags files exceeding 500 lines
- Identifies over-engineered abstractions
- Checks for dead code or unused imports
- Looks for duplicated logic between Rust and SDK code
- Suggests simplifications

### 3. Aggregate Results

Collect findings from all five agents and categorize:

| Priority | Meaning | Action |
|----------|---------|--------|
| **P1 - BLOCKER** | Must fix before commit | Blocks merge |
| **P2 - WARNING** | Should fix, strong recommendation | Review before merge |
| **P3 - NOTE** | Minor improvement suggestion | Optional |

### 4. Output Format

```
## Review Summary

**Verdict**: APPROVED | CHANGES REQUESTED | BLOCKED

### BLOCKERS (P1)
| # | Reviewer | File | Finding |
|---|----------|------|---------|

### WARNINGS (P2)
| # | Reviewer | File | Finding |
|---|----------|------|---------|

### NOTES (P3)
| # | Reviewer | File | Finding |
|---|----------|------|---------|
```

## When to Block

A review returns BLOCKED if any P1 findings exist:
- `unsafe` without SAFETY comment
- Secrets in code
- Upward dependency violation
- SDK logic that should be in Rust
- Missing FFI null checks on pointer parameters
