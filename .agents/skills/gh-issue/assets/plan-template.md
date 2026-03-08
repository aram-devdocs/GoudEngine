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

For each non-trivial batch, dispatch the correct team lead:

- [ ] **{{TEAM_LEAD_TYPE}}** — {{TASK_SUMMARY}}

  Dispatch prompt:
  ```text
  Working directory: ${WORKING_DIRECTORY}

  Task:
  {{DETAILED_TASK_DESCRIPTION}}

  Files to examine/modify:
  {{FILE_LIST}}

  Constraints:
  - Use TDD where practical
  - Keep changes inside the assigned area
  - Run cargo check and cargo test after changes
  - Report files changed, tests run, verification results, and concerns
  ```

- [ ] Keep todos and review gates in `state.json` in sync after every meaningful step

## Verification
- [ ] `cd ${WORKING_DIRECTORY} && cargo check`
- [ ] `cd ${WORKING_DIRECTORY} && cargo fmt --all -- --check`
- [ ] `cd ${WORKING_DIRECTORY} && cargo clippy -- -D warnings`
- [ ] `cd ${WORKING_DIRECTORY} && cargo test`

## Review Gates

- [ ] **spec-reviewer** (MUST run first)

  ```text
  Working directory: ${WORKING_DIRECTORY}
  Diff base: origin/main

  Check:
  - requirements coverage against issue acceptance criteria
  - code quality and anti-patterns
  - layer boundaries

  End with a verdict: APPROVED, REJECTED, or CHANGES REQUESTED.
  Cite specific files and line numbers for any problem.
  ```

  Verdict: {{SPEC_VERDICT}}

- [ ] **code-quality-reviewer** (MUST run after spec-reviewer)

  ```text
  Working directory: ${WORKING_DIRECTORY}
  Diff base: origin/main

  Check:
  - code quality, naming, structure
  - anti-patterns from CLAUDE.md
  - layer boundaries
  - FFI or unsafe handling when relevant

  End with a verdict: APPROVED, REJECTED, or CHANGES REQUESTED.
  Cite specific files and line numbers for any problem.
  ```

  Verdict: {{QUALITY_VERDICT}}

- [ ] **architecture-validator**

  Verdict: {{ARCH_VERDICT}}

- [ ] **security-auditor** if FFI or unsafe changes are involved (sequential only)

  Verdict: {{SECURITY_VERDICT}}

## PR Loop
- [ ] Push changes: `cd ${WORKING_DIRECTORY} && git push -u origin ${BRANCH}`
- [ ] Read PR template: `cat .github/pull_request_template.md`
- [ ] Create PR: `gh pr create --title "{{PR_TITLE}}" --body "$(cat <<'EOF' ... EOF)"`
- [ ] Record the PR number in `state.json`
- [ ] Poll PR status: `gh pr checks {{PR_NUMBER}}`
- [ ] Address review blockers and warnings — push fixes, re-poll
- [ ] Update `state.json` after each PR loop pass

## Cleanup
- [ ] Comment on issue: `gh issue comment ${PRIMARY_ISSUE} --body "Resolved in #{{PR_NUMBER}}"`
- [ ] Remove finished worktree: `cd ${MAIN_REPO_PATH} && git worktree remove ${WORKING_DIRECTORY}`
- [ ] Mark cleanup status in `state.json`
