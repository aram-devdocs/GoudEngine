---
name: gh-issue
description: Strict issue-delivery workflow with worktrees, durable run state, sequential review gates, PR template enforcement, Claude review handling, CI follow-through, and cleanup
argument-hint: "<issue-number> [issue-number...] [--worktree]"
disable-model-invocation: true
---

# `/gh-issue`

Use this skill only when you explicitly want issue-driven work with a canonical run directory and full PR follow-through.

## Workflow

1. Investigate the issue batch with `gh`.
2. Create the branch `codex/issue-<primary>-<slug>`.
3. Create an isolated worktree.
4. Initialize `plan.md` and `state.json`.
5. Implement with one lead role.
6. Verify locally.
7. Run strict review gates in order.
8. Create the PR with `.github/pull_request_template.md`.
9. Wait for GitHub Claude review and handle blockers and warnings.
10. Wait for CI to go green.
11. Clean up the local worktree.

## Defaults

- `/gh-issue` is opt-in. Ordinary sessions stay on the small default workflow.
- One implementation lead at a time.
- `spec-reviewer -> code-quality-reviewer -> architecture-validator` are mandatory.
- `security-auditor` is mandatory only for FFI, unsafe, pointer, or ownership-boundary changes.
- Do not stop at PR open, first review, or first CI poll.
- Do not hand-author PR bodies or review prompts from scratch. Use the prompt assets and PR template.

## Canonical Run Artifacts

```text
.agents/runs/gh-issue/<primary>-<slug>/
  plan.md
  state.json
```

Use `python3 .agents/skills/gh-issue/scripts/gh_issue_run.py` to initialize, update, poll, validate, and clean up the run.

## Required References

- `references/workflow-contract.md`
- `references/resume-contract.md`
- `references/evals.md`
- `assets/plan-template.md`
- `assets/state-template.json`
- `assets/prompts/lead-dispatch.md`
- `assets/prompts/review-dispatch.md`
- `assets/prompts/pr-creation.md`
- `assets/prompts/feedback-triage.md`
- `assets/prompts/ci-polling.md`
- `assets/prompts/cleanup-completion.md`

## Expectations

- `plan.md` is the durable contract for the next session.
- `state.json` is the mutable source of truth for review gates, PR state, Claude review handling, CI, and cleanup.
- Branch names must use `codex/issue-<primary>-<slug>`.
- Commit titles and PR titles must be conventional.
- PR bodies must be derived from `.github/pull_request_template.md`.
