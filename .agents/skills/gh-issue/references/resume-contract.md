# Resume Contract

Use this contract whenever a session starts or context clears.

## Resume Order

1. Read `.agents/runs/gh-issue/<primary>-<slug>/plan.md`
2. Read `.agents/runs/gh-issue/<primary>-<slug>/state.json`
3. Run `scripts/gh_issue_run.py validate-resume --run-dir <run-dir>`
4. Resume from the first todo that is not `done`

## What the Next Session Must Retain

- issue list and branch
- mode: `in-place` or `worktree`
- absolute worktree path when worktree mode is active
- review gate ordering
- PR template, Claude review, and CI follow-through requirements
- cleanup state
- any allowed deferrals and their tracking issue or PR

## Failure Cases

Stop and repair the run if any of these are true:

- `plan.md` is missing
- `state.json` is missing
- branch does not match `state.json`
- worktree mode is active but the current directory is outside the recorded worktree
- the plan still has placeholders
- required plan sections are missing
- the run claims `done` while review, CI, or cleanup state still says otherwise

## Session Notes

Local memory files can help, but they are not the contract. The contract is the shared run directory.
