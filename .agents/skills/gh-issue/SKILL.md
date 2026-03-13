---
name: gh-issue
description: End-to-end GitHub issue workflow with optional worktree isolation and resumable run state
argument-hint: "<issue-number> [issue-number...] [--worktree]"
disable-model-invocation: true
---

# `/gh-issue`

Use this skill only when you explicitly want issue-driven work with a resumable run directory and PR tracking.

## Workflow

1. Investigate the issue or issue batch.
2. Create or refresh the run directory.
3. Write a compact execution plan.
4. Implement with one implementation agent.
5. Verify the branch locally.
6. Run one `reviewer` pass.
7. Open or update the PR.

Add `security-auditor` only for FFI, unsafe, pointer, or ownership-boundary changes.

## Important Defaults

- `/gh-issue` is opt-in. Ordinary sessions should not behave like issue runs.
- Worktree mode is for isolation, not process inflation.
- Do not require nested implementation waves by default.
- Do not require multiple review gates by default.
- Do not idle indefinitely waiting on CI or review. Poll briefly, summarize the state, and stop unless the user explicitly asks you to keep monitoring.

## Canonical Run Artifacts

```text
.agents/runs/gh-issue/<primary>-<slug>/
  plan.md
  state.json
```

Use `python3 .agents/skills/gh-issue/scripts/gh_issue_run.py` to create and update these artifacts.

## Expectations

- The plan should be concise and action-oriented.
- The state file should capture the current phase, todos, review status, PR status, and cleanup status.
- Existing legacy runs can finish as-is. New runs should follow the simplified workflow.
