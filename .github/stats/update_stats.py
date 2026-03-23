"""Update community stats artifacts from live registry data."""

from __future__ import annotations

import json
import re
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

import generate_chart

STATS_DIR = Path(__file__).parent
REPO_ROOT = STATS_DIR.parent.parent
README = REPO_ROOT / "README.md"
HISTORY = STATS_DIR / "history.json"
PYPI_DAILY = STATS_DIR / "pypi_daily.json"

CRATES_URL = "https://crates.io/api/v1/crates/goud-engine"
NUGET_SEARCH_URL = "https://azuresearch-usnc.nuget.org/query?q=packageid:GoudEngine&prerelease=false"
NUGET_REGISTRATION_URL = "https://api.nuget.org/v3/registration5-gz-semver2/goudengine/index.json"
NPM_URL = "https://api.npmjs.org/downloads/range/2020-01-01:2099-12-31/goudengine"
PYPI_URL = "https://pypistats.org/api/packages/goudengine/overall?mirrors=false"
MAVEN_SEARCH_URL = "https://central.sonatype.com/api/v1/search?q=g:io.github.aram-devdocs+AND+a:goud-engine-kotlin"
LUAROCKS_URL = "https://luarocks.org/api/1/modules/aram-devdocs/goudengine"
GO_PROXY_URL = "https://proxy.golang.org/github.com/aram-devdocs/goud-engine-go/goud/@v/list"

README_MARKER_PATTERN = re.compile(
    r"<!-- COMMUNITY-STATS:START -->.*?<!-- COMMUNITY-STATS:END -->",
    flags=re.DOTALL,
)

DOWNLOADS_BADGE_PATTERN = re.compile(
    r"\[!\[total downloads\]\(https://img\.shields\.io/badge/total_downloads-[^)]+\)\]\(#downloads\)"
)


