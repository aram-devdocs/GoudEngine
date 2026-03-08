# Evals

Run these checks whenever the skill changes.

## Local Validation

- `python3 .agents/skills/gh-issue/scripts/validate_skill.py`
- `python3 -m unittest discover .agents/skills/gh-issue/tests`
- `.agents/skills/gh-issue/tests/smoke.sh`

If `skills-ref` is available, run `skills-ref validate .agents/skills/gh-issue` as an extra check.

## Behavioral Prompts

Use these prompt shapes to spot regressions in Claude and Codex:

1. `/gh-issue 101`
2. `/gh-issue 101 --worktree`
3. `/gh-issue 101 205 --worktree`
4. `/gh-issue 101` after a context clear with an active PR

## Acceptance Checks

For every prompt above, confirm that the generated run:

- creates `plan.md` and `state.json`
- records worktree metadata when requested
- creates todos immediately after plan acceptance
- routes non-trivial work through team leads
- keeps the PR open loop active until checks and reviews are clean
- carries enough detail for the next session to resume without re-deriving the rules
