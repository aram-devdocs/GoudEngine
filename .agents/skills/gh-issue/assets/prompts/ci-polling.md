Working directory: <absolute worktree path>

Requirements:
- Poll PR checks until all required jobs are green.
- Record the latest CI status and poll time in `state.json`.
- Do not move to cleanup until CI is fully green.
