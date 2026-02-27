---
alwaysApply: true
---

# TDD Workflow

All new functionality MUST follow the RED-GREEN-REFACTOR cycle.

## RED Phase

1. Write a failing test that captures the desired behavior
2. Run `cargo test` (or the appropriate SDK test command) to confirm the test **fails**
3. The failure message should clearly describe what's missing

## GREEN Phase

1. Write the **minimal** implementation to make the test pass
2. Run `cargo test` to confirm the test **passes**
3. Do not add features beyond what the test requires

## REFACTOR Phase

1. Improve code quality (naming, structure, duplication) while keeping tests green
2. Run `cargo test` after each refactoring step
3. Run `cargo clippy -- -D warnings` to catch lint issues

## Rules

- No `#[ignore]` on committed tests
- No `todo!()` or `unimplemented!()` in committed production code
- One assertion per concept (a test may have multiple asserts if they test the same logical concept)
- Follow Arrange-Act-Assert pattern
- Tests MUST have meaningful names describing the behavior under test
