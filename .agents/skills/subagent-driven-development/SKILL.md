---
name: subagent-driven-development
description: Small multi-agent workflow for GoudEngine tasks
context: fork
user-invocable: true
---

# Subagent-Driven Development

Use this skill when a task is large enough that one dedicated implementation agent and one review pass will improve the result.

## Default Shape

```text
root -> engine-lead|integration-lead -> reviewer
```

Add `security-auditor` only for FFI, unsafe, pointer, or ownership-boundary changes.

## When To Use It

- multi-file engine changes
- FFI or SDK changes
- changes that need a deliberate post-implementation review

Do not use it for trivial one-file fixes.

## Rules

- One implementation agent at a time.
- No nested specialist waves by default.
- One reviewer by default.
- No mandatory disagreement or forced findings.
- Do not leave sessions idling on CI or review waits unless the user explicitly asks for monitoring.
- `/gh-issue` is the exception: it owns the stricter issue-run workflow, specialized review gates, PR follow-through, and cleanup.
