# Sandbox Guide

Sandbox is the shared parity target for Alpha-001 on this branch.

- Flappy Bird stays the onboarding baseline.
- Sandbox is the cross-language feature tour under active recovery.
- Feature Lab stays in the repo as supplemental smoke coverage.

## Run it

```bash
./dev.sh --game sandbox
./dev.sh --sdk python --game sandbox
./dev.sh --sdk typescript --game sandbox
./dev.sh --sdk typescript --game sandbox_web
cargo run -p sandbox
```

## Branch status

This branch keeps the recovery ledger in `.claude/specs/alpha-001-sandbox-execution.csv`.
Use that file for live branch truth while the remaining runtime rows are being closed.

Current state:

- C#, Python, and TypeScript load the shared asset pack and the three-panel HUD.
- Rust uses the same asset pack and now calls `GoudGame::draw_text(...)` for HUD copy.
- The remaining branch recovery work is tracked in the execution CSV row set, not in per-runtime handwritten notes here.

## What to expect

Every runtime uses the shared asset pack and the same three-panel layout:

- Overview panel in the top-left
- Live status panel in the top-right
- Try-this-next panel along the bottom

Rust status in this branch:

- Rust uses the shared manifest, panel layout, and `GoudGame::draw_text(...)` path.
- Rust desktop scene visibility and HUD text are now back on the same public SDK path used by the example.

The desktop targets also share the same native packet contract for peer sync:

```text
sandbox|v1|<role>|<mode>|<x>|<y>|<label>
```

Recovered runs should let you:

- Press `1`, `2`, or `3` to switch between `2D`, `3D`, and `Hybrid`
- Move the local bird with `WASD` or the arrow keys
- See the mouse marker update live
- Press `SPACE` to activate audio and play the shared tone
- Open a second native sandbox instance and watch a peer bird appear

Use the execution CSV for any runtime-specific exceptions that are still open on this branch. As of the current recovery pass, the shared HUD/text path is back in Rust, while some native 3D scene-flow verification rows remain open.

## Panel meanings

- Overview: shared intent, controls, and what the example is proving
- Live status: current scene, runtime capabilities, input state, and networking detail
- Try this next: next checks to perform, audio status, and peer-sync hints

## Networking check

For native runtimes, open two local instances on the same machine. One should report `host` and `waiting` until the second joins. After the second joins, both should show peer discovery and render a remote bird label.

Useful env vars for smoke and CI:

```bash
GOUD_SANDBOX_NETWORK_PORT=38491
GOUD_SANDBOX_NETWORK_ROLE=auto   # or host / client
GOUD_SANDBOX_EXIT_ON_PEER=1
GOUD_SANDBOX_EXPECT_PEER=1
```

## Web limitations

The web target keeps the same HUD structure and shared copy, but it does not fake unsupported desktop behavior.

- Networking stays visibly capability-gated in browser mode.
- Renderer fallback is called out explicitly when the runtime does not expose a usable WebGPU adapter.

Use [Web Platform Gotchas](web-platform-gotchas.md) for browser-specific troubleshooting and [Example Showcase](showcase.md) for the generated run matrix.
