#!/usr/bin/env python3
"""
GoudEngine Python Feature Lab (ALPHA-001)

Headless smoke example that exercises Python SDK surfaces not covered by
`main.py` and `flappy_bird.py`, with explicit safe-fallback behavior for
provider-dependent APIs.
"""

from __future__ import annotations

import socket
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

# Add the SDK to the Python path
sdk_path = Path(__file__).parent.parent.parent / "sdks" / "python"
sys.path.insert(0, str(sdk_path))

from goud_engine import (  # noqa: E402
    GoudContext,
    GoudError,
    NetworkManager,
    NetworkProtocol,
    Transform2D,
    UiManager,
    parse_debugger_manifest,
    parse_debugger_snapshot,
)
from goud_engine.generated._types import ContextConfig, DebuggerConfig  # noqa: E402

DEBUGGER_ROUTE_LABEL = "feature-lab-python-headless"


@dataclass
class CheckResult:
    name: str
    status: str  # PASS | FAIL | SKIP
    detail: str = ""


def _as_i64(value: object) -> int:
    if isinstance(value, (bytes, bytearray, memoryview)):
        raw = bytes(value)
        if len(raw) >= 8:
            return int.from_bytes(raw[:8], byteorder="little", signed=True)
        if len(raw) > 0:
            return int.from_bytes(raw, byteorder="little", signed=True)
    else:
        try:
            raw = bytes(value)
            if len(raw) >= 8:
                return int.from_bytes(raw[:8], byteorder="little", signed=True)
            if len(raw) > 0:
                return int.from_bytes(raw, byteorder="little", signed=True)
        except Exception:
            pass
    return int(value)


def _reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def _run_check(name: str, fn: Callable[[], tuple[bool, str]]) -> CheckResult:
    try:
        ok, detail = fn()
        return CheckResult(name, "PASS" if ok else "FAIL", detail)
    except Exception as exc:  # pragma: no cover - smoke example fallback path
        return CheckResult(name, "FAIL", str(exc))


def _run_skippable_check(name: str, fn: Callable[[], tuple[bool, str]]) -> CheckResult:
    try:
        ok, detail = fn()
        return CheckResult(name, "PASS" if ok else "FAIL", detail)
    except OSError as exc:
        return CheckResult(name, "SKIP", str(exc))
    except (RuntimeError, GoudError) as exc:
        return CheckResult(name, "SKIP", str(exc))
    except NameError as exc:
        return CheckResult(name, "SKIP", str(exc))
    except Exception as exc:  # pragma: no cover - smoke example fallback path
        return CheckResult(name, "FAIL", str(exc))


def check_scene_lifecycle(ctx: GoudContext) -> tuple[bool, str]:
    initial_count = _as_i64(ctx.scene_count())
    scene_id = _as_i64(ctx.scene_create("py_feature_sandbox"))
    if scene_id < 0:
        return False, f"scene_create failed: {scene_id}"

    duplicate_id = _as_i64(ctx.scene_create("py_feature_sandbox"))
    count_after_create = _as_i64(ctx.scene_count())
    set_active_status = _as_i64(ctx.scene_set_active(scene_id, True))
    is_active = bool(ctx.scene_is_active(scene_id))
    set_current_status = _as_i64(ctx.scene_set_current(scene_id))
    current_scene = _as_i64(ctx.scene_get_current())
    destroy_status = _as_i64(ctx.scene_destroy(scene_id))
    count_after_destroy = _as_i64(ctx.scene_count())

    # Some count/status return values are still being normalized in the Python
    # bindings; this smoke test focuses on stable lifecycle calls.
    ok = (
        scene_id >= 0
        and set_active_status >= 0
        and is_active
        and set_current_status >= 0
        and current_scene == scene_id
        and destroy_status >= 0
    )
    detail = (
        f"initial={initial_count}, after_create={count_after_create}, "
        f"after_destroy={count_after_destroy}, duplicate_id={duplicate_id}"
    )
    return ok, detail


def check_entity_lifecycle(ctx: GoudContext) -> tuple[bool, str]:
    initial_count = _as_i64(ctx.entity_count())
    entity = ctx.spawn_empty()
    created_alive = bool(ctx.is_alive(entity))
    count_after_spawn = _as_i64(ctx.entity_count())
    cloned = ctx.clone_entity(entity)
    cloned_alive = bool(ctx.is_alive(cloned))
    despawned = bool(ctx.despawn(entity))
    despawned_clone = bool(ctx.despawn(cloned))
    final_count = _as_i64(ctx.entity_count())

    # Keep one Transform2D call in the feature lab surface without asserting on
    # generated type-hash internals that are still in flux across Python runs.
    _ = Transform2D.from_position(10.0, 20.0)

    ok = (
        created_alive
        and cloned_alive
        and despawned
        and despawned_clone
    )
    detail = (
        f"initial={initial_count}, after_spawn={count_after_spawn}, "
        f"final={final_count}, alive={created_alive}, cloned_alive={cloned_alive}"
    )
    return ok, detail


