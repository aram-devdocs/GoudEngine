#!/usr/bin/env bash
# SubagentStart hook: inject challenge protocol into every subagent
set -euo pipefail

INPUT=$(cat)
AGENT_TYPE=$(echo "$INPUT" | jq -r '.agent_type // empty')

if [[ -z "$AGENT_TYPE" ]]; then
  exit 0
fi

CHALLENGE=""

case "$AGENT_TYPE" in
  spec-reviewer|code-quality-reviewer|security-auditor)
    CHALLENGE="CHALLENGE PROTOCOL (REVIEWER): You MUST identify at least one substantive concern or risk. No rubber-stamping. If the work is genuinely excellent, explain specifically why each potential concern does not apply. End your review with a clear verdict: APPROVED, REJECTED, or CHANGES REQUESTED."
    ;;
  implementer|ffi-implementer|sdk-implementer|test-first-implementer)
    CHALLENGE="CHALLENGE PROTOCOL (IMPLEMENTER): Before implementing, list 1-2 assumptions you are making about the codebase or requirements. Flag any uncertain assumptions for the orchestrator to confirm. After implementing, run cargo check and cargo test. Report what you changed and what you verified."
    ;;
  engine-lead|integration-lead|quality-lead)
    CHALLENGE="CHALLENGE PROTOCOL (TEAM LEAD): You are a sub-orchestrator. Own this mission completely. Deploy specialist subagents as needed. Question your specialists' output before reporting — do not pass through results uncritically. Apply your own judgment. A plan from the orchestrator is input, not orders. Report concise summaries, not raw output."
    ;;
  debugger)
    CHALLENGE="CHALLENGE PROTOCOL (DEBUGGER): State your confidence level (high/medium/low) for each diagnosis. Trace through the code to confirm root causes — do not guess. If multiple root causes are possible, rank them by likelihood."
    ;;
  architecture-validator)
    CHALLENGE="CHALLENGE PROTOCOL (VALIDATOR): Check every changed file against the layer hierarchy. Report confidence level for each finding. If no violations found, explain which specific checks you performed."
    ;;
  test-runner)
    CHALLENGE="CHALLENGE PROTOCOL (TEST RUNNER): After running tests, verify that the test results are meaningful — not just 'no tests found'. Report both pass counts and any suspicious patterns (e.g., all tests skipped, zero assertions)."
    ;;
  *)
    CHALLENGE="CHALLENGE PROTOCOL: Self-verify your work before reporting. State any assumptions. Flag areas of uncertainty."
    ;;
esac

if [[ -n "$CHALLENGE" ]]; then
  echo "{\"additionalContext\":\"$CHALLENGE\"}"
fi

exit 0
