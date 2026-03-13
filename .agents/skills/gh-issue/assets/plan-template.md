# Execution Plan: ${ISSUE_TITLES}

## Metadata
- **Issues**: ${ISSUES}
- **Primary issue**: ${PRIMARY_ISSUE}
- **Branch**: `${BRANCH}`
- **Mode**: ${MODE}
- **Working directory**: `${WORKING_DIRECTORY}`
- **Main repo path**: `${MAIN_REPO_PATH}`
- **Canonical run directory**: `${RUN_DIR}`
- **Created**: ${CREATED_AT}

## Non-Negotiables
- [ ] Read this file before doing any work.
- [ ] Read `state.json` before doing any work.
- [ ] Keep the branch name as `${BRANCH}`.
- [ ] Use one implementation lead at a time.
- [ ] Run review gates in order: `spec-reviewer` -> `code-quality-reviewer` -> `architecture-validator` -> `security-auditor` when required.
- [ ] Use `.github/pull_request_template.md` for the PR body.
- [ ] Wait for GitHub Claude review and handle blockers and warnings before considering the run ready.
- [ ] Wait for CI to be green before cleanup.
- [ ] Remove the local worktree during cleanup.

## Resume Protocol
- [ ] Run `python3 .agents/skills/gh-issue/scripts/gh_issue_run.py validate-resume --run-dir ${RUN_DIR} --cwd "$PWD" --branch "$(git branch --show-current)"`
- [ ] Resume from the first todo in `state.json` that is not `done`
- [ ] Confirm the session is running inside `${WORKING_DIRECTORY}`

## Issue Summary
${ISSUE_SUMMARY}

## Implementation Batches
- [ ] Investigate scope and related cleanup with `gh issue view` and `gh pr list`
- [ ] Dispatch the correct lead role using the prompt below

```text
${LEAD_DISPATCH_PROMPT}
```

- [ ] Record implementation progress in `state.json`

## Verification Matrix
- [ ] `cd ${WORKING_DIRECTORY} && cargo check`
- [ ] `cd ${WORKING_DIRECTORY} && cargo fmt --all -- --check`
- [ ] `cd ${WORKING_DIRECTORY} && cargo clippy -- -D warnings`
- [ ] `cd ${WORKING_DIRECTORY} && cargo test`
- [ ] Run the smallest SDK/codegen checks needed for the changed surface

## Review Gates
- [ ] `spec-reviewer`
- [ ] `code-quality-reviewer`
- [ ] `architecture-validator`
- [ ] `security-auditor` when FFI, unsafe, pointers, or ownership boundaries changed

```text
${REVIEW_DISPATCH_PROMPT}
```

## PR Creation
- [ ] Push the branch `${BRANCH}`
- [ ] Create the PR with a conventional title and the repo PR template
- [ ] Record PR number, PR title, and template completion in `state.json`

```text
${PR_CREATION_PROMPT}
```

## Claude Review Loop
- [ ] Poll for GitHub Claude review
- [ ] Fix all blockers
- [ ] Fix all warnings or record a concrete justification in the PR reply
- [ ] Record Claude review state, blocker status, warning status, and justification status in `state.json`

```text
${FEEDBACK_TRIAGE_PROMPT}
```

## CI Loop
- [ ] Poll CI checks until all required jobs are green
- [ ] Record CI status in `state.json`

```text
${CI_POLLING_PROMPT}
```

## Cleanup
- [ ] Verify review gates, PR metadata, Claude review handling, and CI are complete
- [ ] Remove the local worktree
- [ ] Mark cleanup complete in `state.json`

```text
${CLEANUP_COMPLETION_PROMPT}
```
