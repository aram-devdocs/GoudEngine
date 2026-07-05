# Governance and Enforcement

Rules that only live in prose get forgotten. This page maps each governance
invariant to the automated check that enforces it, so the rule and its tripwire
stay together.

## Enforcement map

| Invariant | Enforced by | Failure mode |
|---|---|---|
| No secret ever committed | `secret-scanner.sh` (PreToolUse Write/Edit/MultiEdit) + `gitleaks` (CI) | Write blocked (exit 2); CI job fails |
| No destructive/bypass shell command | `dangerous-cmd-guard.sh` (PreToolUse Bash) | Command blocked (exit 2) |
| Reviewers end with an explicit verdict | `review-verdict-validator.sh` (SubagentStop) + `.agents/rules/challenge-protocol.md` | Subagent stop blocked until a verdict is present |
| Subagents get an adversarial mandate | `challenge-injector.sh` (PreToolUse Task) | Mandate injected into the dispatch |
| Written code stays formatted | `quality-check.sh` (PostToolUse) + `fmt` step of `verify.sh` | Auto-formatted; `verify.sh` fails on drift |
| Session state survives compaction | `save-session.sh` (PreCompact) + `context-loader.sh` (SessionStart) + `.agents/rules/compaction.md` | `SESSION.md` written / restored |
| Uncommitted or debug leftovers at stop | `completion-check.sh` (Stop) | One advisory block, then allowed |
| Layer hierarchy holds | `cargo run -p lint-layers` (verify.sh + CI) | `verify.sh` / CI fails |
| Generated code is not hand-edited | `check-generated-artifacts.sh`, `check-agents-md.sh` (`.mdc` scan) | `verify.sh` / CI fails |
| Skills reference only real files | `validate-skills.py` (verify.sh + CI) | `verify.sh` / CI fails |
| Hooks behave as specified | `test-hooks.sh` (verify.sh + CI) | `verify.sh` / CI fails |
| Local gate == CI gate | `check-gate-parity.py` (verify.sh) | `verify.sh` fails |
| Commits are conventional, not bypasses | `commit-msg` hook | Commit rejected |

## Red flags — stop and fix the cause

When you catch yourself reaching for one of these, the code or the rule is wrong.
Stop and address the cause; do not bypass the gate.

- Reaching for `--no-verify`, `[skip ci]`, or disabling a hook.
- Adding `#[allow(...)]`, `# noqa`, or `eslint-disable` without an inline reason.
- Hand-editing a `*.g.rs`, `*.g.cs`, `*.g.ts`, or `generated/` file.
- Adding a dependency or changing config that was not part of the request.
- Making a claim about the code without having read the current source.

## When a practice becomes a rule

When an agent repeats a mistake, check whether a rule should have caught it. If
there is no rule, add one. If there is, it was not clear enough — sharpen it. Then
add or tighten the validator so the next occurrence is caught deterministically
rather than trusted.
