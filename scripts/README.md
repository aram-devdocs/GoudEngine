# Scripts

This directory holds helper and CI scripts (agent tooling, gate checks,
codegen helpers). The primary developer entry points live at the repository
root and are documented below.

## Top-level scripts

Run all of these from the repository root.

| Script | What it does |
|--------|--------------|
| `install.sh` | Installs system and toolchain dependencies (ALSA/X11/OpenGL dev libs on Linux, Xcode Command Line Tools on macOS, Rust, and cbindgen). Run once when setting up a fresh checkout. |
| `build.sh` | Builds the Rust engine and SDK artifacts. Defaults to a debug build; `--release`/`--prod` build optimized. Flags select scope: `--core-only`, `--host-runtime-only`, `--skip-csharp-sdk-build`, and `--local` (ProjectReference path instead of NuGet). |
| `codegen.sh` | Regenerates every SDK from `codegen/goud_sdk.schema.json` and the extracted FFI manifest. Run after changing the schema, FFI exports, or `codegen/ffi_mapping.json`. |
| `dev.sh` | Builds and runs an example. `--sdk <lang>` selects the SDK (csharp, python, rust, typescript, c, cpp, go, kotlin, lua, swift) and `--game <name>` the example. See `./dev.sh --help` for the full game list. |
| `clean.sh` | Cleans build output. Default is lightweight (incremental dirs, `bin`/`obj`, `__pycache__`, stray `.nupkg`); `--deep` also runs `cargo clean`, removes `node_modules`, prunes the local NuGet feed, and clears worktrees; `--size` reports sizes without deleting. |
| `package.sh` | Builds and pushes the C# NuGet package to the local feed (`--local`). The `--prod` path is a no-op; production publishing to nuget.org is driven by the release workflow. |
| `graph.sh` | Renders the crate module dependency graph to `docs/diagrams/`. Requires `cargo modules` and Graphviz (`neato`, `dot`). |

## Versioning

`increment_version.sh` bumps version strings across the workspace, but it is
superseded by release-please. Prefer conventional commits (`feat:`, `fix:`,
etc.) merged to `main`; release-please opens a Release PR with the correct
version bumps. The script remains only for local convenience.
