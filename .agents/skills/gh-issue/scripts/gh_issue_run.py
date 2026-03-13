#!/usr/bin/env python3
import argparse
import copy
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve()
SKILL_DIR = SCRIPT_PATH.parent.parent
ASSETS_DIR = SKILL_DIR / "assets"
RUNS_DIR_NAME = Path(".agents/runs/gh-issue")
VALID_TODO_STATUSES = {"pending", "in_progress", "blocked", "done"}
REQUIRED_PLAN_SECTIONS = [
    "## Metadata",
    "## Non-Negotiables",
    "## Resume Protocol",
    "## Issue Summary",
    "## Implementation Batches",
    "## Verification Matrix",
    "## Review Gates",
    "## PR Creation",
    "## Claude Review Loop",
    "## CI Loop",
    "## Cleanup",
]
REVIEW_GATE_ORDER = ["spec-reviewer", "code-quality-reviewer", "architecture-validator", "security-auditor"]
CONVENTIONAL_TITLE_RE = re.compile(
    r"^(feat|fix|refactor|docs|test|chore|ci|build|perf)(\([^)]+\))?!?: .+"
)
BRANCH_RE = re.compile(r"^codex/issue-(?P<primary>\d+)-[a-z0-9]+(?:-[a-z0-9]+)*$")


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def load_json(path: Path) -> dict:
    return json.loads(path.read_text())


def dump_json(data: dict) -> str:
    return json.dumps(data, indent=2, sort_keys=True)


def output(payload: dict, *, code: int = 0) -> int:
    sys.stdout.write(dump_json(payload) + "\n")
    return code


def error(message: str, *, details=None, code: int = 2) -> int:
    payload = {"ok": False, "error": message}
    if details is not None:
        payload["details"] = details
    return output(payload, code=code)


