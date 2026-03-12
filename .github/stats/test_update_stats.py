"""Tests for community stats updater helpers."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import update_stats


class UpdateStatsTests(unittest.TestCase):
    def test_pypi_url_excludes_mirrors(self) -> None:
        self.assertIn("mirrors=false", update_stats.PYPI_URL)

    def test_merge_pypi_daily_updates_existing_and_sorts(self) -> None:
        existing = {
            "2026-03-01": 10,
            "2026-03-03": 30,
        }
        latest = {
            "2026-03-02": 20,
            "2026-03-03": 31,
        }

        merged = update_stats.merge_pypi_daily(existing, latest)

        self.assertEqual(
            merged,
            {
                "2026-03-01": 10,
                "2026-03-02": 20,
                "2026-03-03": 31,
            },
        )

    def test_upsert_history_resets_on_bootstrap(self) -> None:
        history = [
            {
                "date": "2026-03-06",
                "crates_io": 9,
                "nuget": 483,
                "pypi": 5358,
                "npm": 513,
                "total": 6363,
            }
        ]
        entry = {
            "date": "2026-03-12",
            "crates_io": 36,
            "nuget": 932,
            "pypi": 2138,
            "npm": 835,
            "total": 3941,
        }

        updated = update_stats.upsert_history(history, entry, reset=True)

        self.assertEqual(updated, [entry])

    def test_upsert_history_replaces_same_day_entry(self) -> None:
        history = [
            {
                "date": "2026-03-11",
                "crates_io": 35,
                "nuget": 900,
                "pypi": 2100,
                "npm": 800,
                "total": 3835,
            },
            {
                "date": "2026-03-12",
                "crates_io": 36,
                "nuget": 920,
                "pypi": 2120,
                "npm": 820,
                "total": 3896,
            },
        ]
        replacement = {
            "date": "2026-03-12",
            "crates_io": 36,
            "nuget": 932,
            "pypi": 2138,
            "npm": 835,
            "total": 3941,
        }

        updated = update_stats.upsert_history(history, replacement, reset=False)

        self.assertEqual(updated[-1], replacement)
        self.assertEqual(len(updated), 2)

    def test_main_preserves_existing_history_when_pypi_state_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            stats_dir = temp_root / ".github" / "stats"
            stats_dir.mkdir(parents=True)

            readme = temp_root / "README.md"
            history = stats_dir / "history.json"
            pypi_daily = stats_dir / "pypi_daily.json"
            readme.write_text(
                "\n".join(
                    [
                        "# Test",
                        "![total downloads](https://img.shields.io/badge/dynamic/json?url=history.json)",
                        "<!-- COMMUNITY-STATS:START -->",
                        "old block",
                        "<!-- COMMUNITY-STATS:END -->",
                    ]
                )
            )
            history.write_text(
                json.dumps(
                    [
                        {
                            "date": "2026-03-11",
                            "crates_io": 30,
                            "nuget": 900,
                            "pypi": 2000,
                            "npm": 800,
                            "total": 3730,
                        }
                    ]
                )
            )

            original_readme = update_stats.README
            original_history = update_stats.HISTORY
            original_pypi_daily = update_stats.PYPI_DAILY
            original_generate_chart = update_stats.generate_chart.main
            original_fetch_crates = update_stats.fetch_crates_total
            original_fetch_nuget = update_stats.fetch_nuget_total
            original_fetch_npm = update_stats.fetch_npm_total
            original_fetch_pypi = update_stats.fetch_pypi_daily_downloads

            try:
                update_stats.README = readme
                update_stats.HISTORY = history
                update_stats.PYPI_DAILY = pypi_daily
                update_stats.generate_chart.main = lambda: None
                update_stats.fetch_crates_total = lambda: 36
                update_stats.fetch_nuget_total = lambda: 932
                update_stats.fetch_npm_total = lambda: 837
                update_stats.fetch_pypi_daily_downloads = lambda: {
                    "2026-03-01": 1000,
                    "2026-03-02": 200,
                }

                update_stats.main()
            finally:
                update_stats.README = original_readme
                update_stats.HISTORY = original_history
                update_stats.PYPI_DAILY = original_pypi_daily
                update_stats.generate_chart.main = original_generate_chart
                update_stats.fetch_crates_total = original_fetch_crates
                update_stats.fetch_nuget_total = original_fetch_nuget
                update_stats.fetch_npm_total = original_fetch_npm
                update_stats.fetch_pypi_daily_downloads = original_fetch_pypi

            history_payload = json.loads(history.read_text())
            self.assertEqual(len(history_payload), 2)
            self.assertEqual(
                set(history_payload[-1].keys()),
                {"date", "crates_io", "nuget", "pypi", "npm", "total"},
            )
            self.assertEqual(history_payload[-1]["crates_io"], 36)
            self.assertEqual(history_payload[-1]["nuget"], 932)
            self.assertEqual(history_payload[-1]["pypi"], 2000)
            self.assertEqual(history_payload[-1]["npm"], 837)
            self.assertEqual(history_payload[-1]["total"], 3805)
            self.assertEqual(
                json.loads(pypi_daily.read_text()),
                {
                    "2026-03-01": 1000,
                    "2026-03-02": 200,
                },
            )

            readme_text = readme.read_text()
            self.assertIn("| crates.io | [36](https://crates.io/crates/goud-engine) |", readme_text)
            self.assertIn("| PyPI | [2,000](https://pypi.org/project/goudengine/) |", readme_text)
            self.assertIn("<sub>PyPI totals exclude mirrors.</sub>", readme_text)
            self.assertIn("![total downloads](https://img.shields.io/badge/dynamic/json?url=history.json)", readme_text)

    def test_main_bootstrap_raises_when_live_fetch_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            stats_dir = temp_root / ".github" / "stats"
            stats_dir.mkdir(parents=True)

            readme = temp_root / "README.md"
            history = stats_dir / "history.json"
            pypi_daily = stats_dir / "pypi_daily.json"
            readme.write_text(
                "\n".join(
                    [
                        "# Test",
                        "<!-- COMMUNITY-STATS:START -->",
                        "old block",
                        "<!-- COMMUNITY-STATS:END -->",
                    ]
                )
            )
            history.write_text("[]\n")

            original_readme = update_stats.README
            original_history = update_stats.HISTORY
            original_pypi_daily = update_stats.PYPI_DAILY
            original_generate_chart = update_stats.generate_chart.main
            original_fetch_crates = update_stats.fetch_crates_total

            try:
                update_stats.README = readme
                update_stats.HISTORY = history
                update_stats.PYPI_DAILY = pypi_daily
                update_stats.generate_chart.main = lambda: None
                update_stats.fetch_crates_total = lambda: (_ for _ in ()).throw(RuntimeError("boom"))

                with self.assertRaises(RuntimeError):
                    update_stats.main()
            finally:
                update_stats.README = original_readme
                update_stats.HISTORY = original_history
                update_stats.PYPI_DAILY = original_pypi_daily
                update_stats.generate_chart.main = original_generate_chart
                update_stats.fetch_crates_total = original_fetch_crates


if __name__ == "__main__":
    unittest.main()
