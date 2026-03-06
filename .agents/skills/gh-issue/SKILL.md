---
name: gh-issue
description: End-to-end GitHub issue resolution. Investigates the issue, creates a plan, implements with proper orchestration, runs review cycles, creates a PR, and handles code review feedback. Use when working on a specific GitHub issue by number.
argument-hint: "[issue-number] [--worktree]"
disable-model-invocation: true
---

# End-to-End Issue Resolution

You are resolving GitHub issue **$ARGUMENTS[0]**.

## Phase 0: Parse Arguments & Setup

Parse the arguments: `$ARGUMENTS`
- First argument is the issue number (required)
- If `--worktree` flag is present, use git worktree isolation

## Phase 1: Investigation

Gather full context on the issue:

1. View the issue details:
   ```bash
   gh issue view $0 --json title,body,labels,comments,assignees,milestone
   ```

2. Check for related PRs:
   ```bash
   gh pr list --search "$0" --json number,title,state,url
   ```

3. Read prior comments and any existing agent plans:
   ```bash
   gh issue view $0 --comments
   ```

4. Comment on the issue that work is starting:
   ```bash
   gh issue comment $0 --body "Starting work on this issue. Investigating scope and creating implementation plan."
   ```

## Phase 2: Branch Setup

**Default mode (no --worktree flag):**
```bash
git fetch origin main
git checkout -b agent/issue-$0-<short-slug> origin/main
```

**Worktree mode (--worktree flag):**
```bash
git fetch origin main
git worktree add -b agent/issue-$0-<short-slug> ../GoudEngine-issue-$0 origin/main
cd ../GoudEngine-issue-$0
```
Replace `<short-slug>` with a 2-3 word kebab-case summary of the issue title.

## Phase 3: Planning

1. Read `AGENTS.md` (or `CLAUDE.md`) for project conventions and architecture
2. Read relevant `.agents/rules/*.md` files based on which areas the issue touches
3. Explore the relevant codebase areas thoroughly
4. Create a structured implementation plan covering:
   - Summary of changes needed
   - Files to modify/create
   - Testing strategy
   - Which agent teams to involve (engine-lead, integration-lead, quality-lead)
5. Ask follow-up questions if the issue is ambiguous -- comment on the issue with questions and STOP until answered. Do not guess at unclear requirements.

## Phase 4: Implementation

Execute the plan using proper orchestration:

1. Follow the project's orchestration flow -- delegate to appropriate team leads:
   - **engine-lead** for Rust core changes
   - **integration-lead** for FFI/SDK changes
   - **quality-lead** for reviews and validation
2. Use conventional commits (`feat:`, `fix:`, `refactor:`, etc.)
3. After each significant change, run verification:
   ```bash
   cargo check
   cargo fmt --all -- --check
   cargo clippy -- -D warnings
   cargo test
   ```

## Phase 5: Review Cycles

Run the full review pipeline before creating the PR:

1. **spec-reviewer** -- validates implementation matches issue requirements
2. **code-quality-reviewer** -- validates patterns, anti-patterns, quality (MUST run AFTER spec-reviewer)
3. **architecture-validator** -- checks layer hierarchy, dependency flow
4. **security-auditor** -- if FFI/unsafe code was touched (sequential only)
5. Address all REJECTED/CHANGES REQUESTED verdicts before proceeding

## Phase 6: Create PR

1. Push the branch:
   ```bash
   git push origin agent/issue-$0-<branch-slug>
   ```

2. Read the PR template:
   ```bash
   cat .github/pull_request_template.md
   ```

3. Create the PR using the template structure. Include `Closes #$0` in the Related Issues field:
   ```bash
   gh pr create --title "<conventional-commit-prefix>: <description>" \
     --body "<filled-in PR template>" \
     --base main
   ```

## Phase 7: Code Review Feedback Loop (max 3 iterations)

After creating the PR, handle automated code review feedback:

1. Wait briefly, then check for review comments:
   ```bash
   gh pr view <pr-number> --comments --json comments
   gh pr reviews <pr-number> --json body,state
   ```

2. If `@claude-review` has not yet been triggered by the CI workflow, and you want a review:
   ```bash
   gh pr comment <pr-number> --body "@claude-review"
   ```

3. Check for review results:
   ```bash
   gh pr view <pr-number> --comments --json comments --jq '.comments[-3:][].body'
   ```

4. If changes are requested:
   - Address each review comment
   - Push fixes with a `fix:` commit
   - Comment `@claude-review` to trigger another review cycle
   - **Maximum 3 review iterations** -- after 3 rounds, leave remaining issues for human review

5. Track iterations with comments:
   ```bash
   gh pr comment <pr-number> --body "<!-- agent-review-round: N --> Addressed review feedback (round N/3)."
   ```

## Phase 8: Cleanup (worktree mode only)

If `--worktree` was used, after the PR is created and pushed:
```bash
cd -
git worktree remove ../GoudEngine-issue-$0
```

## Rules

- Never force-push
- Never skip pre-commit hooks (no `--no-verify`)
- Never implement logic in SDKs -- all logic in Rust
- Follow the layer hierarchy: libs -> engine -> ffi -> sdks -> examples
- If the issue is unclear, ask questions on the issue rather than guessing
- If blocked after 3 attempts at the same problem, comment on the issue explaining the blocker and stop
