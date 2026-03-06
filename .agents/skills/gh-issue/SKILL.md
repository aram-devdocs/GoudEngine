---
name: gh-issue
description: End-to-end GitHub issue resolution. Accepts one or many issue numbers, resolves them all in a single branch/PR. Optional --worktree flag for worktree isolation. Creates plans with hook-enforced checkpoints, implements with proper orchestration, runs review cycles, and creates one PR.
argument-hint: "<issue-number> [issue-number...] [--worktree]"
disable-model-invocation: true
---

# End-to-End Issue Resolution

## Phase 0: Parse Arguments & Setup

Parse `$ARGUMENTS` into:
- **Issue numbers**: one or many integers (e.g. `123` or `123 234 2312`)
- **--worktree flag**: if present, work in an isolated git worktree

All issues are resolved together in a **single branch and single PR**.

Examples:
- `/gh-issue 123` -- single issue, current checkout
- `/gh-issue 123 --worktree` -- single issue, worktree isolation
- `/gh-issue 123 234 2312` -- three issues, one branch, one PR
- `/gh-issue 123 234 2312 --worktree` -- three issues, one branch, one PR, worktree isolation

---

## Phase 1: Investigation

For each issue number:
- [ ] Fetch issue details:
  ```bash
  gh issue view <ISSUE> --json title,body,labels,comments,assignees,milestone
  ```
- [ ] Check for related PRs:
  ```bash
  gh pr list --search "<ISSUE>" --json number,title,state,url
  ```
- [ ] Read prior comments:
  ```bash
  gh issue view <ISSUE> --comments
  ```
- [ ] Comment on the issue that work is starting:
  ```bash
  gh issue comment <ISSUE> --body "Starting work on this issue. Investigating scope and creating implementation plan."
  ```

After investigating all issues, synthesize a unified understanding of the combined scope.

## Phase 2: Branch & Worktree Setup

Derive `<short-slug>` from the primary issue title (2-3 word kebab-case). Use the lowest issue number as the primary.

**Default mode (no --worktree):**
- [ ] Create feature branch:
  ```bash
  git fetch origin main
  git checkout -b agent/issue-<PRIMARY>-<short-slug> origin/main
  ```
- [ ] Record working directory: the repo root (output of `pwd`)

**Worktree mode (--worktree):**
- [ ] Record main repo path: output of `pwd` (before entering worktree)
- [ ] Create isolated worktree:
  ```bash
  git fetch origin main
  git worktree add -b agent/issue-<PRIMARY>-<short-slug> ../GoudEngine-issue-<PRIMARY> origin/main
  ```
- [ ] Record working directory: the absolute path to `../GoudEngine-issue-<PRIMARY>` (use `realpath`)

## Phase 3: Planning (enter plan mode)

- [ ] Read `CLAUDE.md` for project conventions and architecture
- [ ] Read relevant `.agents/rules/*.md` files based on which areas the issues touch
- [ ] Explore the relevant codebase areas thoroughly (discovery-first protocol)
- [ ] **Fill in the Execution Plan Template below -- EVERY `{{PLACEHOLDER}}`**. The template IS the plan. Do not create a different structure.
- [ ] If any issue is ambiguous, comment questions on that issue and STOP until answered

**CRITICAL**: The plan file MUST use `- [ ]` checkbox syntax for every actionable step. The `plan-completion-guard.sh` hook blocks session end if unchecked items remain.

---

### Execution Plan Template

Copy the template below into the plan file. Replace every `{{PLACEHOLDER}}` with actual values. Do not remove any sections.

````markdown
# Execution Plan: {{ISSUE_TITLES}}

## Metadata (DO NOT MODIFY after creation)
- **Issues**: {{COMMA_SEPARATED_ISSUE_NUMBERS}}
- **Branch**: `agent/issue-{{PRIMARY}}-{{SLUG}}`
- **Working directory**: `{{WORKTREE_ABSPATH_OR_REPO_ROOT}}`
- **Main repo path**: `{{MAIN_REPO_PATH}}` (worktree mode only)
- **Mode**: {{worktree|in-place}}
- **Created by skill**: `/gh-issue` (see `.agents/skills/gh-issue/SKILL.md`)
- **Created**: {{TIMESTAMP}}

## Execution Context (READ THIS FIRST)

You are the **ORCHESTRATOR** executing a `/gh-issue` plan.
- Your working directory is `{{WORKTREE_PATH}}`. EVERY bash command must start with
  `cd {{WORKTREE_PATH}} &&`.
