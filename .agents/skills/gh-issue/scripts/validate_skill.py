#!/usr/bin/env python3
import argparse
import json
import sys
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve()
DEFAULT_SKILL_DIR = SCRIPT_PATH.parent.parent
REQUIRED_FILES = [
    "SKILL.md",
    "references/workflow-contract.md",
    "references/resume-contract.md",
    "references/evals.md",
    "assets/plan-template.md",
    "assets/state-template.json",
    "assets/prompts/lead-dispatch.md",
    "assets/prompts/review-dispatch.md",
    "scripts/gh_issue_run.py",
    "scripts/gh_issue_workflow.py",
]


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate the gh-issue skill package structure.")
    parser.add_argument("--skill-dir", default=str(DEFAULT_SKILL_DIR))
    args = parser.parse_args()

    skill_dir = Path(args.skill_dir).resolve()
    errors = []
    checked = list(REQUIRED_FILES)
    missing = [path for path in REQUIRED_FILES if not (skill_dir / path).exists()]
    if missing:
        errors.append({"missing": missing})

    skill_path = skill_dir / "SKILL.md"
    if skill_path.exists():
        lines = skill_path.read_text().splitlines()
        if len(lines) > 500:
            errors.append({"SKILL.md": f"too long: {len(lines)} lines"})
        body = "\n".join(lines)
        for expected in [
            "references/workflow-contract.md",
            "references/resume-contract.md",
            "assets/plan-template.md",
            "assets/state-template.json",
            "scripts/gh_issue_run.py",
            "scripts/gh_issue_workflow.py",
            "scripts/validate_skill.py",
        ]:
            if expected not in body:
                errors.append({"SKILL.md": f"missing reference: {expected}"})

    state_template = skill_dir / "assets" / "state-template.json"
    if state_template.exists():
        text = state_template.read_text()
        for key in ['"schema_version"', '"issues"', '"mode"', '"branch"', '"todos"', '"review_gates"', '"pr"', '"cleanup"']:
            if key not in text:
                errors.append({"state-template.json": f"missing key: {key}"})

    plan_template = skill_dir / "assets" / "plan-template.md"
    if plan_template.exists():
        text = plan_template.read_text()
        for marker in ["## Metadata", "## Non-Negotiables", "## Resume Protocol", "## Review Gates", "## Cleanup"]:
            if marker not in text:
                errors.append({"plan-template.md": f"missing section: {marker}"})
        if "## PR Loop" not in text and "## PR and Polling Loop" not in text:
            errors.append({"plan-template.md": "missing section: ## PR Loop"})

    payload = {
        "ok": not errors,
        "valid": not errors,
        "skill_dir": str(skill_dir),
        "checked": checked,
    }
    if errors:
        payload["errors"] = errors
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 1
    payload["validated_files"] = checked
    print(json.dumps(payload, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
