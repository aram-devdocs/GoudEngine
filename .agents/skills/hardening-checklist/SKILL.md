---
name: hardening-checklist
description: 12-area audit checklist adapted for Rust game engine projects
user-invocable: true
---

# Hardening Checklist

Comprehensive audit across 12 areas adapted from web-stack hardening to a Rust game engine with FFI and multi-language SDKs.

## When to Use

Run periodically (weekly or before releases) to assess project health. Produces a scored report with actionable findings.

## Scoring

Each area is scored 0-10. Total score out of 120. Ratings:
- **100-120**: Production-hardened
- **80-99**: Solid, minor gaps
- **60-79**: Functional, needs attention
- **Below 60**: Significant gaps

## 12 Audit Areas

### 1. Project Structure (0-10)

- [ ] Clear module boundaries (graphics, ecs, ffi, sdk, assets, core)
- [ ] No circular dependencies between modules
- [ ] Cargo workspace configured correctly
- [ ] Files under 500 lines
- [ ] Dependency hierarchy documented and enforced (5-layer model)
- [ ] Build scripts (`dev.sh`, `build.sh`, `package.sh`) working

### 2. Type Safety (0-10)

- [ ] Rust compiler errors at zero
- [ ] No `#[allow(unused)]` without justification comment
- [ ] Generational handles for resource references (not raw IDs)
- [ ] `#[repr(C)]` on all FFI-shared structs
- [ ] Enums with explicit discriminants for FFI
- [ ] No `as` casts across FFI boundary without validation

### 3. Code Quality Gates (0-10)

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo deny check` passes (license + advisory audit)
- [ ] Pre-commit hook configured and running
- [ ] Pre-push hook with comprehensive checks
- [ ] CI pipeline running all checks

### 4. Testing (0-10)

- [ ] `cargo test` passes with zero failures
- [ ] Unit tests colocated in `#[cfg(test)]` modules
- [ ] Integration tests in `goud_engine/tests/`
- [ ] GL-dependent tests use `test_helpers::init_test_context()`
- [ ] Math/logic tests independent of GL context
- [ ] Python SDK tests passing (`test_bindings.py`)
- [ ] No `#[ignore]` or `todo!()` in committed test code
- [ ] Coverage target: 80%+ new code

### 5. Graphics Architecture (0-10)

- [ ] All raw OpenGL calls confined to backend module
- [ ] Renderer trait abstraction for backend swapping
- [ ] SpriteBatch for batched 2D draw calls
- [ ] Shader compilation with error reporting
- [ ] Camera systems (2D orthographic, 3D perspective) separated
- [ ] Texture management with proper GPU resource cleanup

### 6. AI Agent Configuration (0-10)

- [ ] Root AGENTS.md with commands, architecture, anti-patterns
- [ ] Distributed AGENTS.md in subdirectories
- [ ] CLAUDE.md and GEMINI.md symlinks to AGENTS.md
- [ ] `.claude/agents/` with subagent definitions
- [ ] `.claude/rules/` with contextual rules
- [ ] `.cursor/rules/` with IDE-specific rules

### 7. Agent Infrastructure (0-10)

- [ ] `.claude/hooks/` with lifecycle hooks
- [ ] `.agents/skills/` with reusable skills
- [ ] Session continuity (MEMORY.md, specs/)
- [ ] Dangerous command guard
- [ ] Secret scanner
- [ ] Quality check on file writes

### 8. Error Handling (0-10)

- [ ] `thiserror` for error type definitions
- [ ] `Result` return types (no `unwrap()`/`expect()` in library code)
- [ ] Error codes map to categories for FFI (i32, 0 = success)
- [ ] Graceful degradation for missing assets
- [ ] No `panic!()` in library code paths

### 9. Logging (0-10)

- [ ] Structured logging with levels (error, warn, info, debug, trace)
- [ ] No `println!()` in library code (use logger)
- [ ] Performance-critical paths use `debug!`/`trace!` (compiled out in release)
- [ ] FFI errors logged before returning error codes

### 10. FFI Safety (0-10)

- [ ] All FFI functions `#[no_mangle] extern "C"`
- [ ] Every `unsafe` block has `// SAFETY:` comment
- [ ] Null pointer checks on all pointer parameters
- [ ] Memory ownership documented (who allocates, who frees)
- [ ] No dangling pointers across FFI calls
- [ ] String handling: CStr/CString for C-compatible strings
- [ ] csbindgen generating correct C# bindings

### 11. SDK Parity (0-10)

- [ ] Every FFI export has C# DllImport wrapper
- [ ] Every FFI export has Python ctypes wrapper
- [ ] C# uses PascalCase, Python uses snake_case
- [ ] SDKs are thin wrappers (no logic implemented in SDK)
- [ ] SDK tests verify binding correctness
- [ ] Version numbers synchronized across Rust/C#/Python

### 12. Security Basics (0-10)

- [ ] No hardcoded secrets in source
- [ ] `.gitignore` covers build artifacts, secrets, IDE files
- [ ] `cargo deny` checks for known vulnerabilities
- [ ] Dependency audit (no unnecessary crates)
- [ ] Git hooks prevent secret commits
- [ ] No force-push to main branch

## Output Format

```
# Hardening Audit — GoudEngine

Date: YYYY-MM-DD
Score: XX/120 (RATING)

| # | Area | Score | Key Findings |
|---|------|-------|-------------|
| 1 | Project Structure | X/10 | ... |
| 2 | Type Safety | X/10 | ... |
...

## Critical Findings (must fix)
- ...

## Recommended Improvements
- ...

## Strengths
- ...
```
