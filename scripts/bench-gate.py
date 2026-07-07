#!/usr/bin/env python3
"""Ratio-normalized benchmark gate for GoudEngine renderer/engine benches.

Why ratios instead of absolute times
------------------------------------
Criterion reports absolute wall-clock times (nanoseconds). Those numbers drift
wildly between machines and even between runs on the same machine (thermal
throttling, noisy neighbors in CI, turbo boost, etc.). Comparing an absolute
renderer time against a checked-in absolute baseline would flag the whole world
as "regressed" any time the runner is simply slower.

To survive that variance we normalize every benchmark against a cheap, stable
*reference* benchmark that runs on the same machine in the same session -- by
default ``engine_tick/tick_10k`` (a headless transform-propagation tick). For
each benchmark we compute::

    ratio = mean_ns(benchmark) / mean_ns(reference)

If the machine is 20% slower today, both the benchmark and the reference slow
down together and the ratio is (approximately) unchanged. A ratio that grows
means the benchmark got slower *relative to the engine's baseline work*, which
is a genuine regression rather than runner noise.

The gate compares each benchmark's current ratio against a checked-in baseline
ratio and fails when a ratio regresses beyond a threshold.

Baseline format (``goud_engine/benches/baselines/criterion_baseline.json``)::

    {
      "reference_bench": "engine_tick/tick_10k",
      "entries": {
        "engine_tick/tick_10k": {"mean_ns": 812345.0, "ratio": 1.0},
        "frame_scan/dynamic_10k": {"mean_ns": 2543210.0, "ratio": 3.13},
        ...
      }
    }

Usage
-----
Run the benches first (this script never runs cargo)::

    cargo bench --bench engine_tick_benchmarks
    cargo bench --bench renderer3d_frame_benchmarks

Then gate or (re)save the baseline::

    python3 scripts/bench-gate.py                 # gate, 10% ratio tolerance
    python3 scripts/bench-gate.py --quick         # gate, 15% ratio tolerance
    python3 scripts/bench-gate.py --save-baseline # rewrite the baseline

Exit codes: 0 = pass, 1 = regression / coverage mismatch / missing data.

Pure standard library, no third-party dependencies.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Repository-relative defaults.
REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CRITERION_DIR = REPO_ROOT / "target" / "criterion"
DEFAULT_BASELINE = (
    REPO_ROOT / "goud_engine" / "benches" / "baselines" / "criterion_baseline.json"
)
DEFAULT_REFERENCE = "engine_tick/tick_10k"

# Benchmark groups this gate tracks. A "bench name" is "<group>/<function>".
# Groups are also read from an existing baseline so the two stay in sync.
DEFAULT_GROUPS = [
    "engine_tick",
    "frame_scan",
    "material_sort",
    "cull_scaling",
    "primitive_draw_calls",
    "shadow_record",
]

DEFAULT_THRESHOLD = 0.10  # 10%
QUICK_THRESHOLD = 0.15  # 15%


def read_mean_ns(criterion_dir: Path, bench_name: str) -> float | None:
    """Return the mean point estimate (ns) for a bench, or None if absent."""
    group, _, function = bench_name.partition("/")
    estimates = criterion_dir / group / function / "new" / "estimates.json"
    if not estimates.is_file():
        return None
    try:
        with estimates.open(encoding="utf-8") as fh:
            data = json.load(fh)
        return float(data["mean"]["point_estimate"])
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(f"error: failed to parse {estimates}: {exc}", file=sys.stderr)
        return None


def discover_run_benches(criterion_dir: Path, groups: list[str]) -> set[str]:
    """Find "<group>/<function>" benches present in the criterion output."""
    found: set[str] = set()
    for group in groups:
        group_dir = criterion_dir / group
        if not group_dir.is_dir():
            continue
        for child in sorted(group_dir.iterdir()):
            if not child.is_dir():
                continue
            if (child / "new" / "estimates.json").is_file():
                found.add(f"{group}/{child.name}")
    return found


def collect_run(
    criterion_dir: Path, bench_names: set[str], reference: str
) -> dict[str, dict[str, float]] | None:
    """Build {name: {mean_ns, ratio}} for the current run. None on hard error."""
    ref_mean = read_mean_ns(criterion_dir, reference)
    if ref_mean is None:
        print(
            f"error: reference bench '{reference}' not found under {criterion_dir}.\n"
            f"       Run its bench (e.g. `cargo bench --bench engine_tick_benchmarks`) first.",
            file=sys.stderr,
        )
        return None
    if ref_mean <= 0.0:
        print(f"error: reference bench '{reference}' has non-positive mean", file=sys.stderr)
        return None

    entries: dict[str, dict[str, float]] = {}
    for name in sorted(bench_names):
        mean_ns = read_mean_ns(criterion_dir, name)
        if mean_ns is None:
            continue  # coverage checked by the caller
        entries[name] = {"mean_ns": mean_ns, "ratio": mean_ns / ref_mean}
    return entries


def load_baseline(path: Path) -> dict:
    with path.open(encoding="utf-8") as fh:
        return json.load(fh)


def tracked_groups(baseline_path: Path, extra_groups: list[str]) -> list[str]:
    """Groups to scan: DEFAULT_GROUPS + any groups from an existing baseline."""
    groups = set(DEFAULT_GROUPS)
    groups.update(extra_groups)
    if baseline_path.is_file():
        try:
            base = load_baseline(baseline_path)
            for name in base.get("entries", {}):
                groups.add(name.partition("/")[0])
        except (OSError, ValueError):
            pass
    return sorted(groups)


def save_baseline(args: argparse.Namespace) -> int:
    groups = tracked_groups(args.baseline, args.group)
    run_benches = discover_run_benches(args.criterion_dir, groups)
    if not run_benches:
        print(
            f"error: no benchmarks found under {args.criterion_dir} for groups {groups}.\n"
            f"       Run the benches before --save-baseline.",
            file=sys.stderr,
        )
        return 1
    if args.reference not in run_benches:
        print(
            f"error: reference bench '{args.reference}' not present in this run.",
            file=sys.stderr,
        )
        return 1

    entries = collect_run(args.criterion_dir, run_benches, args.reference)
    if entries is None:
        return 1

    baseline = {"reference_bench": args.reference, "entries": entries}
    args.baseline.parent.mkdir(parents=True, exist_ok=True)
    with args.baseline.open("w", encoding="utf-8") as fh:
        json.dump(baseline, fh, indent=2, sort_keys=True)
        fh.write("\n")

    print(f"Wrote baseline with {len(entries)} benches to {args.baseline}")
    print(f"Reference: {args.reference}")
    for name in sorted(entries):
        e = entries[name]
        print(f"  {name:<32} mean={e['mean_ns']:>14.1f} ns  ratio={e['ratio']:.4f}")
    return 0


def gate(args: argparse.Namespace) -> int:
    if not args.baseline.is_file():
        print(
            f"error: baseline {args.baseline} not found. Create it with --save-baseline.",
            file=sys.stderr,
        )
        return 1

    baseline = load_baseline(args.baseline)
    reference = baseline.get("reference_bench", args.reference)
    baseline_entries: dict = baseline.get("entries", {})
    if not baseline_entries:
        print("error: baseline has no entries", file=sys.stderr)
        return 1

    threshold = QUICK_THRESHOLD if args.quick else DEFAULT_THRESHOLD
    groups = tracked_groups(args.baseline, args.group)
    run_benches = discover_run_benches(args.criterion_dir, groups)

    baseline_names = set(baseline_entries)
    failures: list[str] = []

    # Coverage: any mismatch between baseline and run is a hard failure.
    missing_in_run = sorted(baseline_names - run_benches)
    extra_in_run = sorted(run_benches - baseline_names)
    for name in missing_in_run:
        failures.append(f"bench '{name}' is in the baseline but missing from this run")
    for name in extra_in_run:
        failures.append(f"bench '{name}' ran but is missing from the baseline")

    run_entries = collect_run(args.criterion_dir, run_benches & baseline_names, reference)
    if run_entries is None:
        return 1

    print(f"Bench gate (ratio-normalized against '{reference}')")
    print(f"Tolerance: {threshold * 100:.0f}%  |  baseline: {args.baseline}")
    print(f"{'bench':<32} {'base ratio':>11} {'now ratio':>11} {'delta':>9}  status")
    print("-" * 78)

    for name in sorted(baseline_names & run_benches):
        base_ratio = float(baseline_entries[name]["ratio"])
        now_ratio = run_entries[name]["ratio"]
        if base_ratio <= 0.0:
            status = "SKIP(base<=0)"
            delta = 0.0
        else:
            delta = (now_ratio - base_ratio) / base_ratio
            if delta > threshold:
                status = "FAIL"
                failures.append(
                    f"bench '{name}' regressed {delta * 100:.1f}% "
                    f"(ratio {base_ratio:.4f} -> {now_ratio:.4f}, tolerance {threshold * 100:.0f}%)"
                )
            else:
                status = "ok"
        print(
            f"{name:<32} {base_ratio:>11.4f} {now_ratio:>11.4f} {delta * 100:>8.1f}%  {status}"
        )

    print("-" * 78)
    if failures:
        print(f"\nFAILED: {len(failures)} problem(s):")
        for msg in failures:
            print(f"  - {msg}")
        return 1

    print("\nPASS: all tracked benches within tolerance.")
    return 0


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Ratio-normalized benchmark gate for GoudEngine.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--save-baseline",
        action="store_true",
        help="(re)write the baseline from the current criterion run",
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="use the looser 15%% ratio tolerance instead of 10%%",
    )
    parser.add_argument(
        "--reference",
        default=DEFAULT_REFERENCE,
        help=f"reference bench for ratio normalization (default: {DEFAULT_REFERENCE})",
    )
    parser.add_argument(
        "--baseline",
        type=Path,
        default=DEFAULT_BASELINE,
        help="path to the checked-in baseline JSON",
    )
    parser.add_argument(
        "--criterion-dir",
        type=Path,
        default=DEFAULT_CRITERION_DIR,
        help="path to target/criterion",
    )
    parser.add_argument(
        "--group",
        action="append",
        default=[],
        help="additional bench group to track (repeatable)",
    )
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    if args.save_baseline:
        return save_baseline(args)
    return gate(args)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