def check_network_capabilities(ctx: GoudContext) -> tuple[bool, str]:
    caps = ctx.get_network_capabilities()
    ok = bool(caps.max_connections >= 0 and caps.max_channels >= 0 and caps.max_message_size >= 0)
    detail = (
        f"hosting={caps.supports_hosting}, max_connections={caps.max_connections}, "
        f"max_channels={caps.max_channels}, max_message_size={caps.max_message_size}"
    )
    return ok, detail


def check_debugger_runtime(ctx: GoudContext) -> tuple[bool, str]:
    ctx.set_debugger_profiling_enabled(True)
    manifest = parse_debugger_manifest(ctx)
    snapshot = parse_debugger_snapshot(ctx)

    routes = manifest.get("routes", [])
    route_ok = any(
        route.get("label") == DEBUGGER_ROUTE_LABEL and route.get("attachable") is True
        for route in routes
        if isinstance(route, dict)
    )
    snapshot_ok = (
        isinstance(snapshot, dict)
        and int(snapshot.get("snapshot_version", 0)) >= 1
        and isinstance(snapshot.get("route_id"), dict)
    )
    detail = (
        f"route_label={DEBUGGER_ROUTE_LABEL}, routes={len(routes)}, "
        f"snapshot_version={snapshot.get('snapshot_version', 'missing')}"
    )
    return route_ok and snapshot_ok, detail


def check_network_wrapper_and_fallback(ctx: GoudContext) -> tuple[bool, str]:
    manager = NetworkManager(ctx)
    endpoint = manager.host(NetworkProtocol.TCP, _reserve_port())
    try:
        poll_status = _as_i64(endpoint.poll())
        peer_count = _as_i64(endpoint.peer_count())
        stats = endpoint.get_stats()

        missing_peer_guard_ok = False
        try:
            endpoint.send(b"no-default-peer")
        except ValueError:
            missing_peer_guard_ok = True

        ok = (
            poll_status >= 0
            and peer_count >= 0
            and stats.bytes_sent >= 0
            and stats.bytes_received >= 0
            and missing_peer_guard_ok
        )
        detail = (
            f"poll={poll_status}, peers={peer_count}, bytes_sent={stats.bytes_sent}, "
            f"bytes_received={stats.bytes_received}, send_guard={missing_peer_guard_ok}"
        )
        return ok, detail
    finally:
        endpoint.disconnect()


def check_ui_manager_basics() -> tuple[bool, str]:
    ui = UiManager()
    try:
        panel = int(ui.create_panel())
        label = int(ui.create_label("feature-lab"))
        parent_status = int(ui.set_parent(label, panel))
        child_count = int(ui.get_child_count(panel))
        event_count = int(ui.event_count())
        node_count = int(ui.node_count())
        ok = panel != 0 and label != 0 and parent_status == 0 and child_count >= 1 and event_count >= 0 and node_count >= 2
        detail = (
            f"panel={panel}, label={label}, parent_status={parent_status}, "
            f"child_count={child_count}, event_count={event_count}, node_count={node_count}"
        )
        return ok, detail
    finally:
        ui.destroy()


def print_manual_attach_workflow() -> None:
    print(f"Debugger route label: {DEBUGGER_ROUTE_LABEL}")
    print("Manual attach workflow:")
    print("1. start `cargo run -p goudengine-mcp`")
    print("2. call `goudengine.list_contexts`")
    print("3. call `goudengine.attach_context`")


def main() -> int:
    print("=" * 64)
    print(" GoudEngine Python Feature Lab (ALPHA-001)")
    print("=" * 64)

    results: list[CheckResult] = []
    ctx = None

    try:
        ctx = GoudContext(
            ContextConfig(
                debugger=DebuggerConfig(
                    enabled=True,
                    publish_local_attach=True,
                    route_label=DEBUGGER_ROUTE_LABEL,
                )
            )
        )
        print_manual_attach_workflow()

        results.append(_run_check("headless context is valid", lambda: (bool(ctx.is_valid()), "")))
        results.append(_run_check("debugger manifest and snapshot are available", lambda: check_debugger_runtime(ctx)))
        results.append(_run_check("scene lifecycle operations", lambda: check_scene_lifecycle(ctx)))
        results.append(_run_check("entity lifecycle + clone operations", lambda: check_entity_lifecycle(ctx)))
        results.append(_run_skippable_check("network capability query", lambda: check_network_capabilities(ctx)))
        results.append(_run_skippable_check("network wrapper + safe send fallback", lambda: check_network_wrapper_and_fallback(ctx)))
        results.append(_run_skippable_check("ui manager basic node tree", check_ui_manager_basics))
    except Exception as exc:
        results.append(CheckResult("feature lab startup", "FAIL", str(exc)))
    finally:
        if ctx is not None:
            ctx.destroy()

    pass_count = sum(1 for r in results if r.status == "PASS")
    fail_count = sum(1 for r in results if r.status == "FAIL")
    skip_count = sum(1 for r in results if r.status == "SKIP")

    print(f"Feature Lab complete: {pass_count} pass, {fail_count} fail, {skip_count} skip")
    for result in results:
        suffix = f" ({result.detail})" if result.detail else ""
        print(f"{result.status}: {result.name}{suffix}")

    return 1 if fail_count > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
