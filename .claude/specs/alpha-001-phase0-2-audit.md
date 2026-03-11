# ALPHA-001 Phase 0-2 Audit

## Status

- Branch: `codex/alpha-001-phase0-2-remediation`
- Scope: Phase 0 through Phase 2 release-readiness remediation for `#114`
- GitHub issue policy: no new issues; post wave summaries to `#114`
- Allowed explicit deferrals: `#361`, `#366`, `#378`, `#380`, `#382`
- CSV batch automation is currently locked; ledger updates below are manual and re-queued for retry.
- Latest `spawn_agents_on_csv` retry still failed with `error returned from database: (code: 5) database is locked`.
- Clean-room SDK regeneration now passes locally via `bash scripts/clean-room-regenerate.sh`.
- Clean-room SDK + docs regeneration now passes locally via `PATH="$HOME/.cargo/bin:$HOME/.dotnet/tools:$PATH" bash scripts/clean-room-regenerate.sh --docs`.
- Main repo verification currently passes locally:
  - `cargo check`
  - `cargo fmt --all -- --check`
  - `cargo clippy -- -D warnings`
  - `cargo deny check`
  - `cargo doc --no-deps -p goud-engine-core -p goud-engine`
  - `python3 sdks/python/test_bindings.py`
  - `python3 sdks/python/test_network_loopback.py`
  - `cd sdks/typescript && npm test`
  - `DOTNET_ROOT_X64=/usr/local/share/dotnet/x64 /usr/local/share/dotnet/x64/dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal`
  - `python3 -m coverage run --source=sdks/python/goud_engine sdks/python/test_bindings.py && python3 -m coverage run --append --source=sdks/python/goud_engine sdks/python/test_network_loopback.py && python3 -m coverage report`
  - `cd sdks/typescript && npx c8 --reporter=text-summary npm test`
  - `DOTNET_ROOT_X64=/usr/local/share/dotnet/x64 /usr/local/share/dotnet/x64/dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -c Release -v minimal /p:CollectCoverage=true /p:CoverletOutput=sdks/csharp.tests/TestResults/coverage/ /p:CoverletOutputFormat=cobertura`
  - `cargo test -p goud-engine-core --doc -- --nocapture`
  - `cargo test --workspace --doc -- --nocapture`
- `cargo test --workspace --quiet` passes locally.

## Audit Summary

### Phase 0

- Functional child scope is merged: all child issues under `#115` are closed.
- Core artifacts exist:
  - `/Users/aramhammoudeh/dev/game/GoudEngine/CONTRIBUTING.md`
  - `/Users/aramhammoudeh/dev/game/GoudEngine/ARCHITECTURE.md`
  - `/Users/aramhammoudeh/dev/game/GoudEngine/CODE_OF_CONDUCT.md`
  - `/Users/aramhammoudeh/dev/game/GoudEngine/book.toml`
  - `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/getting-started/csharp.md`
  - `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/getting-started/python.md`
  - `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/getting-started/rust.md`
  - `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/getting-started/typescript.md`
- Tracker normalization is still incomplete overall, but `#115` is now closed on GitHub and no longer blocks the Phase 0 parent state from a tracker perspective.

### Phase 1

- Proven complete:
  - `cargo run -p lint-layers` passes.
  - `cargo check` passes.
  - `cargo test --workspace --quiet` passes.
  - Rust source file line limit check passes.
- Current repo no longer reproduces the earlier `#[allow(dead_code)]` production-code finding.
- Remaining `#[allow(dead_code)]` usages are in test fixtures/helpers only.
- Tracker inconsistency remains because the issue comments and ledger have not been reconciled to match the current repo state.

### Phase 2

- Engine-side feature work is largely merged:
  - All listed child issues under `#117`, `#118`, `#119`, `#120`, `#121`, `#122`, `#123`, `#125`, `#126`, and `#127` are closed.
  - Networking parent `#140` and children `#364` / `#386` are now closed on GitHub with comments linking the native ws/wss tests, browser runtime smoke, and generated SDK wrapper proof.
- `#128` is now closed after the coverage/reporting and codegen drift acceptance criteria were met.
- Phase 2 networking regressions reproduced at the start of this branch are fixed.
- Cross-language example parity now includes both Flappy Bird baseline coverage and Feature Lab smoke coverage across Rust, C#, Python, TS desktop, and TS web.
- The generated-file size-bar blocker is explicitly waived for this release by user direction and is not part of the stop-the-line criteria.

## Reproduced Failures

- Python SDK:
  - Command: `python3 sdks/python/test_bindings.py`
  - Original failure: generated Python bindings tried to bind `goud_engine_config_set_physics_debug`, but the loaded native library did not export the symbol.
