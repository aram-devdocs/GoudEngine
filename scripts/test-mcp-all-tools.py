#!/usr/bin/env python3
"""E2E test: exercise all 20 MCP tools against a live sandbox."""

import json
import os
import signal
import subprocess
import sys
import time
import glob

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TMPDIR = os.environ.get("TMPDIR", "/tmp")
RUNTIME_DIR = os.path.join(TMPDIR, "goudengine")

SANDBOX_BIN = os.path.join(REPO_ROOT, "target", "release", "sandbox")
MCP_BIN = os.path.join(REPO_ROOT, "target", "release", "goudengine-mcp")

PASSED = 0
FAILED = 0
ERRORS = []


def log(msg):
    print(f"[mcp-e2e] {msg}", flush=True)


def check(label, ok, detail=""):
    global PASSED, FAILED
    if ok:
        PASSED += 1
        log(f"PASS: {label}" + (f" ({detail})" if detail else ""))
    else:
        FAILED += 1
        ERRORS.append(label)
        log(f"FAIL: {label}" + (f" ({detail})" if detail else ""))
    return ok


def wait_for_manifest(label, timeout=15):
    deadline = time.time() + timeout
    while time.time() < deadline:
        for f in glob.glob(os.path.join(RUNTIME_DIR, "runtime-*.json")):
            try:
                with open(f) as fp:
                    m = json.load(fp)
                for r in m.get("routes", []):
                    if r.get("label") == label:
                        return f, m
            except (json.JSONDecodeError, IOError):
                pass
        time.sleep(0.5)
    return None, None


