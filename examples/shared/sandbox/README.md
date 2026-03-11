# Sandbox Shared Assets

This directory is the shared asset root for the Alpha-001 Sandbox examples.

All sandbox implementations should load assets and copy from here so the public parity app stays aligned across Rust, C#, Python, TypeScript desktop, and TypeScript web.

Shared contract files:

- `manifest.json` -- scene labels, panel copy, capability-gating text, and the native sandbox packet version/port
- `fonts/test_font.ttf` -- current default HUD font used by the sandbox manifest
- `fonts/AtkinsonHyperlegible-Regular.ttf` -- vendored alternate font kept for future text-renderer compatibility work

Current shared assets:

- `sprites/background-day.png`
- `sprites/yellowbird-midflap.png`
- `sprites/pipe-green.png`
- `textures/default_grey.png`
- `audio/sandbox-tone.wav`
