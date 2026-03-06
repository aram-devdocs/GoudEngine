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

**Worktree mode (--worktree):**
- [ ] Create isolated worktree:
  ```bash
  git fetch origin main
  git worktree add -b agent/issue-<PRIMARY>-<short-slug> ../GoudEngine-issue-<PRIMARY> origin/main
  cd ../GoudEngine-issue-<PRIMARY>
  ```

## Phase 3: Planning (enter plan mode)

- [ ] Read `CLAUDE.md` for project conventions and architecture
- [ ] Read relevant `.agents/rules/*.md` files based on which areas the issues touch
- [ ] Explore the relevant codebase areas thoroughly (discovery-first protocol)
- [ ] Create a structured plan with ALL of the following as checkboxed items:
  - [ ] Summary of combined changes needed across all issues
  - [ ] Files to modify/create (with rationale)
  - [ ] Testing strategy (TDD: red-green-refactor)
  - [ ] Which agent teams to involve (engine-lead, integration-lead, quality-lead)
  - [ ] Review cycle checkpoints (spec-reviewer, code-quality-reviewer, architecture-validator)
  - [ ] PR creation and CI verification steps
- [ ] If any issue is ambiguous, comment questions on that issue and STOP until answered

**CRITICAL**: The plan file MUST use `- [ ]` checkbox syntax for every actionable step. The `plan-completion-guard.sh` hook blocks session end if unchecked items remain. Every review gate, verification command, and feedback loop must be a checkbox -- not just prose.

### Required Plan Checkboxes

The plan MUST include these checkpoint items (copy them into the plan):

```markdown
### Implementation
- [ ] Implement changes following TDD (red-green-refactor)
- [ ] Run `cargo check` after each significant change
- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy -- -D warnings`
- [ ] Run `cargo test`
- [ ] Commit with conventional prefix (feat:/fix:/refactor:/etc.)

### Review Gates (hook-enforced ordering)
- [ ] Run spec-reviewer -- validate implementation matches issue requirements
- [ ] Receive spec-reviewer verdict: APPROVED
- [ ] Run code-quality-reviewer -- validate patterns, anti-patterns, quality
- [ ] Receive code-quality-reviewer verdict: APPROVED
- [ ] Run architecture-validator -- check layer hierarchy, dependency flow
- [ ] Run security-auditor (if FFI/unsafe touched -- sequential only)
- [ ] Address all REJECTED/CHANGES REQUESTED verdicts

### PR & CI
- [ ] Push branch to origin
- [ ] Create PR with conventional commit title and `Closes #<ISSUE>` for each issue
- [ ] Verify CI passes (plan-completion-guard checks this)
- [ ] Handle code review feedback (max 3 iterations)

### Cleanup (worktree mode only)
- [ ] Remove worktree after PR is created and pushed
```

## Phase 4: Implementation

Execute the plan using proper orchestration:

- [ ] Follow the project's orchestration flow -- delegate to appropriate team leads:
  - **engine-lead** for Rust core changes
  - **integration-lead** for FFI/SDK changes
  - **quality-lead** for reviews and validation
- [ ] Use conventional commits (`feat:`, `fix:`, `refactor:`, etc.)
- [ ] After each significant change, run verification:
  ```bash
  cargo check
  cargo fmt --all -- --check
  cargo clippy -- -D warnings
  cargo test
  ```
- [ ] Governance hooks enforce: delegation-guard (no direct .rs/.cs/.py writes from orchestrator), delegation-tracker (audit trail), quality-check (auto-lint on edit)

## Phase 5: Review Cycles (hook-enforced gate ordering)

Run the full review pipeline before creating the PR. The `review-gate-guard.sh` hook enforces ordering.

- [ ] **spec-reviewer** -- validates implementation matches ALL issue requirements
- [ ] Confirm spec-reviewer verdict is APPROVED (tracked by `review-gate-tracker.sh`)
- [ ] **code-quality-reviewer** -- validates patterns, anti-patterns, quality (BLOCKED until spec-reviewer APPROVED)
- [ ] Confirm code-quality-reviewer verdict is APPROVED
- [ ] **architecture-validator** -- checks layer hierarchy, dependency flow
- [ ] **security-auditor** -- if FFI/unsafe code was touched (sequential only, never parallelized)
- [ ] Address ALL REJECTED/CHANGES REQUESTED verdicts before proceeding
- [ ] Re-run failed review gates after fixes (review-verdict-validator.sh enforces verdicts exist)

## Phase 6: Create PR

- [ ] Push the branch:
  ```bash
  git push -u origin agent/issue-<PRIMARY>-<branch-slug>
  ```
- [ ] Read the PR template:
  ```bash
  cat .github/pull_request_template.md
  ```
- [ ] Create the PR. Include `Closes #<N>` for EVERY issue number in the Related Issues field:
  ```bash
  gh pr create --title "<conventional-commit-prefix>: <description>" \
    --body "<filled-in PR template with Closes #N for each issue>" \
    --base main
  ```

## Phase 7: Code Review Feedback Loop (max 3 iterations)

After creating the PR, handle automated code review feedback:

- [ ] Check for review comments:
  ```bash
  gh pr view <pr-number> --comments --json comments
  gh pr reviews <pr-number> --json body,state
  ```
- [ ] If changes are requested:
  - [ ] Address each review comment
  - [ ] Push fixes with a `fix:` commit
  - [ ] Re-verify: `cargo check && cargo test`
  - [ ] Comment iteration count:
    ```bash
    gh pr comment <pr-number> --body "<!-- agent-review-round: N --> Addressed review feedback (round N/3)."
    ```
- [ ] **Maximum 3 review iterations** -- after 3 rounds, leave remaining issues for human review
- [ ] Verify CI passes on final push (plan-completion-guard checks this at session end)

## Phase 8: Cleanup

**Worktree mode only:**
- [ ] After PR is created and pushed, remove the worktree:
  ```bash
  cd /Users/aramhammoudeh/dev/game/GoudEngine
  git worktree remove ../GoudEngine-issue-<PRIMARY>
  ```

**All modes:**
- [ ] Comment final status on each issue:
  ```bash
  gh issue comment <ISSUE> --body "PR #<pr-number> created. All review gates passed."
  ```

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

## Rules

- Never force-push
- Never skip pre-commit hooks (no `--no-verify`)
- Never implement logic in SDKs -- all logic in Rust
- Follow the layer hierarchy: libs -> engine -> ffi -> sdks -> examples
- If any issue is unclear, ask questions on that issue rather than guessing
- If blocked after 3 attempts at the same problem, comment on the issue explaining the blocker and stop
- Multiple issues = one branch, one plan, one PR (with `Closes #N` for each)
- Every actionable step in the plan MUST be a `- [ ]` checkbox for hook enforcement