- C# SDK:
  - Command: `dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal`
  - Original failure: compile errors due to missing generated C# types such as `NetworkPacket`, `NetworkStats`, `NetworkConnectResult`, `NetworkSimulationConfig`, and `PhysicsBackend2D`.
- TypeScript SDK:
  - Command: `cd sdks/typescript && npm test`
  - Original failure: loopback networking test failed with `Failed to convert napi value Undefined into rust type i32` in `NetworkManager.host`.
- Codegen parity:
  - Command: `python3 codegen/validate_coverage.py`
  - Current result: now hard-fails on skew and currently reports zero mismatches.

## Implemented Remediation Snapshot

- Python SDK:
  - `python3 sdks/python/test_bindings.py` now passes.
  - The generated loader now prefers a native library that actually exports `goud_engine_config_set_physics_debug`.
  - `sdks/python/goud_engine/networking.py` is now generator-owned.
- C# SDK:
  - `dotnet build sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal` now passes.
  - The test project now imports `/Users/aramhammoudeh/dev/game/GoudEngine/sdks/csharp/build/GoudEngine.targets`, and the native runtime copy logic now places `libgoud_engine.dylib` in the test output directory.
  - `sdks/csharp/NetworkManager.cs` and `sdks/csharp/NetworkEndpoint.cs` are now generator-owned.
  - The macOS runtime payloads are corrected: `sdks/csharp/runtimes/osx-x64/native/libgoud_engine.dylib` is x64 and `sdks/csharp/runtimes/osx-arm64/native/libgoud_engine.dylib` now exists and is arm64.
  - `DOTNET_ROOT_X64=/usr/local/share/dotnet/x64 /usr/local/share/dotnet/x64/dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal` now passes locally.
- TypeScript SDK:
  - `cd sdks/typescript && npm test` now passes.
  - Regenerated TS artifacts restored the missing `NetworkProtocol.Tcp` path used by the loopback wrapper tests.
  - `sdks/typescript/src/shared/network.ts`, `sdks/typescript/src/index.ts`, `sdks/typescript/src/node/index.ts`, and `sdks/typescript/src/web/index.ts` are now generator-owned.
- Rust docs/tests:
  - `goud_engine/src/assets/loaders/config/asset.rs` rustdoc examples now go through `ConfigLoader` + `LoadContext`, which removes the workspace doctest failure caused by doctest-local `serde_json` type skew.
- CI/docs/release hardening:
  - Added `/Users/aramhammoudeh/dev/game/GoudEngine/scripts/check-generated-artifacts.sh`.
  - Added `/Users/aramhammoudeh/dev/game/GoudEngine/scripts/clean-room-regenerate.sh`.
  - Added `/Users/aramhammoudeh/dev/game/GoudEngine/codegen/gen_sdk_scaffolding.py`.
  - `codegen.sh` now bootstraps TypeScript native sources, regenerates package/build scaffolding for C#, Python, and TypeScript, and formats generated Rust outputs, so delete-and-regenerate covers more than just the wrapper subtree.
  - `/Users/aramhammoudeh/dev/game/GoudEngine/.github/workflows/ci.yml` now includes release-please branch pushes, full Python SDK binding coverage, and the generated-artifact check.
  - `/Users/aramhammoudeh/dev/game/GoudEngine/.github/workflows/ci.yml` now also includes a clean-room regeneration lane and a Feature Lab parity lane.
  - `/Users/aramhammoudeh/dev/game/GoudEngine/.github/workflows/docs.yml` now uses strict TypeScript and C# doc steps, passes `GOUD_ENGINE_LIB` plus `PDOC_ALLOW_EXEC` for Python docs, and uses `sdks/typescript/tsconfig.typedoc.json` for warning-free TypeDoc generation.
  - `/Users/aramhammoudeh/dev/game/GoudEngine/.github/workflows/release.yml` no longer manufactures a synthetic `CI Success` check.
  - `scripts/check-generated-artifacts.sh` now covers the generated networking wrappers, TypeScript native/lib outputs, and the generated package/build scaffolding files that were previously handwritten.
  - Generated docs download/media binaries are now treated as pure generated outputs: required for docs builds, but ignored in Git because their archive/video containers are nondeterministic across clean-room runs.
  - `scripts/check-generated-artifacts.sh` now has a split contract: default mode validates fresh-checkout SDK/codegen outputs, and `--docs` adds showcase/download/media/docs proof for the hosted docs path.
  - The docs workflow now generates the downloadable Flappy bundles before publishing and validates the full docs artifact set with `bash scripts/check-generated-artifacts.sh --docs`.
  - Generated showcase screenshots are required for docs builds but are no longer enforced as byte-stable drift artifacts across environments, because renderer/font/runtime differences make the PNG outputs nondeterministic in CI.
