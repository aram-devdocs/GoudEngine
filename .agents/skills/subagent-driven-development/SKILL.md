---
name: subagent-driven-development
description: Orchestrate parallel subagent batches with two-stage review for GoudEngine tasks
context: fork
user-invocable: true
---

# Subagent-Driven Development

Orchestrate implementation through parallel subagent batches with structured review gates.

## When to Use

Invoke this skill for any non-trivial implementation task that touches multiple files or modules. Single-file fixes do not require subagent orchestration.

## Workflow

### 1. Analyze Task Independence

Before dispatching, classify each subtask:

- **Independent**: Different files, no import relationships, no shared state
- **Dependent**: One task's output is another's input (e.g., Rust impl → FFI export → SDK wrapper)

### 2. Wave-Based Dispatch

Group independent tasks into waves. Execute each wave in parallel, then synchronize before the next wave.

**Example wave plan for a new engine feature:**

```
Wave 1 (parallel):
  - implementer: Rust core implementation
  - test-first-implementer: Write failing tests

Wave 2 (sequential, depends on Wave 1):
  - ffi-implementer: FFI exports
  - test-runner: Verify Rust tests pass

Wave 3 (parallel, depends on Wave 2):
  - sdk-implementer (C#): C# SDK wrapper
  - sdk-implementer (Python): Python SDK wrapper

Wave 4 (sequential review gate):
  - spec-reviewer: Validate against requirements
  - code-quality-reviewer: Check patterns and anti-patterns (only after spec-reviewer APPROVED)

Wave 5 (parallel, conditional):
  - architecture-validator: Layer hierarchy check
  - security-auditor: Only if FFI/unsafe touched
```

### 3. Batch Size Limits

- Maximum 3-5 tasks per parallel batch
- Security-sensitive work (FFI, unsafe) is ALWAYS sequential
- Review agents run sequentially (spec-reviewer THEN code-quality-reviewer)

### 4. Module-Based Batching

Group by GoudEngine module boundaries:

| Module | Directory | Typical Agent |
|--------|-----------|---------------|
| Graphics | `goud_engine/src/libs/graphics/` | implementer + graphics-domain-expert |
| ECS | `goud_engine/src/ecs/` | implementer + ecs-domain-expert |
| FFI | `goud_engine/src/ffi/` | ffi-implementer + ffi-domain-expert |
| C# SDK | `sdks/GoudEngine/` | sdk-implementer |
| Python SDK | `sdks/python/` | sdk-implementer |
| Examples | `examples/` | implementer |

### 5. Verification

After all waves complete:

```bash
cargo test                              # All Rust tests
cargo clippy -- -D warnings             # Lint check
python3 sdks/python/test_bindings.py    # Python SDK tests
```

## Two-Stage Review Protocol

Every implementation MUST pass through both review gates in order:

1. **spec-reviewer** — Does the implementation match the task requirements? No scope creep? No missing edge cases?
2. **code-quality-reviewer** — Only runs after spec-reviewer returns APPROVED. Checks code patterns, anti-patterns (16-item list from CLAUDE.md), naming, structure.

If either reviewer returns REJECTED or CHANGES REQUESTED, route back to the appropriate implementer agent with the findings.

## Anti-Patterns

- Dispatching dependent tasks in the same parallel batch
- Skipping the review gate
- Running security-auditor in parallel with other agents
- Batches larger than 5 tasks
- Implementing directly without subagent dispatch
