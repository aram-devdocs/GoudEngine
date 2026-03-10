#!/usr/bin/env python3
"""Unit tests for scripts/sync-agent-configs.py."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parent / "sync-agent-configs.py"
SPEC = importlib.util.spec_from_file_location("sync_agent_configs", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"Unable to load module from {MODULE_PATH}")
SYNC = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SYNC)

try:
    import tomllib  # type: ignore[attr-defined]

    def parse_toml(text: str):
        return tomllib.loads(text)
except ModuleNotFoundError:  # pragma: no cover
    import toml  # type: ignore[import-not-found]

    def parse_toml(text: str):
        return toml.loads(text)


class SyncAgentConfigsTests(unittest.TestCase):
    def test_multiline_basic_string_supports_apostrophes(self) -> None:
        value = "I'd write docs exactly like this.\n"
        encoded = SYNC.toml_multiline_basic(value)
        parsed = parse_toml(f"developer_instructions = {encoded}\n")
        self.assertEqual(parsed["developer_instructions"], value)

    def test_multiline_basic_string_rejects_triple_quote(self) -> None:
        with self.assertRaises(SYNC.CatalogError):
            SYNC.toml_multiline_basic('contains """ triple quote')

    def test_validate_catalog_rejects_tier_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            spec_path = Path(tmpdir) / "default.md"
            spec_path.write_text("# default\n", encoding="utf-8")

            catalog = {
                "version": 1,
                "settings": {
                    "root_model": "gpt-5.4",
                    "root_plan_mode_reasoning_effort": "xhigh",
                    "max_threads": 5,
                    "max_depth": 2,
                    "job_max_runtime_seconds": 1800,
                },
                "role_order": ["default"],
                "roles": {
                    "default": {
                        "description": "fallback",
                        "spec_file": str(spec_path),
                        "tier": "fast",
                        "codex_model": "gpt-5.4",
                        "codex_model_reasoning_effort": "high",
                        "codex_sandbox_mode": "workspace-write",
                        "nickname_candidates": ["Northstar"],
                        "claude_enabled": False,
                    }
                },
            }

            with self.assertRaises(SYNC.CatalogError):
                SYNC.validate_catalog(catalog)

    def test_render_codex_config_uses_relative_agent_paths(self) -> None:
        catalog = {
            "settings": {
                "root_model": "gpt-5.4",
                "root_plan_mode_reasoning_effort": "xhigh",
                "max_threads": 5,
                "max_depth": 2,
                "job_max_runtime_seconds": 1800,
            },
            "role_order": ["default"],
            "roles": {
                "default": {
                    "description": "fallback",
                    "nickname_candidates": ["Northstar"],
                }
            },
        }

        rendered = SYNC.render_codex_config(catalog)
        self.assertIn('config_file = "agents/default.toml"', rendered)
        self.assertIn('nickname_candidates = ["Northstar"]', rendered)


if __name__ == "__main__":
    unittest.main()
