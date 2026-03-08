# Workflow Contract

`/gh-issue` creates one shared run directory per issue batch:

```text
.agents/runs/gh-issue/<primary>-<slug>/
  plan.md
  state.json
```

`plan.md` is the durable contract. It should be readable by a fresh Claude or Codex session with no extra context. `state.json` tracks mutable run state.

## Run Phases

- `investigating`
- `planning`
- `bootstrapped`
- `implementing`
- `verifying`
- `reviewing`
- `pr-open`
- `waiting-ci`
- `waiting-review`
- `cleanup`
- `done`

## Todo Rules

Each todo needs:

- stable `id`
- short `title`
- `owner`
- `status`

Allowed statuses:

- `pending`
- `in_progress`
- `blocked`
- `done`

The orchestrator keeps the plan checkboxes and `state.json` in sync.

## Worktree Rules

- In worktree mode, every command starts with `cd <worktree-path> &&`.
- The main repo path is only for worktree lifecycle commands.
- Subagents always receive the absolute worktree path.
- If the run says `worktree` and the active session is not in that worktree, stop and fix it before implementation continues.

## Orchestration Rules

- Use `engine-lead`, `integration-lead`, and `quality-lead` for non-trivial work.
- Use `quick-fix` directly only for trivial single-file edits.
- Reviews are sequential:
  1. `spec-reviewer`
  2. `code-quality-reviewer`
  3. `architecture-validator`
  4. `security-auditor` when FFI or unsafe changes are in play

## Completion Rules

The task is not complete when the PR opens. It is complete when:

- all issue requirements are done
- related untracked cleanup needed for this ticket is done
- review feedback is handled
- CI is green
- cleanup is finished or explicitly marked safe to defer

Only defer related work if it is already tracked elsewhere and clearly out of scope.
