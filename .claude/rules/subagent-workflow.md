---
alwaysApply: true
---

# Subagent Workflow

Implementation work SHOULD be dispatched through specialized subagents for consistency and quality.

## Dispatch Protocol

1. **Read** the task or spec
2. **Consult** a domain expert if the task touches graphics, ECS, or FFI
3. **Analyze** task independence — can subtasks run in parallel?
4. **Dispatch** to appropriate implementation agents (parallel if independent, sequential if dependent)
5. **Review** in two stages: spec-reviewer FIRST, then code-quality-reviewer
6. **Test** via test-runner agent
7. **Validate** architecture (always) and security (if FFI/unsafe changes)

## Parallel Batching

- Max 3–5 independent tasks per parallel batch
- Independence criteria: different files, no import/dependency relationships
- Security-sensitive work (FFI, unsafe, pointer operations) is ALWAYS sequential

## Agent Selection

| Task Type | Agent |
|---|---|
| General Rust implementation | implementer |
| FFI boundary changes | ffi-implementer |
| SDK wrapper development | sdk-implementer |
| Write failing tests first | test-first-implementer |
| Run and analyze tests | test-runner |
| Diagnose failures | debugger |
| Docs and README | documentation-writer |

## Anti-Patterns

- Implementing directly without considering subagent dispatch
- Running code-quality-reviewer before spec-reviewer
- Parallelizing security-sensitive tasks
- Batching more than 5 tasks at once