- You MUST NOT write .rs/.cs/.py files directly -- dispatch subagents.
- IMPORTANT: You must use tasks, todos, state, and whatever Agents.md / Claude.md in root and each subdirectory direct to ensure all of plan is executed and tracked properly. 
- Read `CLAUDE.md` sections: "Worktree Execution Protocol", "Plan Execution Protocol",
  "Subagent Dispatch Reference".
- Read `.agents/skills/gh-issue/SKILL.md` to understand the full workflow this plan follows.
- Read `.agents/rules/orchestrator-protocol.md` for the three-tier agent hierarchy.

## Issue Summary
{{For each issue: number, title, key requirements, acceptance criteria}}

## Changes Needed
{{Unified analysis of what code changes are required across all issues}}
- Files to modify: {{list with rationale}}
- Files to create: {{list with rationale}}
- Testing strategy: {{TDD approach}}

---

## Phase 1: Implementation

### Step 1.1: {{TEAM_LEAD_TYPE}} -- {{TASK_SUMMARY}}
- [ ] Dispatch `{{engine-lead|integration-lead}}` subagent:
  ```
  You are the {{engine-lead|integration-lead}} for GoudEngine.
  Working directory: {{WORKTREE_PATH}}

  ## Task
  {{SPECIFIC_TASK_DESCRIPTION}}

  ## Files
  {{LIST_OF_FILES_WITH_RATIONALE}}

  ## Requirements
  1. Use TDD: write failing test first, then implement, then refactor.
  2. After changes run: cd {{WORKTREE_PATH}} && cargo check && cargo test && cargo clippy -- -D warnings
  3. Commit with conventional prefix (feat:/fix:/refactor:).
  4. You may dispatch sub-specialists: implementer, test-first-implementer, quick-fix.

  ## Report Back
  - Files changed (with line counts)
  - Tests added/modified
  - Verification results (cargo check, test, clippy)
  - Any concerns or assumptions made
  ```
- [ ] Review engine-lead report. Question gaps. Verify claims.

### Step 1.N: {{REPEAT for each independent task batch}}
(parallel if independent files, sequential if dependent)

## Phase 2: Verification
- [ ] Run: `cd {{WORKTREE_PATH}} && cargo check`
- [ ] Run: `cd {{WORKTREE_PATH}} && cargo fmt --all -- --check`
- [ ] Run: `cd {{WORKTREE_PATH}} && cargo clippy -- -D warnings`
- [ ] Run: `cd {{WORKTREE_PATH}} && cargo test`
- [ ] All pass? If not, dispatch quick-fix or engine-lead to address failures.

## Phase 3: Review Gates (SEQUENTIAL -- hook-enforced)

### Step 3.1: spec-reviewer (FIRST)
- [ ] Dispatch `spec-reviewer` subagent:
  ```
  Review implementation against these issue requirements:
  {{ISSUE_REQUIREMENTS_SUMMARY}}

  Working directory: {{WORKTREE_PATH}}
  Run: cd {{WORKTREE_PATH}} && git diff origin/main --stat
  Then read the changed files.

  Check:
  1. Does the implementation satisfy ALL issue requirements?
  2. Are there missing edge cases or requirements?
  3. Do tests cover the specified behavior?

  You MUST end with a verdict: APPROVED, REJECTED, or CHANGES REQUESTED.
  If not APPROVED, cite specific file:line references.
  ```
- [ ] spec-reviewer verdict: ________

### Step 3.2: code-quality-reviewer (BLOCKED until spec-reviewer APPROVED)
- [ ] Dispatch `code-quality-reviewer` subagent:
  ```
  Review code quality of changes in: {{WORKTREE_PATH}}
  Run: cd {{WORKTREE_PATH}} && git diff origin/main --stat
  Then read the changed files.

  Check against CLAUDE.md anti-patterns:
  1. Logic in SDKs instead of Rust?
  2. Missing #[no_mangle] or #[repr(C)] on FFI?
  3. Unsafe without // SAFETY: comment?
  4. Layer hierarchy violations?
  5. Files exceeding 500 lines?
  6. Tests without assertions?

  You MUST end with a verdict: APPROVED, REJECTED, or CHANGES REQUESTED.
  Cite specific file:line references.
  ```
- [ ] code-quality-reviewer verdict: ________

### Step 3.3: architecture-validator
- [ ] Dispatch `architecture-validator` subagent:
  ```
  Validate dependency flow in: {{WORKTREE_PATH}}
  Run: cd {{WORKTREE_PATH}} && git diff origin/main --name-only
  Check all changed files for layer violations.
  Layer hierarchy: libs(1) -> engine(2) -> ffi(3) -> sdks(4) -> examples(5)
  No upward imports. No same-layer cross-imports.

  End with verdict: APPROVED or REJECTED with specific violations.
  ```