def fetch_json(url: str, *, retries: int = 1) -> Any:
    last_error: Exception | None = None
    for attempt in range(retries):
        request = urllib.request.Request(
            url,
            headers={
                "Accept": "application/json",
                "User-Agent": "GoudEngine Community Stats",
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return json.load(response)
        except urllib.error.HTTPError as exc:
            last_error = exc
            if exc.code != 429 or attempt == retries - 1:
                raise
            retry_after = exc.headers.get("Retry-After")
            delay = float(retry_after) if retry_after else float(attempt + 1)
            time.sleep(delay)
        except Exception as exc:  # noqa: BLE001
            last_error = exc
            if attempt == retries - 1:
                raise
            time.sleep(float(attempt + 1))
    if last_error is not None:
        raise last_error
    raise RuntimeError(f"Failed to fetch JSON from {url}")


def safe_fetch_int(fetcher: Callable[[], int], previous: int) -> int:
    try:
        return max(0, int(fetcher()))
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError, urllib.error.URLError):
        return previous


def fetch_crates_total() -> int:
    payload = fetch_json(CRATES_URL)
    return int(payload["crate"]["downloads"])


def fetch_nuget_total() -> int:
    search_payload = fetch_json(NUGET_SEARCH_URL)
    search_data = search_payload.get("data", [])
    if search_data:
        total_downloads = int(search_data[0].get("totalDownloads", 0))
        if total_downloads > 0:
            return total_downloads

    registration_payload = fetch_json(NUGET_REGISTRATION_URL)
    return sum(
        int(version.get("catalogEntry", {}).get("downloads", 0))
        for page in registration_payload.get("items", [])
        for version in page.get("items", []) or []
    )


def fetch_npm_total() -> int:
    payload = fetch_json(NPM_URL)
    return sum(int(day.get("downloads", 0)) for day in payload.get("downloads", []))


def fetch_maven_total() -> int:
    payload = fetch_json(MAVEN_SEARCH_URL)
    items = payload.get("items", [])
    if items:
        return int(items[0].get("downloadCount", 0))
    return 0


def fetch_luarocks_total() -> int:
    payload = fetch_json(LUAROCKS_URL)
    return int(payload.get("downloads", 0))


def fetch_go_versions() -> int:
    """Return number of published Go module versions as a proxy for activity."""
    request = urllib.request.Request(
        GO_PROXY_URL,
        headers={"User-Agent": "GoudEngine Community Stats"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            lines = response.read().decode().strip().splitlines()
            return len(lines)
    except Exception:  # noqa: BLE001
        return 0


def fetch_pypi_daily_downloads() -> dict[str, int]:
    payload = fetch_json(PYPI_URL, retries=5)
    return {
        entry["date"]: int(entry.get("downloads", 0))
        for entry in payload.get("data", [])
        if entry.get("date")
    }


def load_json(path: Path, default: Any) -> Any:
    if not path.exists():
        return default
    return json.loads(path.read_text())


def write_json(path: Path, payload: Any) -> None:
    path.write_text(json.dumps(payload, indent=2) + "\n")


def merge_pypi_daily(existing: dict[str, int], latest: dict[str, int]) -> dict[str, int]:
    merged = {str(date): int(downloads) for date, downloads in existing.items()}
    for date, downloads in latest.items():
        merged[str(date)] = int(downloads)
    return dict(sorted(merged.items()))


def upsert_history(history: list[dict[str, int | str]], entry: dict[str, int | str], *, reset: bool) -> list[dict[str, int | str]]:
    if reset:
        return [entry]

    filtered = [item for item in history if item.get("date") != entry["date"]]
    filtered.append(entry)
    return sorted(filtered, key=lambda item: str(item["date"]))


def format_number(value: int) -> str:
    return f"{value:,}"


def build_stats_block(entry: dict[str, int | str]) -> str:
    return "\n".join(
        [
            "<!-- COMMUNITY-STATS:START -->",
            "| | Stars | Forks | Contributors |",
            "|--|-------|-------|--------------|",
            "| **GitHub** | ![stars](https://img.shields.io/github/stars/aram-devdocs/GoudEngine) | ![forks](https://img.shields.io/github/forks/aram-devdocs/GoudEngine) | ![contributors](https://img.shields.io/github/contributors/aram-devdocs/GoudEngine) |",
            "",
            "### Downloads",
            "",
            "| Registry | Total Downloads |",
            "|----------|-----------------|",
            f"| crates.io | [{format_number(int(entry['crates_io']))}](https://crates.io/crates/goud-engine) |",
            f"| NuGet | [{format_number(int(entry['nuget']))}](https://www.nuget.org/packages/GoudEngine/) |",
            f"| PyPI | [{format_number(int(entry['pypi']))}](https://pypi.org/project/goudengine/) |",
            f"| npm | [{format_number(int(entry['npm']))}](https://www.npmjs.com/package/goudengine) |",
            f"| Maven Central | [{format_number(int(entry.get('maven', 0)))}](https://central.sonatype.com/artifact/io.github.aram-devdocs/goud-engine-kotlin) |",
            f"| LuaRocks | [{format_number(int(entry.get('luarocks', 0)))}](https://luarocks.org/modules/aram-devdocs/goudengine) |",
            f"| Go | [{int(entry.get('go_versions', 0))} versions](https://pkg.go.dev/github.com/aram-devdocs/goud-engine-go/goud) |",
            "",
            "<sub>PyPI totals exclude mirrors.</sub>",
            "",
            "![Total Downloads Over Time](.github/stats/downloads.svg)",
            "",
            "[![Star History Chart](https://api.star-history.com/svg?repos=aram-devdocs/GoudEngine&type=Date)](https://star-history.com/#aram-devdocs/GoudEngine&Date)",
            "",
            f"<sub>Last updated: {entry['date']} via [GitHub Action](.github/workflows/community-stats.yml)</sub>",
            "<!-- COMMUNITY-STATS:END -->",
        ]
    )


def build_downloads_badge(total: int) -> str:
    label = format_number(total).replace(",", "%2C")
    return f"[![total downloads](https://img.shields.io/badge/total_downloads-{label}-brightgreen)](#downloads)"


def rewrite_readme(stats_block: str, *, total: int) -> None:
    readme = README.read_text()
    if not README_MARKER_PATTERN.search(readme):
        raise RuntimeError("README community stats block markers were not found")
    updated = README_MARKER_PATTERN.sub(stats_block, readme)
    badge = build_downloads_badge(total)
    updated = DOWNLOADS_BADGE_PATTERN.sub(badge, updated)
    README.write_text(updated)


def main() -> None:
    existing_pypi_daily = load_json(PYPI_DAILY, {})
    history = load_json(HISTORY, [])
    bootstrap_history = not history
    previous_entry = history[-1] if history else {}

    if bootstrap_history:
        crates_total = fetch_crates_total()
        nuget_total = fetch_nuget_total()
        npm_total = fetch_npm_total()
        maven_total = safe_fetch_int(fetch_maven_total, 0)
        luarocks_total = safe_fetch_int(fetch_luarocks_total, 0)
        go_versions = safe_fetch_int(fetch_go_versions, 0)
        latest_pypi_daily = fetch_pypi_daily_downloads()
    else:
        crates_total = safe_fetch_int(fetch_crates_total, int(previous_entry.get("crates_io", 0)))
        nuget_total = safe_fetch_int(fetch_nuget_total, int(previous_entry.get("nuget", 0)))
        npm_total = safe_fetch_int(fetch_npm_total, int(previous_entry.get("npm", 0)))
        maven_total = safe_fetch_int(fetch_maven_total, int(previous_entry.get("maven", 0)))
        luarocks_total = safe_fetch_int(fetch_luarocks_total, int(previous_entry.get("luarocks", 0)))
        go_versions = safe_fetch_int(fetch_go_versions, int(previous_entry.get("go_versions", 0)))
        try:
            latest_pypi_daily = fetch_pypi_daily_downloads()
        except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError, urllib.error.URLError):
            latest_pypi_daily = existing_pypi_daily

    merged_pypi_daily = merge_pypi_daily(existing_pypi_daily, latest_pypi_daily)
    pypi_total = sum(merged_pypi_daily.values())
    if not bootstrap_history:
        pypi_total = max(int(previous_entry.get("pypi", 0)), pypi_total)

    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    entry = {
        "date": today,
        "crates_io": crates_total,
        "nuget": nuget_total,
        "pypi": pypi_total,
        "npm": npm_total,
        "maven": maven_total,
        "luarocks": luarocks_total,
        "go_versions": go_versions,
        "total": crates_total + nuget_total + pypi_total + npm_total + maven_total + luarocks_total,
    }

    write_json(PYPI_DAILY, merged_pypi_daily)
    write_json(HISTORY, upsert_history(history, entry, reset=bootstrap_history))
    rewrite_readme(build_stats_block(entry), total=entry["total"])
    generate_chart.main()

    print(
        "Updated community stats:",
        json.dumps(entry, sort_keys=True),
    )


if __name__ == "__main__":
    main()
