---
name: tdd-workflow
description: RED-GREEN-REFACTOR workflow for GoudEngine changes
context: fork
user-invocable: true
---

# TDD Workflow

Use this skill when a change clearly benefits from writing the failing test first.

## Default Shape

- engine/core change -> `engine-lead`
- FFI or SDK change -> `integration-lead`
- post-change review -> `reviewer`

## Cycle

1. **RED**: add the smallest failing test that proves the missing behavior.
2. **GREEN**: implement the smallest change that makes the test pass.
3. **REFACTOR**: improve clarity without changing behavior.
4. **VERIFY**: run targeted tests, then the smallest broader validation that matches the risk.

## Rules

- Do not add placeholder tests.
- GL-dependent tests must initialize context correctly.
- SDK-facing behavior should be proven in the layer that changed.
- Keep the cycle tight; do not turn TDD into a multi-agent process tree.
