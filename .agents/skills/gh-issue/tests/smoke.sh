#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
python3 "$ROOT/scripts/gh_issue_workflow.py" --help >/dev/null
python3 "$ROOT/scripts/gh_issue_workflow.py" init-run --help >/dev/null
python3 "$ROOT/scripts/gh_issue_run.py" --help >/dev/null
python3 "$ROOT/scripts/validate_skill.py" >/dev/null
