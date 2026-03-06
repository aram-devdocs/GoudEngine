"""Generate a downloads-over-time SVG chart from history.json."""

import json
import sys
from pathlib import Path

STATS_DIR = Path(__file__).parent
HISTORY = STATS_DIR / "history.json"
OUTPUT = STATS_DIR / "downloads.svg"

# Chart dimensions
W, H = 720, 320
PAD_L, PAD_R, PAD_T, PAD_B = 70, 30, 30, 50


def fmt(n: int) -> str:
    """Format number with K/M suffix."""
    if n >= 1_000_000:
        return f"{n / 1_000_000:.1f}M"
    if n >= 1_000:
        return f"{n / 1_000:.1f}K"
    return str(n)


def main() -> None:
    data = json.loads(HISTORY.read_text())
    if len(data) < 2:
        # Need at least 2 data points for a meaningful chart
        # Write a placeholder SVG
        svg = (
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}">'
            f'<rect width="{W}" height="{H}" fill="#0d1117" rx="8"/>'
            f'<text x="{W // 2}" y="{H // 2}" text-anchor="middle"'
            f' fill="#8b949e" font-family="sans-serif" font-size="14">'
            f"Collecting data — chart available after 2 days</text></svg>"
        )
        OUTPUT.write_text(svg)
        return

    dates = [d["date"] for d in data]
    totals = [
        d.get("crates_io", 0) + d.get("nuget", 0) + d.get("pypi_monthly", 0) + d.get("npm_monthly", 0)
        for d in data
    ]

    plot_w = W - PAD_L - PAD_R
    plot_h = H - PAD_T - PAD_B

    min_y = 0
    max_y = max(totals) * 1.15 or 1
    n = len(dates)

    def x(i: int) -> float:
        return PAD_L + (i / max(n - 1, 1)) * plot_w

    def y(v: float) -> float:
        return PAD_T + plot_h - ((v - min_y) / (max_y - min_y)) * plot_h

    # Build polyline points
    points = " ".join(f"{x(i):.1f},{y(v):.1f}" for i, v in enumerate(totals))

    # Area fill (close path to bottom)
    area = points + f" {x(n - 1):.1f},{y(0):.1f} {x(0):.1f},{y(0):.1f}"

    # Y-axis gridlines (5 lines)
    gridlines = ""
    for i in range(5):
        val = min_y + (max_y - min_y) * i / 4
        yp = y(val)
        gridlines += (
            f'<line x1="{PAD_L}" y1="{yp:.1f}" x2="{W - PAD_R}" y2="{yp:.1f}"'
            f' stroke="#21262d" stroke-width="1"/>'
            f'<text x="{PAD_L - 8}" y="{yp + 4:.1f}" text-anchor="end"'
            f' fill="#8b949e" font-family="sans-serif" font-size="11">{fmt(int(val))}</text>'
        )

    # X-axis labels (show ~5 evenly spaced dates)
    xlabels = ""
    step = max(1, (n - 1) // 4)
    for i in range(0, n, step):
        label = dates[i][5:]  # MM-DD
        xlabels += (
            f'<text x="{x(i):.1f}" y="{H - 10}" text-anchor="middle"'
            f' fill="#8b949e" font-family="sans-serif" font-size="11">{label}</text>'
        )

    # Latest value label
    latest_label = (
        f'<text x="{x(n - 1) + 4:.1f}" y="{y(totals[-1]) - 8:.1f}"'
        f' fill="#58a6ff" font-family="sans-serif" font-size="12"'
        f' font-weight="bold">{fmt(totals[-1])}</text>'
    )

    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}">
  <rect width="{W}" height="{H}" fill="#0d1117" rx="8"/>
  <text x="{PAD_L}" y="22" fill="#c9d1d9" font-family="sans-serif" font-size="14" font-weight="bold">Total Downloads Over Time</text>
  {gridlines}
  <polygon points="{area}" fill="#58a6ff" opacity="0.15"/>
  <polyline points="{points}" fill="none" stroke="#58a6ff" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/>
  {xlabels}
  {latest_label}
</svg>"""

    OUTPUT.write_text(svg)
    print(f"Chart written to {OUTPUT} ({n} data points)")


if __name__ == "__main__":
    main()
