---
name: gh-issue
description: Resolve one or more GitHub issues in a single branch and PR. Supports optional worktree isolation, shared run plans, deterministic state tracking, orchestration with subagents, review gates, and PR follow-through until CI and reviews are complete.
argument-hint: "<issue-number> [issue-number...] [--worktree]"
disable-model-invocation: true
---

# GitHub Issue Resolution

Use this skill when the user wants an issue handled end to end: investigate, plan, implement, review, open the PR, stay on the PR until checks and reviews are clean, then clean up the worktree.

## Load Order

Read these files in order:

1. `references/workflow-contract.md`
2. `references/resume-contract.md`
3. `references/evals.md` only when validating the skill output or running regression checks

Use these bundled assets instead of inventing new structures:

- `assets/plan-template.md`
- `assets/state-template.json`
- `assets/prompts/lead-dispatch.md`
- `assets/prompts/review-dispatch.md`

Use these scripts for deterministic behavior:

- `scripts/gh_issue_run.py`
- `scripts/gh_issue_workflow.py` (compatibility wrapper)
- `scripts/validate_skill.py`

## Core Rules

- `--worktree` means isolated worktree execution. Do not work on `main` in that mode.
- The canonical run directory is `.agents/runs/gh-issue/<primary>-<slug>/`.
- Every run must create both `plan.md` and `state.json` before implementation starts.
- `plan.md` is the durable contract for the next session. Keep it accurate and leave the non-negotiables in place.
- `state.json` is the machine-readable source of truth for todos, review gates, PR status, and cleanup state.
- Non-trivial work goes through team leads. Use `quick-fix` directly only for trivial single-file changes.
- Review order is fixed: `spec-reviewer`, then `code-quality-reviewer`, with `architecture-validator` and `security-auditor` as required.
- PR creation is not the end of the task. Stay with the PR until CI is green and review feedback is handled.
- Do related cleanup now if it is needed to finish the ticket cleanly. Only defer work that is already tracked elsewhere and truly out of scope.

## Workflow

### 1. Parse Arguments

- Collect one or more issue numbers from `$ARGUMENTS`.
- Detect `--worktree`.
- Use the lowest issue number as the primary issue.

### 2. Investigate

For each issue:

- `gh issue view <ISSUE> --json title,body,labels,comments,assignees,milestone`
- `gh pr list --search "<ISSUE>" --json number,title,state,url`
- `gh issue comment <ISSUE> --body "Starting work on this issue. Investigating scope and creating implementation plan."`

If the issue is ambiguous after investigation, ask on the issue and stop there.

### 3. Create the Canonical Run

Use `scripts/gh_issue_run.py init-run` to create:

- `.agents/runs/gh-issue/<primary>-<slug>/plan.md`
- `.agents/runs/gh-issue/<primary>-<slug>/state.json`

Pass the real branch, mode, worktree path, repo path, issue titles, and acceptance summary. In worktree mode, create the worktree first and record its absolute path.

### 4. Read the Generated Plan

- Read `plan.md` from the canonical run directory.
- Follow the non-negotiables and resume protocol exactly.
- Use the prompt fragments in `assets/prompts/` when dispatching leads and reviewers.

### 5. Keep State Fresh

Use `scripts/gh_issue_run.py update-state` whenever any of these change:

- phase
- todo ownership or status
- review gate verdicts
- PR number
- CI state
- review state
- cleanup state
- deferrals with linked tracking issue or PR

### 6. Validate Before Resume or Implementation

Run `scripts/gh_issue_run.py validate-resume`:

- before implementation starts
- after context clears
- after switching back into a worktree session

If it fails, stop and fix the mismatch first.

### 7. Stay With the PR

After `gh pr create`:

- record the PR number in `state.json`
- use `scripts/gh_issue_run.py poll-pr` for compact PR status
- keep going until the PR is ready, checks pass, and review feedback is handled

### 8. Validate the Skill Package

Run:

```bash
python3 .agents/skills/gh-issue/scripts/validate_skill.py
```

If the `skills-ref` tool is available, run it too. The local validator is the required baseline.