- [ ] architecture-validator verdict: ________

### Step 3.4: security-auditor (ONLY if FFI/unsafe changed)
- [ ] Dispatch `security-auditor` subagent (SEQUENTIAL -- never parallel):
  ```
  Audit FFI and unsafe code in: {{WORKTREE_PATH}}
  {{SPECIFIC_FFI_FILES_CHANGED}}
  Check: null pointer validation, SAFETY comments, memory ownership docs,
  repr(C) on shared structs, C-compatible types only.
  End with verdict.
  ```
- [ ] security-auditor verdict: ________

### Step 3.5: Address rejections
- [ ] If ANY verdict is REJECTED or CHANGES REQUESTED:
  - Dispatch engine-lead/integration-lead to fix cited issues
  - Re-run the failed reviewer(s)
  - Max 2 fix-review iterations per gate

## Phase 4: Create PR
- [ ] Push branch:
  ```bash
  cd {{WORKTREE_PATH}} && git push -u origin agent/issue-{{PRIMARY}}-{{SLUG}}
  ```
- [ ] Read PR template:
  ```bash
  cd {{WORKTREE_PATH}} && cat .github/pull_request_template.md
  ```
- [ ] Create PR with template filled in and `Closes #N` for each issue:
  ```bash
  cd {{WORKTREE_PATH}} && gh pr create \
    --title "{{CONVENTIONAL_PREFIX}}: {{SHORT_DESCRIPTION}}" \
    --body "$(cat <<'PREOF'
  {{FILLED_IN_PR_TEMPLATE}}
  PREOF
  )" --base main
  ```
- [ ] Record PR number: ________

## Phase 5: CI Verification & Review Loop (max 3 iterations)
- [ ] Wait 30s, then check CI:
  ```bash
  cd {{WORKTREE_PATH}} && gh pr checks {{PR_NUMBER}}
  ```
- [ ] If CI fails: diagnose, dispatch quick-fix, push fix commit, re-check
- [ ] Check for review comments:
  ```bash
  cd {{WORKTREE_PATH}} && gh pr view {{PR_NUMBER}} --json reviews --jq '.reviews[-3:]'
  ```
- [ ] If changes requested: fix, push, comment iteration count
- [ ] CI green and reviews addressed? Mark complete.

## Phase 6: Cleanup
- [ ] Comment on each issue:
  ```bash
  gh issue comment {{ISSUE_N}} --body "Resolved in PR #{{PR_NUMBER}}. All review gates passed."
  ```
- [ ] (Worktree mode) Remove worktree:
  ```bash
  cd {{MAIN_REPO_PATH}} && git worktree remove {{WORKTREE_PATH}}
  ```
````

---

## Rules

- Never force-push
- Never skip pre-commit hooks (no `--no-verify`)
- Never implement logic in SDKs -- all logic in Rust
- Follow the layer hierarchy: libs -> engine -> ffi -> sdks -> examples
- If any issue is unclear, ask questions on that issue rather than guessing
- If blocked after 3 attempts at the same problem, comment on the issue explaining the blocker and stop
- Multiple issues = one branch, one plan, one PR (with `Closes #N` for each)
- Every actionable step in the plan MUST be a `- [ ]` checkbox for hook enforcement

## Hook Integration Summary

These hooks fire automatically and enforce governance. The plan checkboxes align with them:

| Hook | When | What It Enforces |
|------|------|-----------------|
| `delegation-guard.sh` | PreToolUse (Write/Edit) | Orchestrator cannot write .rs/.cs/.py directly |
| `review-gate-guard.sh` | PreToolUse (Agent) | spec-reviewer APPROVED before code-quality-reviewer |
| `review-verdict-validator.sh` | SubagentStop | Reviewers must produce a verdict |
| `review-gate-tracker.sh` | SubagentStop | Tracks APPROVED/REJECTED verdicts in state |
| `delegation-tracker.sh` | PostToolUse (Agent) | Logs all subagent dispatches for audit |
| `quality-check.sh` | PostToolUse (Write/Edit) | Auto-lints edited files |
| `plan-completion-guard.sh` | Stop | Blocks if unchecked `- [ ]` items remain in plan |
| `governance-completion-check.sh` | Stop | Blocks if impl files changed without delegation/reviews |
| `completion-check.sh` | Stop | Advisory warnings (cargo check, FFI/SDK parity, todo!()) |
| `save-session.sh` | PreCompact | Saves session state for context recovery |