- Feature Lab parity:
  - Added `/Users/aramhammoudeh/dev/game/GoudEngine/examples/rust/feature_lab`.
  - Added `/Users/aramhammoudeh/dev/game/GoudEngine/examples/python/feature_lab.py`.
  - Added `/Users/aramhammoudeh/dev/game/GoudEngine/examples/csharp/feature_lab`.
  - Added `/Users/aramhammoudeh/dev/game/GoudEngine/examples/typescript/feature_lab` with desktop and web entrypoints.
  - Updated `/Users/aramhammoudeh/dev/game/GoudEngine/examples/README.md`, `/Users/aramhammoudeh/dev/game/GoudEngine/examples/rust/README.md`, `/Users/aramhammoudeh/dev/game/GoudEngine/examples/python/README.md`, `/Users/aramhammoudeh/dev/game/GoudEngine/examples/typescript/README.md`, and example/agent docs to expose the new parity commands.
- Example docs cleanup:
  - `/Users/aramhammoudeh/dev/game/GoudEngine/examples/python/README.md` now reflects current Python API names and current rendering/input/audio capabilities instead of the stale pre-rendering wording.
  - Added docs pages for `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/guides/build-your-first-game.md`, `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/guides/deployment.md`, `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/guides/faq.md`, `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/guides/videos.md`, and `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/guides/showcase.md`, and wired them into `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/SUMMARY.md`.
  - Added `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/guides/web-platform-gotchas.md` and linked it from the TypeScript getting-started page plus the TypeScript SDK README.
  - Generated snippet includes now exist under `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/generated/snippets/`, and the generated reference page is `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/reference/snippets.generated.md`.
  - Generated tutorial download bundles now exist under `/Users/aramhammoudeh/dev/game/GoudEngine/docs/src/generated/downloads/` and are linked from the Build Your First Game guide.

## Remaining Blockers

- No technical blockers remain for the Phase 0-2 remediation scope on this branch.
- The CSV batch agent flow is still blocked by the tool-side database lock, so the ledger remains manually maintained.
- `#114` remains open because it is the master alpha tracker for later-phase work outside this remediation branch; it is no longer blocked by Phase 0-2 SDK/docs/examples gaps.
- Remaining release mechanics before merge:
  - run final review gates
  - commit the regenerated/generated outputs
  - rerun clean-room checks from the committed tree
  - open the PR and confirm GitHub Actions are green

## CI and Release Gaps

- Fixed on this branch:
  - `ci.yml` no longer relies on `git diff -- sdks/`; it uses `scripts/check-generated-artifacts.sh`.
  - `ci.yml` now runs the full Python binding suite plus networking loopback coverage.
  - `ci.yml` now runs a clean-room regeneration proof, a Feature Lab parity proof, and the dedicated TypeScript web network smoke.
  - `docs.yml` no longer contains `|| true` or `--skipErrorChecking`.
  - `release.yml` no longer synthesizes a fake `CI Success` check.
- Tracker normalization is complete for this remediation scope:
  - Closed during this pass: `#128`, `#291`, `#293`, `#317`
  - Previously reconciled and closed: `#138`, `#140`, `#294`, `#296`, `#297`, `#312`, `#314`, `#316`, `#318`, `#319`, `#364`, `#386`
  - `#114` remains open as the broader alpha tracker

## Source-Of-Truth Findings

- Fixed on this branch:
  - The public networking surfaces in C#, Python, and TypeScript are now generator-owned.
  - C#, Python, and TypeScript package/build scaffolding now regenerates from codegen rather than remaining handwritten.
  - `python3 codegen/validate.py` and `python3 codegen/validate_coverage.py` both pass.
  - Clean-room delete-and-regenerate now works for SDK outputs and docs outputs from the checked-out source tree.
  - Generated reusable docs snippets now come from checked snippet sources plus validated/generated SDK/example artifacts.
- The enforced source-of-truth for the supported alpha SDK targets is now sufficient for release scope:
  - public SDK surfaces regenerate from codegen
  - package/build scaffolding regenerates from codegen
  - CI fails on SDK/schema/generated drift
  - clean-room regeneration works for SDKs and docs

## Examples And Documentation Findings

- Current cross-language parity now includes both baseline and broader smoke coverage:
  - Flappy Bird baseline: C#, Python, Rust, TS desktop, TS web
  - Feature Lab parity smoke: C#, Python, Rust, TS desktop, TS web
