---
name: code-review
description: 7-phase structured code review for GoudEngine changes
user-invocable: true
---

# Code Review

Perform a thorough 7-phase structured code review on specified files or the current diff.

## When to Use

Run on PRs, before merges, or on any set of changes that need careful review. More thorough than `/review-changes` — this is a deep single-agent review rather than parallel multi-agent scan.

## 7 Review Phases

### Phase 1: Scope Analysis

- What modules are affected? (graphics, ecs, ffi, sdk, examples)
- Is the change size reasonable? (flag if >500 lines changed)
- Are all changed files in the correct module directories?
- Does the change match its stated purpose?

### Phase 2: Correctness

- Does the implementation match requirements?
- Are edge cases handled? (null inputs, empty collections, overflow)
- Is error handling complete? (`Result` types, no silent failures)
- Are there potential panics? (`unwrap()`, `expect()`, `todo!()`, `unimplemented!()`)
- Do tests cover the changed behavior?

### Phase 3: Security

- **unsafe blocks**: Every `unsafe` block MUST have a `// SAFETY:` comment explaining why it's sound
- **FFI boundary**: All pointer parameters validated for null, memory ownership documented
- **External crates**: Any new dependencies reviewed for security posture
- **Secrets**: No hardcoded credentials, API keys, tokens
- **Integer overflow**: Checked arithmetic in FFI boundary code

### Phase 4: Performance

- Unnecessary allocations (`.clone()` on hot paths, `String` where `&str` works)
- Missing batch optimizations (SpriteBatch for draw calls)
- Allocation in per-frame code paths
- Large structs passed by value across FFI
- Missing `#[inline]` on small frequently-called functions

### Phase 5: Maintainability

- Files under 500 lines
- Functions under 50 lines
- Clear naming (Rust conventions: `snake_case` functions, `PascalCase` types)
- No code duplication between Rust and SDK layers
- Dependency hierarchy respected (5-layer model, flow DOWN only)

### Phase 6: Testing

- New code has corresponding tests
- Tests follow Arrange-Act-Assert pattern
- No `#[ignore]` or `todo!()` in test code
- GL-dependent tests use `test_helpers::init_test_context()`
- Math/logic tests do NOT require GL context
- Integration tests in `goud_engine/tests/`, unit tests colocated

### Phase 7: Documentation

- Public items have doc comments
- FFI functions document memory ownership
- Complex algorithms have explanatory comments
- CLAUDE.md files updated if module structure changed
- No AI-sounding prose ("leverage", "utilize", "comprehensive", "robust")

## Output Format

```
## Code Review: [scope description]

### Phase 1: Scope — ✅ PASS | ⚠️ CONCERNS | ❌ FAIL
[findings]

### Phase 2: Correctness — ✅ PASS | ⚠️ CONCERNS | ❌ FAIL
[findings]

...

### Overall Verdict: APPROVED | CHANGES REQUESTED | BLOCKED

### Action Items
| Priority | Phase | Finding | Suggested Fix |
|----------|-------|---------|---------------|
```

## Verdict Criteria

- **APPROVED**: No P1 findings, ≤3 P2 findings
- **CHANGES REQUESTED**: Any P2 findings that affect correctness or security
- **BLOCKED**: Any P1 finding (unsafe without SAFETY, secrets, panics in library code)
