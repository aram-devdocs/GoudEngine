# Feature Lab — Rust

Headless Rust SDK smoke example that exercises scene management, ECS composition, animation helpers, input mapping, provider capability queries, and safe headless fallbacks.

Unlike `flappy_bird`, this example is not a playable parity demo. It exists to expand Rust SDK coverage with a compileable, mergeable sample that runs without a window.

## Run

```bash
cargo run -p feature-lab
```

Run it from the repository root. The process prints PASS/FAIL lines for each exercised surface and exits non-zero if any check fails.

## Debugger Note

`feature-lab` now auto-enables debugger mode for one headless route labeled
`feature-lab-rust-headless`, then prints the manual attach flow:

1. start `cargo run -p goudengine-mcp`
2. call `goudengine.list_contexts`
3. call `goudengine.attach_context`

Use the Rust getting-started guide and debugger runtime guide for the full
desktop/headless config variants and the rest of the inspection workflow.
