# Execution Plan: ${ISSUE_TITLES}

## Metadata
- **Issues**: ${ISSUES}
- **Branch**: `${BRANCH}`
- **Mode**: ${MODE}
- **Working directory**: `${WORKING_DIRECTORY}`
- **Main repo path**: `${MAIN_REPO_PATH}`
- **Canonical run directory**: `${RUN_DIR}`
- **Created**: ${CREATED_AT}

## Summary
${ISSUE_SUMMARY}

## Workflow

### Investigate
- Confirm issue scope, constraints, and related cleanup.

### Implement
- Use one implementation agent for the main change set.
- Prefer `engine-lead` for engine/core work.
- Prefer `integration-lead` for FFI/SDK/codegen work.
- Use `quick-fix` only for tightly scoped follow-up edits.

### Verify
- Run the smallest command set that proves the changed surface is healthy.

### Review
- Run one `reviewer` pass.
- Add `security-auditor` only when the change touches FFI, unsafe, or memory ownership.

### PR
- Push, open or update the PR, poll briefly, then summarize status instead of idling indefinitely.

## Notes
- In worktree mode, commands should start with `cd ${WORKING_DIRECTORY} &&`.
- Use the run state to resume cleanly after interruptions.