- Coverage and reporting now meet the parent acceptance criteria:
  - Python line coverage: `81%`
  - TypeScript line coverage: `80.16%`
  - C# line coverage: `83.36%`
- `#138` repo deliverables now exist and the parent is closed on GitHub. Important scope note:
  - `#294` was closed against the currently supported Alpha v1 SDK set: Rust, C#, Python, and TypeScript.
- `#296` repo deliverable now exists via the Web Platform Gotchas guide, and the issue was closed during tracker reconciliation.
- `#319` is no longer a repo blocker. The generated TypeScript SDK now exposes `game.preload(...)`, the Flappy Bird TypeScript example uses it, `cd sdks/typescript && npm test` covers it, and `cd sdks/typescript && npm run build:web` plus `cd examples/typescript/feature_lab && npm ci && npm run build:web` verify the browser packaging path. The remaining limitation is scope: the current generated preloader covers the SDK's path-based texture/font loading path, not every possible future asset class.
- Fixed stale docs:
  - `/Users/aramhammoudeh/dev/game/GoudEngine/examples/python/README.md` no longer implies that Python rendering is still unavailable.
  - `/Users/aramhammoudeh/dev/game/GoudEngine/sdks/typescript/README.md` now links developers to an explicit browser gotchas guide instead of leaving the web caveats implicit.

## Execution Waves

### Wave 1

- Normalize the audit ledger and tracker state.
- Use the audit CSV batch to finalize per-row verdicts.
- Comment progress to `#114`.

### Wave 2

- Fix Python export/generation mismatch.
- Fix C# generated surface completeness from a fresh checkout.
- Fix TS networking loopback failure.
- Turn FFI signature mismatches into hard failures.
- Remove handwritten public networking wrappers in favor of generated equivalents.

### Wave 3

- Collapse SDK generation onto one Rust-derived source of truth.
- Add clean-room regeneration.
- Generated file size enforcement is waived for this release by explicit user direction.

### Wave 4

- Add shared `Feature Lab` in Rust, C#, Python, TS desktop, and TS web.
- Close docs/tutorial/example parity gaps.
- Generate API docs and reusable snippets from the same source of truth.

### Wave 5

- Harden CI drift/docs/release gates.
- Make coverage, SDK regeneration, docs, and example proof mandatory.

## Verification Baseline

- `cargo run -p lint-layers`
- `cargo check`
- `cargo test --workspace --quiet`
- `cargo test --lib sdk`
- `cargo fmt --all -- --check`
- `cargo clippy -- -D warnings`
- `cargo deny check`
- `python3 sdks/python/test_bindings.py`
- `python3 sdks/python/test_network_loopback.py`
- `dotnet build sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal`
- `DOTNET_ROOT_X64=/usr/local/share/dotnet/x64 /usr/local/share/dotnet/x64/dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal`
- `cd sdks/typescript && npm test`
- `cargo doc --no-deps -p goud-engine-core -p goud-engine`
- `GOUD_ENGINE_LIB=$PWD/target/release PYTHONPATH=$PWD/sdks/python PDOC_ALLOW_EXEC=1 pdoc --output-dir docs/book/api/python sdks/python/goud_engine`
- `cd sdks/typescript && npx typedoc --tsconfig tsconfig.typedoc.json --entryPoints src/generated/node/index.g.ts src/generated/web/index.g.ts src/generated/types/engine.g.ts src/generated/types/math.g.ts --out ../../docs/book/api/typescript --name "GoudEngine TypeScript SDK"`
- `export PATH="$HOME/.dotnet/tools:$PATH" && cd sdks/csharp && dotnet restore GoudEngine.csproj && dotnet build -c Release --no-restore && docfx build docfx.json`
- `cargo check --manifest-path examples/rust/feature_lab/Cargo.toml`
- `cargo run -p feature-lab`
- `python3 examples/python/feature_lab.py`
- `cd examples/csharp/feature_lab && dotnet build && DOTNET_ROLL_FORWARD=Major dotnet run --no-build`
- `cd examples/typescript/feature_lab && npm ci && npm run build:web && npx tsc --noEmit --target ES2020 --module ES2020 --moduleResolution bundler --types node --skipLibCheck desktop.ts && node -e "import('./dist/lab.js').then(()=>console.log('feature_lab dist import ok'))"`
- `bash scripts/clean-room-regenerate.sh`
- `PATH="$HOME/.cargo/bin:$HOME/.dotnet/tools:$PATH" bash scripts/clean-room-regenerate.sh --docs`
