#!/usr/bin/env bash
# PreToolUse hook (matcher: Task) — inject a short, role-specific mandate into a
# dispatched subagent's prompt via additionalContext. Reviewers get an
# adversarial "raise a concern or justify none, then give a verdict" charge;
# implementers get a "state assumptions, run checks, list changed files" charge.
#
# Fails open: any error, a missing subagent_type, or an unknown role exits 0
# without altering the Task call.
set -uo pipefail

INPUT=$(cat 2>/dev/null || true)
ROLE=$(printf '%s' "$INPUT" | jq -r '.tool_input.subagent_type // empty' 2>/dev/null || true)

# Nothing to inject without a role.
[[ -z "$ROLE" ]] && exit 0

case "$ROLE" in
  security-auditor)
    MANDATE='Adversarial mandate: audit every unsafe block, FFI boundary, raw pointer, and ownership transfer. Confirm each export is #[no_mangle] extern "C" with #[repr(C)] structs, null checks before every dereference, and a // SAFETY: comment on each unsafe block. Raise >=1 concrete concern anchored to file:line, or explicitly justify why none exist, then end with a clear verdict (APPROVED or CHANGES REQUESTED).'
    ;;
  reviewer|spec-reviewer|code-quality-reviewer)
    MANDATE='Adversarial mandate: do not rubber-stamp. Raise >=1 concrete concern anchored to file:line, or explicitly justify why none exist. End with an explicit verdict (APPROVED, REJECTED, or CHANGES REQUESTED).'
    ;;
  engine-lead|integration-lead|quick-fix)
    MANDATE='Implementation mandate: state your assumptions up front, run the required checks (cargo check, cargo fmt --check, cargo clippy -D warnings, plus ./codegen.sh for any FFI/SDK/schema change), and finish by listing every file you changed.'
    ;;
  *)
    # Unknown role — stay silent rather than guess.
    exit 0
    ;;
esac

# jq builds valid, correctly escaped JSON for the additionalContext payload.
jq -cn --arg ctx "$MANDATE" \
  '{hookSpecificOutput: {hookEventName: "PreToolUse", additionalContext: $ctx}}' \
  2>/dev/null || true

exit 0
