# Orchestrator Protocol

Use the smallest workflow that still protects quality.

## Default

1. Scope the task.
2. Choose one implementation agent.
3. Verify the result.
4. Run one `reviewer` pass for substantive changes.
5. Add `security-auditor` only when the boundary is security-sensitive.

## Dispatch Guide

- engine/core work -> `engine-lead`
- FFI/SDK/codegen work -> `integration-lead`
- trivial fix -> `quick-fix`
- diagnosis -> `debugger`

Do not add extra review stages or nested agent trees unless the user explicitly asks for them.
