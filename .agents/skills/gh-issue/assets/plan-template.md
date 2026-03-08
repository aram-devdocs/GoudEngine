# Execution Plan: ${ISSUE_TITLES}

## Metadata
- **Issues**: ${ISSUES}
- **Branch**: `${BRANCH}`
- **Mode**: ${MODE}
- **Working directory**: `${WORKING_DIRECTORY}`
- **Main repo path**: `${MAIN_REPO_PATH}`
- **Canonical run directory**: `${RUN_DIR}`
- **Created**: ${CREATED_AT}

## Non-Negotiables
- [ ] Read this file before doing any work.
- [ ] Read `state.json` before doing any work.
- [ ] In worktree mode, every command starts with `cd ${WORKING_DIRECTORY} &&`.
- [ ] Do not implement non-trivial work directly. Dispatch team leads.
- [ ] Run `spec-reviewer` before `code-quality-reviewer`.
- [ ] Stay with the PR until checks pass and review feedback is handled.
- [ ] Do related cleanup now unless it is already tracked elsewhere and out of scope.

## Resume Protocol
- [ ] Run `python3 .agents/skills/gh-issue/scripts/gh_issue_run.py validate-resume --run-dir ${RUN_DIR} --cwd "$PWD" --branch "$(git branch --show-current)"`
- [ ] Read the first todo in `state.json` that is not `done`
- [ ] Confirm branch `${BRANCH}` and working directory `${WORKING_DIRECTORY}` before implementation

## Issue Summary
${ISSUE_SUMMARY}

## Run Bootstrap
- [ ] Create or confirm the branch `${BRANCH}`
- [ ] Create or confirm the canonical run directory `${RUN_DIR}`
- [ ] Create or confirm `plan.md` and `state.json`
- [ ] Update `state.json` phase to `bootstrapped`

## Implementation
- [ ] Dispatch the correct team lead for the first non-trivial batch
- [ ] Keep todos and review gates in `state.json` in sync after every meaningful step
- [ ] Run local verification before review
## Review Gates
- [ ] Run `spec-reviewer`
- [ ] Run `code-quality-reviewer`
- [ ] Run `architecture-validator`
- [ ] Run `security-auditor` if FFI or unsafe changes are involved

## PR Loop
- [ ] Create the PR and record the number in `state.json`
- [ ] Poll PR status until checks pass
- [ ] Address review blockers and warnings
- [ ] Update `state.json` after each PR loop pass

## Cleanup
- [ ] Remove finished worktree when the run is complete and cleanup is safe
- [ ] Mark cleanup status in `state.json`
