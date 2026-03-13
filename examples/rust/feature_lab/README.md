# Feature Lab — Rust

Headless Rust SDK smoke example that exercises scene management, ECS composition, animation helpers, input mapping, provider capability queries, and safe headless fallbacks.

Unlike `flappy_bird`, this example is not a playable parity demo. It exists to expand Rust SDK coverage with a compileable, mergeable sample that runs without a window.

## Run

```bash
cargo run -p feature-lab
```

Run it from the repository root. The process prints PASS/FAIL lines for each exercised surface and exits non-zero if any check fails.

## Debugger Note

`feature-lab` is the reference headless Rust example for exercising SDK surfaces
without a window. It does not enable debugger mode automatically, but the intended
Rust attach path is the same one used by the rest of the public rollout:

- build a `DebuggerConfig` with `enabled: true` and `publish_local_attach: true`
- pass it through `ContextConfig` for headless flows
- start `cargo run -p goudengine-mcp`
- attach with `goudengine.list_contexts` and `goudengine.attach_context`

Use the Rust getting-started guide and debugger runtime guide for the full config
and inspection workflow.
