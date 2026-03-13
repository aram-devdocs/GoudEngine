# Workflow Contract

`/gh-issue` creates one shared run directory per issue batch:

```text
.agents/runs/gh-issue/<primary>-<slug>/
  plan.md
  state.json
```

`plan.md` is the durable execution contract. `state.json` is the mutable run state.

## Run Phases

- `investigating`
- `planning`
- `bootstrapped`
- `implementing`
- `verifying`
- `reviewing`
- `pr`
- `waiting-claude`
- `waiting-ci`
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

- The strict `/gh-issue` path uses a worktree by default.
- In worktree mode, every repo command starts with `cd <worktree-path> &&`.
- The main repo path is only for worktree lifecycle commands.
- If the run says `worktree` and the active session is not in that worktree, stop and repair it before continuing.

## Orchestration Rules

- Use one implementation lead at a time.
- Review gates run in order:
  1. `spec-reviewer`
  2. `code-quality-reviewer`
  3. `architecture-validator`
  4. `security-auditor` when required
- Use `.github/pull_request_template.md` for the PR body.
- Wait for GitHub Claude review and handle blockers and warnings before cleanup.
- Wait for CI to be green before cleanup.

## Completion Rules

The run is complete only when:

- issue requirements are implemented
- local verification is complete
- strict review gates are satisfied
- PR metadata is recorded
- Claude review feedback is handled
- CI is green
- the worktree is removed or the run is explicitly in in-place mode
