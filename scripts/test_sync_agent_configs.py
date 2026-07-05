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
                    "codex_max_threads": 5,
                    "codex_max_depth": 2,
                    "shared_team_mode": "single-lead",
                    "shared_max_specialists_per_lead": 2,
                    "shared_fallback_mode": "single-agent",
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
                "codex_max_threads": 5,
                "codex_max_depth": 2,
                "shared_team_mode": "single-lead",
                "shared_max_specialists_per_lead": 2,
                "shared_fallback_mode": "single-agent",
                "job_max_runtime_seconds": 1800,
            },
            "role_order": ["default"],
            "roles": {
                "default": {
                    "description": "fallback",
                    "nickname_candidates": ["Northstar"],
                    "codex_enabled": True,
                }
            },
        }

        rendered = SYNC.render_codex_config(catalog)
        self.assertIn("# Per-session caps for this Codex run.", rendered)
        self.assertIn("# Unset max_threads to remove the thread bound.", rendered)
        self.assertIn("max_threads = 5", rendered)
        self.assertIn("max_depth = 2", rendered)
        self.assertIn("job_max_runtime_seconds = 1800", rendered)
        self.assertIn('config_file = "agents/default.toml"', rendered)
        self.assertIn('nickname_candidates = ["Northstar"]', rendered)

    def test_parse_rule_frontmatter_extracts_globs_and_body(self) -> None:
        text = '---\nglobs:\n  - "**/ecs/**"\n  - "**/foo/**"\nalwaysApply: false\n---\n# Title\n\nBody.\n'
        front, body = SYNC.parse_rule_frontmatter(text)
        self.assertEqual(front["globs"], ["**/ecs/**", "**/foo/**"])
        self.assertEqual(front["alwaysApply"], "false")
        self.assertTrue(body.startswith("# Title"))

    def test_parse_rule_frontmatter_handles_no_frontmatter(self) -> None:
        text = "# Title\n\nBody only.\n"
        front, body = SYNC.parse_rule_frontmatter(text)
        self.assertEqual(front, {})
        self.assertEqual(body, text)

    def test_render_cursor_rule_scoped(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            rule = Path(tmpdir) / "ecs-patterns.md"
            rule.write_text('---\nglobs:\n  - "**/ecs/**"\n---\n# ECS Patterns\n\nBody.\n', encoding="utf-8")
            rendered = SYNC.render_cursor_rule(rule)
        self.assertIn("description: ECS Patterns", rendered)
        self.assertIn("globs: **/ecs/**", rendered)
        self.assertIn("alwaysApply: false", rendered)  # scoped rule auto-attaches
        self.assertIn(SYNC.GENERATED_MARKER, rendered)
        self.assertIn("# ECS Patterns", rendered)

    def test_render_cursor_rule_unscoped_always_applies(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            rule = Path(tmpdir) / "orchestrator-protocol.md"
            rule.write_text("# Orchestrator Protocol\n\nBody.\n", encoding="utf-8")
            rendered = SYNC.render_cursor_rule(rule)
        self.assertIn("description: Orchestrator Protocol", rendered)
        self.assertIn("alwaysApply: true", rendered)  # no globs -> always on


if __name__ == "__main__":
    unittest.main()
