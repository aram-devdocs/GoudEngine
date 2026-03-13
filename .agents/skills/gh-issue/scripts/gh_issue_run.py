#!/usr/bin/env python3
import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve()
SKILL_DIR = SCRIPT_PATH.parent.parent
ASSETS_DIR = SKILL_DIR / "assets"
RUNS_DIR_NAME = Path(".agents/runs/gh-issue")
VALID_TODO_STATUSES = {"pending", "in_progress", "blocked", "done"}


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


def command_init_run(args: argparse.Namespace) -> int:
    repo_root = Path(args.repo_root).resolve()
    run_dir = resolve_run_dir(repo_root, args.primary, args.slug)
    run_dir.mkdir(parents=True, exist_ok=True)

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
            "BRANCH": args.branch,
            "MODE": args.mode,
            "WORKING_DIRECTORY": str(working_directory),
            "MAIN_REPO_PATH": str(main_repo_path),
            "RUN_DIR": str(run_dir),
            "CREATED_AT": utc_now(),
            "ISSUE_SUMMARY": render_issue_summary(issue_titles),
            "PRIMARY_ISSUE": args.primary,
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
    try:
        if args.phase:
            state["phase"] = args.phase
        if args.pr_number is not None:
            state.setdefault("pr", {})["number"] = args.pr_number
        if args.ci_state:
            state.setdefault("pr", {})["ci_state"] = args.ci_state
        if args.review_state:
            state.setdefault("pr", {})["review_state"] = args.review_state
        if args.mark_polled:
            state.setdefault("pr", {})["last_poll"] = utc_now()
        if args.cleanup_removed:
            state.setdefault("cleanup", {})["worktree_removed"] = True
        if args.mark_cleanup_attempt:
            state.setdefault("cleanup", {})["last_attempt"] = utc_now()
        for todo_id, status in parse_assignment(args.todo or []).items():
            set_todo_status(state, todo_id, status)
        for todo_id, owner in parse_assignment(args.todo_owner or []).items():
            set_todo_owner(state, todo_id, owner)
        for gate, verdict in parse_assignment(args.review_gate or []).items():
            state.setdefault("review_gates", {})[gate] = verdict
        for item in args.defer or []:
            if "::" not in item:
                raise ValueError(f"expected TITLE::TRACKING, got: {item}")
            title, tracking = item.split("::", 1)
            state.setdefault("deferred", []).append({"title": title, "tracking": tracking})
    except ValueError as exc:
        return error(str(exc))

    write_state(state_path, state)
    return output({"ok": True, "state_path": str(state_path), "next_todo": next_incomplete_todo(state), "phase": state.get("phase")})


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
    ready_to_merge = ci_state == "success" and review_state in {"approved", "no_reviews"}

    state.setdefault("pr", {})["number"] = int(pr_number)
    state["pr"]["ci_state"] = ci_state
    state["pr"]["review_state"] = review_state
    state["pr"]["last_poll"] = utc_now()
    if ready_to_merge:
        state["phase"] = "cleanup"
    else:
        state["phase"] = "pr"
    write_state(state_path, state)

    return output({
        "ok": True,
        "pr_number": int(pr_number),
        "ci_state": ci_state,
        "review_state": review_state,
        "ready_to_merge": ready_to_merge,
        "phase": state["phase"],
        "checks_count": len(checks),
        "reviews_count": len(reviews),
    })


def command_cleanup_worktree(args: argparse.Namespace) -> int:
    run_dir = Path(args.run_dir).resolve()
    state_path = run_dir / "state.json"
    if not state_path.exists():
        return error("state.json not found", details=str(state_path))
    state = load_json(state_path)
    if state.get("mode") != "worktree":
        return output({"ok": True, "removed": False, "reason": "in-place mode"})

    worktree_path = state.get("worktree_path")
    if not worktree_path:
        return error("missing worktree path")

    ci_state = state.get("pr", {}).get("ci_state")
    review_state = state.get("pr", {}).get("review_state")
    incomplete = [todo["id"] for todo in state.get("todos", []) if todo.get("status") != "done" and todo.get("id") != "cleanup"]
    if ci_state != "success" or review_state not in {"approved", "no_reviews"} or incomplete:
        return error("cleanup blocked", details={"ci_state": ci_state, "review_state": review_state, "incomplete_todos": incomplete})

    state.setdefault("cleanup", {})["last_attempt"] = utc_now()
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
    update_state.add_argument("--ci-state")
    update_state.add_argument("--review-state")
    update_state.add_argument("--review-gate", action="append")
    update_state.add_argument("--todo", action="append")
    update_state.add_argument("--todo-owner", action="append")
    update_state.add_argument("--defer", action="append")
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
