# Workflow Contract

`/gh-issue` creates one shared run directory per issue batch:

```text
.agents/runs/gh-issue/<primary>-<slug>/
  plan.md
  state.json
```

## Run Phases

- `investigating`
- `planning`
- `implementing`
- `verifying`
- `reviewing`
- `pr`
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

## Worktree Rules

- In worktree mode, every command starts with `cd <worktree-path> &&`.
- The main repo path is only for worktree lifecycle commands.
- If the run says `worktree` and the active session is not in that worktree, stop and fix it before implementation continues.

## Orchestration Rules

- Use one implementation agent for the main task.
- Run one `reviewer` pass after implementation.
- Add `security-auditor` only when the change touches FFI, unsafe, or ownership boundaries.

## Completion Rules

The task is complete when:

- issue requirements are implemented
- local verification is complete
- review feedback is handled
- PR status is captured
- cleanup is finished or intentionally deferred