def write_file(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def write_state(path: Path, state: dict) -> None:
    write_file(path, dump_json(state) + "\n")


def resolve_run_dir(repo_root: Path, primary: str, slug: str) -> Path:
    return repo_root / RUNS_DIR_NAME / f"{primary}-{slug}"


def render_issue_summary(issue_titles: list[str]) -> str:
    return "\n".join(f"- {entry}" for entry in issue_titles) if issue_titles else "- No issue titles recorded"


def render_text(template_text: str, replacements: dict[str, str]) -> str:
    rendered = template_text
    for key, value in replacements.items():
        rendered = rendered.replace(f"${{{key}}}", value)
        rendered = rendered.replace(f"__{key}__", value)
    return rendered


def read_template(name: str) -> str:
    return (ASSETS_DIR / name).read_text()


def read_prompt(name: str) -> str:
    return (ASSETS_DIR / "prompts" / name).read_text().rstrip()


def read_state_template() -> dict:
    return json.loads((ASSETS_DIR / "state-template.json").read_text())


def git_branch() -> str:
    try:
        result = subprocess.run(["git", "branch", "--show-current"], check=True, text=True, capture_output=True)
    except Exception:
        return ""
    return result.stdout.strip()


def next_incomplete_todo(state: dict):
    for todo in state.get("todos", []):
        if todo.get("status") != "done":
            return todo
    return None


def set_todo_status(state: dict, todo_id: str, status: str) -> None:
    if status not in VALID_TODO_STATUSES:
        raise ValueError(f"invalid todo status: {status}")
    for todo in state.get("todos", []):
        if todo.get("id") == todo_id:
            todo["status"] = status
            return
    raise ValueError(f"unknown todo id: {todo_id}")


def set_todo_owner(state: dict, todo_id: str, owner: str) -> None:
    for todo in state.get("todos", []):
        if todo.get("id") == todo_id:
            todo["owner"] = owner
            return
    raise ValueError(f"unknown todo id: {todo_id}")


def parse_assignment(values: list[str], separator: str = "=") -> dict:
    parsed = {}
    for item in values:
        if separator not in item:
            raise ValueError(f"expected KEY{separator}VALUE, got: {item}")
        key, value = item.split(separator, 1)
        parsed[key] = value
    return parsed


def plan_contains_placeholders(plan_text: str) -> bool:
    return "${" in plan_text or "__" in plan_text


def missing_plan_sections(plan_text: str) -> list[str]:
    return [section for section in REQUIRED_PLAN_SECTIONS if section not in plan_text]


def validate_branch_name(branch: str, primary: str | None = None) -> list[str]:
    match = BRANCH_RE.match(branch)
    if not match:
        return [f"branch must match codex/issue-<primary>-<slug>, got: {branch}"]
    if primary is not None and match.group("primary") != str(primary):
        return [f"branch primary mismatch: expected {primary}, got {match.group('primary')}"]
    return []


def is_conventional_title(title: str) -> bool:
    return bool(CONVENTIONAL_TITLE_RE.match(title.strip()))


def accepted_gate_verdict(gate: str, verdict: str) -> bool:
    if gate == "architecture-validator":
        return verdict in {"VALID", "APPROVED"}
    if gate == "security-auditor":
        return verdict in {"APPROVED", "not_required"}
    return verdict == "APPROVED"


def validate_review_gate_order(state: dict) -> list[str]:
    gates = state.get("review_gates", {})
    errors = []
    spec = gates.get("spec-reviewer", "pending")
    quality = gates.get("code-quality-reviewer", "pending")
    architecture = gates.get("architecture-validator", "pending")
    security = gates.get("security-auditor", "not_required")

    if quality != "pending" and spec != "APPROVED":
        errors.append("code-quality-reviewer cannot run before spec-reviewer is APPROVED")
    if architecture != "pending" and quality != "APPROVED":
        errors.append("architecture-validator cannot run before code-quality-reviewer is APPROVED")
    if security not in {"pending", "not_required"} and architecture not in {"VALID", "APPROVED"}:
        errors.append("security-auditor cannot run before architecture-validator is VALID")
    return errors


def review_gates_complete(state: dict) -> bool:
    gates = state.get("review_gates", {})
    if not accepted_gate_verdict("spec-reviewer", gates.get("spec-reviewer", "pending")):
        return False
    if not accepted_gate_verdict("code-quality-reviewer", gates.get("code-quality-reviewer", "pending")):
        return False
    if not accepted_gate_verdict("architecture-validator", gates.get("architecture-validator", "pending")):
        return False
    return accepted_gate_verdict("security-auditor", gates.get("security-auditor", "not_required"))


def claude_feedback_cleared(state: dict) -> bool:
    review = state.get("claude_review", {})
    status = review.get("status", "pending")
    blockers = review.get("blockers_status", "pending")
    warnings = review.get("warnings_status", "pending")
    justification = review.get("justification_recorded", False)

    if status in {"pending", "no_review"}:
        return False
    if blockers not in {"fixed", "not_required"}:
        return False
    if warnings == "justified" and not justification:
        return False
    return warnings in {"fixed", "not_required", "justified"}


def pr_metadata_complete(state: dict) -> bool:
    pr = state.get("pr", {})
    naming = state.get("naming", {})
    return bool(
        pr.get("number")
        and pr.get("template_used")
        and pr.get("template_sections_complete")
        and naming.get("branch_valid")
        and naming.get("pr_title_valid")
        and naming.get("commit_titles_state") == "valid"
    )


def run_ready_for_cleanup(state: dict) -> bool:
    return (
        review_gates_complete(state)
        and pr_metadata_complete(state)
        and claude_feedback_cleared(state)
        and state.get("ci", {}).get("status") == "success"
    )


def completion_errors(state: dict) -> list[str]:
    errors = []
    if not run_ready_for_cleanup(state):
        errors.append("run is not ready for cleanup")
    if state.get("mode") == "worktree" and not state.get("cleanup", {}).get("worktree_removed"):
        errors.append("worktree mode requires cleanup.worktree_removed = true before done")
    incomplete = [
        todo["id"]
        for todo in state.get("todos", [])
        if todo.get("status") != "done"
    ]
    if incomplete:
        errors.append(f"incomplete todos: {', '.join(incomplete)}")
    return errors


def summarize_ci(checks: list[dict]) -> str:
    states = [item.get("state", "") for item in checks]
    if not states:
        return "unknown"
    if any(state == "FAILURE" for state in states):
        return "failure"
    if any(state in {"PENDING", "IN_PROGRESS", "QUEUED", "STARTUP_FAILURE"} for state in states):
        return "pending"
    if all(state == "SUCCESS" for state in states):
        return "success"
    return "unknown"


def summarize_reviews(reviews: list[dict]) -> str:
    states = [item.get("state", "") for item in reviews]
    if any(state == "CHANGES_REQUESTED" for state in states):
        return "changes_requested"
    if any(state == "COMMENTED" for state in states):
        return "commented"
    if any(state == "APPROVED" for state in states):
        return "approved"
    return "no_reviews"


def is_claude_author(review: dict) -> bool:
    author = review.get("author") or {}
    haystacks = [author.get("login", ""), author.get("name", "")]
    lowered = " ".join(part.lower() for part in haystacks if part)
    return "claude" in lowered or "anthropic" in lowered


def summarize_claude_review(reviews: list[dict]) -> tuple[str, str | None]:
    claude_reviews = [review for review in reviews if is_claude_author(review)]
    if not claude_reviews:
        return "no_review", None
    author = (claude_reviews[-1].get("author") or {}).get("login")
    states = [review.get("state", "") for review in claude_reviews]
    if any(state == "CHANGES_REQUESTED" for state in states):
        return "changes_requested", author
    if any(state == "COMMENTED" for state in states):
        return "commented", author
    if any(state == "APPROVED" for state in states):
        return "approved", author
    return "pending", author


def load_checks_and_reviews(args: argparse.Namespace, pr_number: int):
    if args.checks_json_file:
        checks = load_json(Path(args.checks_json_file))
    else:
        result = subprocess.run(["gh", "pr", "checks", str(pr_number), "--json", "name,state"], check=True, text=True, capture_output=True)
        checks = json.loads(result.stdout)

    if args.reviews_json_file:
        reviews = load_json(Path(args.reviews_json_file))
    else:
        result = subprocess.run(["gh", "pr", "view", str(pr_number), "--json", "reviews"], check=True, text=True, capture_output=True)
        reviews = json.loads(result.stdout).get("reviews", [])
    return checks, reviews


def command_init_run(args: argparse.Namespace) -> int:
    repo_root = Path(args.repo_root).resolve()
    run_dir = resolve_run_dir(repo_root, args.primary, args.slug)
    run_dir.mkdir(parents=True, exist_ok=True)

    branch_errors = validate_branch_name(args.branch, args.primary)
    if branch_errors:
        return error("invalid branch", details=branch_errors)

    working_directory = Path(args.working_directory).resolve()
    main_repo_path = Path(args.main_repo_path).resolve() if args.main_repo_path else repo_root
    issue_titles = args.issue_title or [f"#{issue}" for issue in args.issues]

    state = read_state_template()
    state.update(
        {
            "run_id": f"{args.primary}-{args.slug}",
            "primary_issue": args.primary,
            "issues": args.issues,
            "issue_titles": issue_titles,
            "mode": args.mode,
            "branch": args.branch,
            "run_dir": str(run_dir),
            "worktree_path": str(working_directory),
            "main_repo_path": str(main_repo_path),
            "phase": "bootstrapped",
        }
    )
    state["naming"]["branch_valid"] = True
    for todo in state["todos"]:
        if todo["id"] == "bootstrap":
            todo["status"] = "done"
    state_path = run_dir / "state.json"
    write_state(state_path, state)

    plan = render_text(
        read_template("plan-template.md"),
        {
            "ISSUE_TITLES": ", ".join(issue_titles),
            "ISSUES": ", ".join(args.issues),
            "PRIMARY_ISSUE": args.primary,
            "BRANCH": args.branch,
            "MODE": args.mode,
            "WORKING_DIRECTORY": str(working_directory),
            "MAIN_REPO_PATH": str(main_repo_path),
            "RUN_DIR": str(run_dir),
            "CREATED_AT": utc_now(),
            "ISSUE_SUMMARY": render_issue_summary(issue_titles),
            "LEAD_DISPATCH_PROMPT": read_prompt("lead-dispatch.md"),
            "REVIEW_DISPATCH_PROMPT": read_prompt("review-dispatch.md"),
            "PR_CREATION_PROMPT": read_prompt("pr-creation.md"),
            "FEEDBACK_TRIAGE_PROMPT": read_prompt("feedback-triage.md"),
            "CI_POLLING_PROMPT": read_prompt("ci-polling.md"),
            "CLEANUP_COMPLETION_PROMPT": read_prompt("cleanup-completion.md"),
        },
    )
    plan_path = run_dir / "plan.md"
    write_file(plan_path, plan)

    return output(
        {
            "ok": True,
            "run_dir": str(run_dir),
            "plan_path": str(plan_path),
            "state_path": str(state_path),
            "next_todo": next_incomplete_todo(state),
        }
    )


def command_update_state(args: argparse.Namespace) -> int:
    run_dir = Path(args.run_dir).resolve()
    state_path = run_dir / "state.json"
    if not state_path.exists():
        return error("state.json not found", details=str(state_path))

    state = load_json(state_path)
    updated = copy.deepcopy(state)
    try:
        if args.phase:
            updated["phase"] = args.phase
        if args.pr_number is not None:
            updated.setdefault("pr", {})["number"] = args.pr_number
        if args.pr_title is not None:
            if not is_conventional_title(args.pr_title):
                raise ValueError(f"invalid PR title: {args.pr_title}")
            updated.setdefault("pr", {})["title"] = args.pr_title
            updated.setdefault("naming", {})["pr_title_valid"] = True
        if args.ci_state:
            updated.setdefault("ci", {})["status"] = args.ci_state
        if args.review_state:
            updated.setdefault("pr", {})["review_state"] = args.review_state
        if args.pr_template_used:
            updated.setdefault("pr", {})["template_used"] = True
        if args.pr_template_sections_complete:
            updated.setdefault("pr", {})["template_sections_complete"] = True
        if args.commit_titles_state:
            updated.setdefault("naming", {})["commit_titles_state"] = args.commit_titles_state
        if args.claude_review_state:
            updated.setdefault("claude_review", {})["status"] = args.claude_review_state
        if args.claude_review_author:
            updated.setdefault("claude_review", {})["author"] = args.claude_review_author
        if args.claude_blockers_status:
            updated.setdefault("claude_review", {})["blockers_status"] = args.claude_blockers_status
        if args.claude_warnings_status:
            updated.setdefault("claude_review", {})["warnings_status"] = args.claude_warnings_status
        if args.claude_justification_recorded:
            updated.setdefault("claude_review", {})["justification_recorded"] = True
        if args.mark_polled:
            updated.setdefault("pr", {})["last_poll"] = utc_now()
            updated.setdefault("ci", {})["last_poll"] = utc_now()
            updated.setdefault("claude_review", {})["last_poll"] = utc_now()
        if args.cleanup_removed:
            updated.setdefault("cleanup", {})["worktree_removed"] = True
        if args.mark_cleanup_attempt:
            updated.setdefault("cleanup", {})["last_attempt"] = utc_now()
        for todo_id, status in parse_assignment(args.todo or []).items():
            set_todo_status(updated, todo_id, status)
        for todo_id, owner in parse_assignment(args.todo_owner or []).items():
            set_todo_owner(updated, todo_id, owner)
        for gate, verdict in parse_assignment(args.review_gate or []).items():
            updated.setdefault("review_gates", {})[gate] = verdict
        for item in args.defer or []:
            if "::" not in item:
                raise ValueError(f"expected TITLE::TRACKING, got: {item}")
            title, tracking = item.split("::", 1)
            updated.setdefault("deferred", []).append({"title": title, "tracking": tracking})
        if updated.get("claude_review", {}).get("warnings_status") == "justified" and not updated.get("claude_review", {}).get("justification_recorded"):
            raise ValueError("warnings_status=justified requires justification_recorded=true")
        gate_errors = validate_review_gate_order(updated)
        if gate_errors:
            raise ValueError("; ".join(gate_errors))
        if updated.get("phase") == "done":
            errors = completion_errors(updated)
            if errors:
                raise ValueError("; ".join(errors))
    except ValueError as exc:
        return error(str(exc))

    write_state(state_path, updated)
    return output(
        {
            "ok": True,
            "state_path": str(state_path),
            "next_todo": next_incomplete_todo(updated),
            "phase": updated.get("phase"),
        }
    )


def command_validate_resume(args: argparse.Namespace) -> int:
    run_dir = Path(args.run_dir).resolve()
    plan_path = run_dir / "plan.md"
    state_path = run_dir / "state.json"
    errors = []

    if not plan_path.exists():
        errors.append(f"missing plan: {plan_path}")
    if not state_path.exists():
        errors.append(f"missing state: {state_path}")
    if errors:
        return error("resume validation failed", details=errors)

    plan_text = plan_path.read_text()
    state = load_json(state_path)
    cwd = Path(args.cwd).resolve() if args.cwd else Path.cwd().resolve()
    branch = args.branch if args.branch is not None else git_branch()

    if plan_contains_placeholders(plan_text):
        errors.append("plan still contains placeholders")
    missing_sections = missing_plan_sections(plan_text)
    if missing_sections:
        errors.append(f"missing plan sections: {', '.join(missing_sections)}")
    errors.extend(validate_branch_name(state.get("branch", ""), state.get("primary_issue")))
    if branch and branch != state.get("branch"):
        errors.append(f"branch mismatch: expected {state.get('branch')}, got {branch}")

    if state.get("mode") == "worktree":
        worktree_path = state.get("worktree_path")
        if not worktree_path:
            errors.append("worktree mode requires worktree_path")
        else:
            recorded = Path(worktree_path).resolve()
            try:
                cwd.relative_to(recorded)
            except ValueError:
                if cwd != recorded:
                    errors.append(f"cwd {cwd} is outside worktree {recorded}")

    errors.extend(validate_review_gate_order(state))
    if state.get("phase") == "done":
        errors.extend(completion_errors(state))

    payload = {
        "ok": not errors,
        "run_dir": str(run_dir),
        "branch": state.get("branch"),
        "mode": state.get("mode"),
        "next_todo": next_incomplete_todo(state),
    }
    if errors:
        payload["errors"] = errors
        return output(payload, code=2)
    return output(payload)


def command_poll_pr(args: argparse.Namespace) -> int:
    run_dir = Path(args.run_dir).resolve()
    state_path = run_dir / "state.json"
    if not state_path.exists():
        return error("state.json not found", details=str(state_path))
    state = load_json(state_path)
    pr_number = args.pr_number if args.pr_number is not None else state.get("pr", {}).get("number")
    if not pr_number:
        return error("pr number is required")

    try:
        checks, reviews = load_checks_and_reviews(args, int(pr_number))
    except subprocess.CalledProcessError as exc:
        return error("failed to poll GitHub", details=exc.stderr.strip())

    ci_state = summarize_ci(checks)
    review_state = summarize_reviews(reviews)
    claude_state, claude_author = summarize_claude_review(reviews)

    state.setdefault("pr", {})["number"] = int(pr_number)
    state["pr"]["review_state"] = review_state
    state["pr"]["last_poll"] = utc_now()
    state.setdefault("ci", {})["status"] = ci_state
    state["ci"]["last_poll"] = utc_now()
    state.setdefault("claude_review", {})["status"] = claude_state
    state["claude_review"]["author"] = claude_author
    state["claude_review"]["last_poll"] = utc_now()

    if claude_state == "approved":
        state["claude_review"].setdefault("blockers_status", "not_required")
        state["claude_review"].setdefault("warnings_status", "not_required")
        if state["claude_review"]["blockers_status"] == "pending":
            state["claude_review"]["blockers_status"] = "not_required"
        if state["claude_review"]["warnings_status"] == "pending":
            state["claude_review"]["warnings_status"] = "not_required"

    if run_ready_for_cleanup(state):
        state["phase"] = "cleanup"
    elif claude_state in {"no_review", "pending"}:
        state["phase"] = "waiting-claude"
    elif ci_state != "success":
        state["phase"] = "waiting-ci"
    else:
        state["phase"] = "reviewing"
    write_state(state_path, state)

    return output(
        {
            "ok": True,
            "pr_number": int(pr_number),
            "ci_state": ci_state,
            "review_state": review_state,
            "claude_review_state": claude_state,
            "ready_for_cleanup": run_ready_for_cleanup(state),
            "phase": state["phase"],
            "checks_count": len(checks),
            "reviews_count": len(reviews),
        }
    )


def command_cleanup_worktree(args: argparse.Namespace) -> int:
    run_dir = Path(args.run_dir).resolve()
    state_path = run_dir / "state.json"
    if not state_path.exists():
        return error("state.json not found", details=str(state_path))
    state = load_json(state_path)
    state.setdefault("cleanup", {})["last_attempt"] = utc_now()

    if not run_ready_for_cleanup(state):
        write_state(state_path, state)
        return error(
            "cleanup blocked",
            details={
                "review_gates_complete": review_gates_complete(state),
                "pr_metadata_complete": pr_metadata_complete(state),
                "claude_feedback_cleared": claude_feedback_cleared(state),
                "ci_status": state.get("ci", {}).get("status"),
            },
        )

    if state.get("mode") != "worktree":
        state["phase"] = "done"
        for todo in state.get("todos", []):
            if todo.get("id") == "cleanup":
                todo["status"] = "done"
        write_state(state_path, state)
        return output({"ok": True, "removed": False, "reason": "in-place mode"})

    worktree_path = state.get("worktree_path")
    if not worktree_path:
        return error("missing worktree path")

    if not args.confirm:
        write_state(state_path, state)
        return output({"ok": True, "removed": False, "dry_run": True, "worktree_path": worktree_path})

    main_repo_path = state.get("main_repo_path")
    try:
        subprocess.run(["git", "-C", main_repo_path, "worktree", "remove", worktree_path], check=True, text=True, capture_output=True)
    except subprocess.CalledProcessError as exc:
        return error("worktree removal failed", details=exc.stderr.strip())

    state["cleanup"]["worktree_removed"] = True
    state["phase"] = "done"
    for todo in state.get("todos", []):
        if todo.get("id") == "cleanup":
            todo["status"] = "done"
    write_state(state_path, state)
    return output({"ok": True, "removed": True, "dry_run": False, "worktree_path": worktree_path})


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Deterministic gh-issue workflow helper")
    subparsers = parser.add_subparsers(dest="command", required=True)

    init_run = subparsers.add_parser("init-run", help="Create canonical plan and state artifacts")
    init_run.add_argument("--repo-root", required=True)
    init_run.add_argument("--primary", required=True)
    init_run.add_argument("--slug", required=True)
    init_run.add_argument("--branch", required=True)
    init_run.add_argument("--mode", choices=["in-place", "worktree"], required=True)
    init_run.add_argument("--working-directory", required=True)
    init_run.add_argument("--main-repo-path")
    init_run.add_argument("--issues", nargs="+", required=True)
    init_run.add_argument("--issue-title", action="append")
    init_run.set_defaults(func=command_init_run)

    update_state = subparsers.add_parser("update-state", help="Update canonical run state")
    update_state.add_argument("--run-dir", required=True)
    update_state.add_argument("--phase")
    update_state.add_argument("--pr-number", type=int)
    update_state.add_argument("--pr-title")
    update_state.add_argument("--ci-state", choices=["not_started", "pending", "success", "failure", "unknown"])
    update_state.add_argument("--review-state")
    update_state.add_argument("--review-gate", action="append")
    update_state.add_argument("--todo", action="append")
    update_state.add_argument("--todo-owner", action="append")
    update_state.add_argument("--defer", action="append")
    update_state.add_argument("--pr-template-used", action="store_true")
    update_state.add_argument("--pr-template-sections-complete", action="store_true")
    update_state.add_argument("--commit-titles-state", choices=["pending", "valid", "invalid"])
    update_state.add_argument("--claude-review-state", choices=["pending", "no_review", "approved", "commented", "changes_requested"])
    update_state.add_argument("--claude-review-author")
    update_state.add_argument("--claude-blockers-status", choices=["pending", "fixed", "not_required"])
    update_state.add_argument("--claude-warnings-status", choices=["pending", "fixed", "justified", "not_required"])
    update_state.add_argument("--claude-justification-recorded", action="store_true")
    update_state.add_argument("--mark-polled", action="store_true")
    update_state.add_argument("--cleanup-removed", action="store_true")
    update_state.add_argument("--mark-cleanup-attempt", action="store_true")
    update_state.set_defaults(func=command_update_state)

    validate_resume = subparsers.add_parser("validate-resume", help="Validate that a session can resume safely")
    validate_resume.add_argument("--run-dir", required=True)
    validate_resume.add_argument("--cwd")
    validate_resume.add_argument("--branch")
    validate_resume.set_defaults(func=command_validate_resume)

    poll_pr = subparsers.add_parser("poll-pr", help="Summarize PR checks and reviews")
    poll_pr.add_argument("--run-dir", required=True)
    poll_pr.add_argument("--pr-number", type=int)
    poll_pr.add_argument("--checks-json-file")
    poll_pr.add_argument("--reviews-json-file")
    poll_pr.set_defaults(func=command_poll_pr)

    cleanup = subparsers.add_parser("cleanup-worktree", help="Remove the recorded worktree when the run is complete")
    cleanup.add_argument("--run-dir", required=True)
    cleanup.add_argument("--confirm", action="store_true")
    cleanup.set_defaults(func=command_cleanup_worktree)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