class McpClient:
    def __init__(self, binary):
        self.proc = subprocess.Popen(
            [binary],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self._id = 0

    def call(self, method, params, timeout=10):
        self._id += 1
        req = json.dumps({
            "jsonrpc": "2.0",
            "id": self._id,
            "method": method,
            "params": params,
        })
        self.proc.stdin.write((req + "\n").encode())
        self.proc.stdin.flush()

        # Read one line response
        import select
        ready, _, _ = select.select([self.proc.stdout], [], [], timeout)
        if not ready:
            return {"error": "timeout"}
        line = self.proc.stdout.readline().decode().strip()
        if not line:
            return {"error": "empty response"}
        return json.loads(line)

    def notify(self, method, params=None):
        req = json.dumps({"jsonrpc": "2.0", "method": method})
        if params:
            req = json.dumps({"jsonrpc": "2.0", "method": method, "params": params})
        self.proc.stdin.write((req + "\n").encode())
        self.proc.stdin.flush()

    def tool(self, name, args=None, timeout=10):
        params = {"name": name, "arguments": args or {}}
        resp = self.call("tools/call", params, timeout=timeout)
        if "result" in resp:
            content = resp["result"].get("content", [])
            if content and "text" in content[0]:
                try:
                    return json.loads(content[0]["text"]), None
                except json.JSONDecodeError:
                    return content[0]["text"], None
            return resp["result"], None
        return None, resp.get("error", "unknown error")

    def close(self):
        try:
            self.proc.stdin.close()
        except Exception:
            pass
        try:
            self.proc.terminate()
            self.proc.wait(timeout=5)
        except Exception:
            self.proc.kill()


def run_full_tool_test(sdk_label, context_id, process_nonce):
    """Run all 20 MCP tools against an attached sandbox."""
    mcp = McpClient(MCP_BIN)

    try:
        # 1. Initialize
        resp = mcp.call("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "e2e-all-tools", "version": "1.0"},
        })
        check(f"{sdk_label}: MCP initialize", "result" in resp)
        mcp.notify("notifications/initialized")
        time.sleep(0.3)

        # 2. list_contexts
        data, err = mcp.tool("goudengine.list_contexts")
        check(f"{sdk_label}: list_contexts", data is not None and "contexts" in data,
              f"{len(data.get('contexts', []))} contexts" if data else str(err))

        # 3. attach_context
        data, err = mcp.tool("goudengine.attach_context", {
            "contextId": context_id,
            "processNonce": process_nonce,
        })
        if not check(f"{sdk_label}: attach_context", data is not None,
                     f"session={data.get('session', {}).get('session_id', '?')}" if data else str(err)):
            log(f"Cannot continue without attach. Skipping remaining tools for {sdk_label}.")
            return

        # 4. get_snapshot
        data, err = mcp.tool("goudengine.get_snapshot")
        check(f"{sdk_label}: get_snapshot", data is not None,
              f"frame={data.get('frame', {}).get('index', '?')}" if data else str(err))

        # 5. inspect_entity — use an entity id from the snapshot
        snapshot_data = data
        snapshot_entities = snapshot_data.get("entities", []) if snapshot_data else []
        if snapshot_entities:
            first_entity_id = snapshot_entities[0].get("entity_id", 1)
            data, err = mcp.tool("goudengine.inspect_entity", {"entityId": first_entity_id})
            check(f"{sdk_label}: inspect_entity", data is not None, str(err) if err else "")
        else:
            check(f"{sdk_label}: inspect_entity", True, "skipped (no entities in snapshot)")

        # 6. get_diagnostics
        data, err = mcp.tool("goudengine.get_diagnostics")
        if data and "diagnostics" in data:
            keys = sorted(data["diagnostics"].keys())
            check(f"{sdk_label}: get_diagnostics", True, f"{len(keys)} subsystems: {','.join(keys)}")
        else:
            check(f"{sdk_label}: get_diagnostics", False, str(err))

        # 7. get_subsystem_diagnostics (render)
        data, err = mcp.tool("goudengine.get_subsystem_diagnostics", {"key": "render"})
        check(f"{sdk_label}: get_subsystem_diagnostics(render)", data is not None, str(err) if err else "")

        # 8. get_logs
        data, err = mcp.tool("goudengine.get_logs")
        if data:
            entries = data.get("entries", [])
            check(f"{sdk_label}: get_logs", True, f"{len(entries)} entries")
        else:
            check(f"{sdk_label}: get_logs", False, str(err))

        # 9. get_scene_hierarchy
        data, err = mcp.tool("goudengine.get_scene_hierarchy")
        if data:
            entities = data.get("entities", [])
            check(f"{sdk_label}: get_scene_hierarchy", True, f"{len(entities)} entities")
        else:
            check(f"{sdk_label}: get_scene_hierarchy", False, str(err))

        # 10. set_paused(true)
        data, err = mcp.tool("goudengine.set_paused", {"paused": True})
        check(f"{sdk_label}: set_paused(true)", data is not None, str(err) if err else "")

        # 11. step(3 ticks)
        data, err = mcp.tool("goudengine.step", {"kind": "tick", "count": 3})
        check(f"{sdk_label}: step(3)", data is not None, str(err) if err else "")

        # 12. set_time_scale(0.5)
        data, err = mcp.tool("goudengine.set_time_scale", {"scale": 0.5})
        check(f"{sdk_label}: set_time_scale(0.5)", data is not None, str(err) if err else "")

        # 13. inject_input
        data, err = mcp.tool("goudengine.inject_input", {
            "events": [{"device": "mouse", "action": "move", "position": [100.0, 200.0]}]
        })
        check(f"{sdk_label}: inject_input", data is not None, str(err) if err else "")

        # 14. set_paused(false)
        data, err = mcp.tool("goudengine.set_paused", {"paused": False})
        check(f"{sdk_label}: set_paused(false)", data is not None, str(err) if err else "")

        # 15. set_time_scale(1.0) - restore
        data, err = mcp.tool("goudengine.set_time_scale", {"scale": 1.0})
        check(f"{sdk_label}: set_time_scale(1.0)", data is not None, str(err) if err else "")

        # 16. get_metrics_trace
        data, err = mcp.tool("goudengine.get_metrics_trace")
        check(f"{sdk_label}: get_metrics_trace", data is not None,
              f"artifact={data.get('artifact_id', '?')}" if data else str(err))

        # 17. capture_frame
        data, err = mcp.tool("goudengine.capture_frame")
        check(f"{sdk_label}: capture_frame", data is not None,
              f"artifact={data.get('artifact_id', '?')}" if data else str(err))

        # 18. start_recording + stop_recording
        data, err = mcp.tool("goudengine.start_recording")
        check(f"{sdk_label}: start_recording", data is not None, str(err) if err else "")
        time.sleep(1)
        data, err = mcp.tool("goudengine.stop_recording")
        recording_id = data.get("artifact_id", "") if data else ""
        check(f"{sdk_label}: stop_recording", data is not None,
              f"artifact={recording_id}" if data else str(err))

        # 19. start_replay
        if recording_id:
            data, err = mcp.tool("goudengine.start_replay", {"artifactId": recording_id})
            check(f"{sdk_label}: start_replay", data is not None, str(err) if err else "")

            # 20. stop_replay
            data, err = mcp.tool("goudengine.stop_replay")
            check(f"{sdk_label}: stop_replay", data is not None, str(err) if err else "")
        else:
            check(f"{sdk_label}: start_replay", False, "no recording_id")
            check(f"{sdk_label}: stop_replay", False, "skipped")

        # 21. record_diagnostics (one-shot convenience tool)
        data, err = mcp.tool("goudengine.record_diagnostics", {
            "durationSeconds": 1.0,
            "sliceCount": 5,
        }, timeout=15)
        check(f"{sdk_label}: record_diagnostics", data is not None,
              f"export slices={len(data.get('export', {}).get('slices', []))}" if data else str(err))

    finally:
        mcp.close()


