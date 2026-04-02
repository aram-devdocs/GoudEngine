"""Thin JSON helpers for the debugger runtime surface."""

from __future__ import annotations

import json
from typing import Any, Protocol


class _DebuggerJsonSource(Protocol):
    def get_debugger_snapshot_json(self) -> str: ...
    def get_debugger_manifest_json(self) -> str: ...


def parse_debugger_snapshot(source: _DebuggerJsonSource) -> Any:
    return json.loads(source.get_debugger_snapshot_json())


def parse_debugger_manifest(source: _DebuggerJsonSource) -> Any:
    return json.loads(source.get_debugger_manifest_json())
