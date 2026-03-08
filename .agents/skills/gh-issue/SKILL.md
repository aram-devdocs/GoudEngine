---
name: gh-issue
description: Resolve one or more GitHub issues in a single branch and PR. Supports optional worktree isolation, shared run plans, deterministic state tracking, orchestration with subagents, review gates, and PR follow-through until CI and reviews are complete.
argument-hint: "<issue-number> [issue-number...] [--worktree]"
disable-model-invocation: true
---

# GitHub Issue Resolution

Use this skill when the user wants an issue handled end to end: investigate, plan, implement, review, open the PR, stay on the PR until checks and reviews are clean, then clean up the worktree.

## Workflow Rules

!`cat ${CLAUDE_SKILL_DIR}/references/workflow-contract.md`

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

The generated `plan.md` MUST match the Plan Template below. `scripts/gh_issue_run.py init-run` renders this template with actual values.

### 4. Execute the Plan

- Read `plan.md` from the canonical run directory.
- Follow the non-negotiables and resume protocol exactly.
- Use the dispatch prompts below when dispatching leads and reviewers.

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

## Plan Template

CRITICAL: Every `plan.md` must follow this exact structure. Do not invent a different structure. `scripts/gh_issue_run.py init-run` renders this template with actual values.

!`cat ${CLAUDE_SKILL_DIR}/assets/plan-template.md`

## Dispatch Prompts

### Team Lead Dispatch

!`cat ${CLAUDE_SKILL_DIR}/assets/prompts/lead-dispatch.md`

### Review Dispatch

!`cat ${CLAUDE_SKILL_DIR}/assets/prompts/review-dispatch.md`

## Assets

- `assets/state-template.json` — machine-readable state schema for `state.json`

## Scripts

Use these scripts for deterministic behavior:

- `scripts/gh_issue_run.py` — run lifecycle (init-run, update-state, validate-resume, poll-pr)
- `scripts/gh_issue_workflow.py` — compatibility wrapper
- `scripts/validate_skill.py` — skill package validation

## References

- `references/resume-contract.md` — Read when resuming an interrupted run
- `references/evals.md` — Read when validating skill output or running regression checks