def run_sandbox(sdk_label, start_cmd, env_extra=None, timeout_sec=30, cwd=None):
    """Start a sandbox, find its manifest, run all tool tests, then clean up."""
    log(f"=== {sdk_label} sandbox E2E test ===")

    # Clean stale manifests
    for f in glob.glob(os.path.join(RUNTIME_DIR, "runtime-*.json")):
        os.remove(f)
    os.makedirs(RUNTIME_DIR, exist_ok=True)

    env = os.environ.copy()
    env["GOUD_SANDBOX_SMOKE_SECONDS"] = str(timeout_sec)
    if env_extra:
        env.update(env_extra)

    run_dir = cwd or REPO_ROOT
    log(f"Starting {sdk_label} sandbox ({timeout_sec}s runtime)...")
    proc = subprocess.Popen(start_cmd, env=env, cwd=run_dir)
    log(f"Sandbox PID: {proc.pid}")

    try:
        log(f"Waiting for manifest (route: {sdk_label})...")
        manifest_file, manifest = wait_for_manifest(sdk_label, timeout=20)
        if not manifest_file:
            check(f"{sdk_label}: manifest published", False, "not found within 20s")
            return
        check(f"{sdk_label}: manifest published", True, os.path.basename(manifest_file))

        # Extract context_id and process_nonce
        context_id = None
        for r in manifest.get("routes", []):
            if r.get("label") == sdk_label:
                context_id = r["route_id"]["context_id"]
                break
        process_nonce = manifest.get("process_nonce")

        if context_id is None:
            check(f"{sdk_label}: context discovered", False, "no matching route")
            return
        check(f"{sdk_label}: context discovered", True, f"id={context_id}")

        run_full_tool_test(sdk_label, context_id, process_nonce)

    finally:
        log(f"Stopping {sdk_label} sandbox...")
        try:
            proc.terminate()
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
        log(f"{sdk_label} sandbox stopped")


def main():
    sdk = sys.argv[1] if len(sys.argv) > 1 else "rust"

    if sdk in ("rust", "all"):
        run_sandbox(
            "sandbox-rust",
            [SANDBOX_BIN],
            timeout_sec=40,
        )

    if sdk in ("csharp", "all"):
        run_sandbox(
            "sandbox-csharp",
            ["./dev.sh", "--game", "sandbox", "--local"],
            timeout_sec=40,
        )

    if sdk in ("python", "all"):
        lib_path = os.path.join(REPO_ROOT, "target", "release")
        sdk_path = os.path.join(REPO_ROOT, "sdks", "python")
        run_sandbox(
            "sandbox-python",
            ["python3", "examples/python/sandbox.py"],
            env_extra={
                "PYTHONPATH": sdk_path,
                "DYLD_LIBRARY_PATH": lib_path,
                "LD_LIBRARY_PATH": lib_path,
            },
            timeout_sec=40,
        )

    if sdk in ("typescript", "ts-desktop", "all"):
        ts_dir = os.path.join(REPO_ROOT, "examples", "typescript", "sandbox")
        run_sandbox(
            "sandbox-typescript-desktop",
            ["npx", "tsx", "desktop.ts"],
            timeout_sec=40,
            cwd=ts_dir,
        )

    # Summary
    print()
    log("=========================================")
    log(f"Results: {PASSED} passed, {FAILED} failed")
    if FAILED > 0:
        log("Failures:")
        for e in ERRORS:
            log(f"  - {e}")
        sys.exit(1)
    log("All E2E MCP tests passed!")


if __name__ == "__main__":
    main()
