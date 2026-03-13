# FAQ and Troubleshooting

This page tracks common problems and fixes.
It is organized by category and currently covers more than 10 recurring build, runtime, SDK, graphics, platform, and tutorial issues.
To add another entry, open a PR against this file using the contribution format below.

Contribution format for new FAQ entries:

```md
### [Category] Short failure description

Symptoms:
- ...

Cause:
- ...

Fix:
1. ...
2. ...

Verification:
- command or observable result
```

## Build issues

### [Build] `cargo check` fails after pulling latest changes

Symptoms:

- New compile errors in generated SDK files.

Cause:

- Generated artifacts are out of sync.

Fix:

1. Run `./codegen.sh`
2. Run `bash scripts/check-generated-artifacts.sh`

Verification:

- `cargo check` completes without generated-file drift errors.

### [Build] `scripts/check-generated-artifacts.sh` fails

Symptoms:

- Missing generated files or diff output under generated directories.

Cause:

- Generated outputs were not refreshed.

Fix:

1. Run `./codegen.sh`
2. Run `python3 scripts/generate-doc-snippets.py`
3. Run `cd sdks/typescript && node scripts/generate-doc-media.mjs`
4. Run `python3 scripts/generate-showcase-docs.py`
5. Re-run `bash scripts/check-generated-artifacts.sh`

Verification:

- Script prints `Generated artifact check passed.`

### [Build] `mdbook build` fails

Symptoms:

- Include path errors or broken links.

Cause:

- Missing generated snippet/docs outputs or stale links.

Fix:

1. Run `cd sdks/typescript && node scripts/generate-doc-media.mjs`
2. Run `python3 scripts/generate-doc-snippets.py`
3. Run `python3 scripts/generate-showcase-docs.py`
4. Run `mdbook build`

Verification:

- `docs/book/` rebuilds with no fatal errors.

## Runtime issues

### [Runtime] `goudengine.list_contexts` returns no attachable routes

Symptoms:

- `cargo run -p goudengine-mcp` starts, but `goudengine.list_contexts` returns an empty list.

Cause:

- The target process was started without debugger mode enabled, or you launched a browser/WASM target that does not publish debugger routes in this batch.

Fix:

1. Start a desktop or headless process with debugger mode enabled before creation.
2. Use one of the shipped Feature Lab examples if you want a known-good route:
   - `./dev.sh --game feature_lab`
   - `python3 examples/python/feature_lab.py`
   - `cargo run -p feature-lab`
   - `./dev.sh --sdk typescript --game feature_lab`
3. Re-run `cargo run -p goudengine-mcp`, then call `goudengine.list_contexts` and `goudengine.attach_context`.

Verification:

- `goudengine.list_contexts` shows one of the stable Feature Lab labels such as `feature-lab-csharp-headless`, `feature-lab-python-headless`, `feature-lab-rust-headless`, or `feature-lab-typescript-desktop`.

### [Runtime] Python bindings fail with symbol/load errors

Symptoms:

- Import error or unresolved symbol when running Python examples.

Cause:

- Loader found old native library.

Fix:

1. `cargo build --release`
2. `GOUD_ENGINE_LIB=$PWD/target/release PYTHONPATH=$PWD/sdks/python python3 sdks/python/test_bindings.py`

Verification:

- Binding tests pass and examples start.

### [Runtime] C# tests fail on Apple Silicon architecture mismatch

Symptoms:

- Runtime load/test failures under x64 test paths on Apple Silicon.

Cause:

- Host/runtime architecture mismatch.

Fix:

1. Verify binaries:
   - `file sdks/csharp/runtimes/osx-x64/native/libgoud_engine.dylib`
   - `file sdks/csharp/runtimes/osx-arm64/native/libgoud_engine.dylib`
2. Run x64-hosted tests:
   - `DOTNET_ROOT_X64=/usr/local/share/dotnet/x64 /usr/local/share/dotnet/x64/dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal`

Verification:

- Tests pass on explicit x64 host path.

### [Runtime] TypeScript web example starts but renders blank

Symptoms:

- Page opens with no gameplay output.

Cause:

- Wrong import-map/asset path or running from `file://`.

Fix:

1. Use repo command: `./dev.sh --sdk typescript --game flappy_bird_web`
2. Confirm page is served by HTTP URL printed by `dev.sh`.

Verification:

- Game renders and input responds.

## SDK and codegen issues

### [SDK] `python3 codegen/validate_coverage.py` fails

Symptoms:

- Mapping coverage errors for FFI exports.

Cause:

- Rust exports changed without codegen mapping updates.

Fix:

1. Build once to refresh manifest.
2. Run `python3 codegen/validate.py`
3. Update `codegen/goud_sdk.schema.json` and/or `codegen/ffi_mapping.json`
4. Run `./codegen.sh`

Verification:

- Both validation commands pass.

### [SDK] TypeScript tests fail after codegen changes

Symptoms:

- Missing generated type exports or runtime wrapper mismatches.

Cause:

- Node/web generated entrypoints drifted.

Fix:

1. Run `./codegen.sh`
2. Run `cd sdks/typescript && npm test`

Verification:

- Test suite is green with regenerated outputs.

### [SDK] Clean-room regenerate fails

Symptoms:

- `scripts/clean-room-regenerate.sh` fails after deleting generated files.

Cause:

- Regeneration pipeline no longer rebuilds from source truth.

Fix:

1. Run `bash scripts/clean-room-regenerate.sh`
2. If docs are required, run:
   - `PATH="$HOME/.cargo/bin:$HOME/.dotnet/tools:$PATH" bash scripts/clean-room-regenerate.sh --docs`
3. Fix any step that fails before release.

Verification:

- Script completes without manual file restoration.

## Graphics and platform issues

### [Graphics] OpenGL context errors in CI/headless runs

Symptoms:

- Graphics tests fail in environments without display/GL context.

Cause:

- Tests requiring GL context run in unsupported environment.

Fix:

1. Prefer headless-safe smoke paths for CI.
2. Use helper context setup where graphics tests require it.

Verification:

- CI lane runs expected headless-safe coverage without GL crashes.

### [Platform] Linux desktop app fails to start with GLFW/OpenGL errors

Symptoms:

- Startup fails before first frame.

Cause:

- Missing system packages for GLFW/OpenGL.

Fix:

1. Install required OpenGL/GLFW runtime packages.
2. Re-run the same command via `dev.sh`.

Verification:

- Window opens and loop runs.

### [Platform] Web networking behavior differs from native SDKs

Symptoms:

- Browser networking path behaves differently from Node/native.

Cause:

- Browser SDK uses WASM/browser APIs, not native transports.

Fix:

1. Review [Web Platform Gotchas](web-platform-gotchas.md)
2. Validate with web-specific smoke paths.

Verification:

- Web networking behavior matches documented browser scope.

### [Platform] TypeScript web networking fails to connect

Symptoms:

- Browser client never reaches `peerCount() > 0`.

Cause:

- Wrong protocol, wrong URL, or sending before the WebSocket connection is established.

Fix:

1. Use `NetworkProtocol.WebSocket`
2. Point it at a browser-reachable `ws://` or `wss://` URL
3. Wait for `peerCount() > 0` before first send
4. Validate with:
   - `./dev.sh --sdk typescript --game sandbox_web`
   - `cd examples/typescript/feature_lab`
   - `npm run smoke:web-network`

Verification:

- Browser smoke passes with a real `ping`/`pong` roundtrip.

## Example and tutorial issues

### [Examples] Not sure which example to run first

Use this order:

1. Flappy Bird for baseline behavior parity.
2. Sandbox for the full cross-language feature tour.
3. Feature Lab for supplemental smoke coverage.
4. C# specialization demos for renderer/gameplay patterns.

Reference:

- `examples/README.md`
- [Sandbox Guide](sandbox.md)
- [Example Showcase](showcase.md)

### [Tutorial] Need the final code for Build Your First Game

Use shipped final projects:

- `examples/csharp/flappy_goud/`
- `examples/python/flappy_bird.py`
- `examples/typescript/flappy_bird/`
- `examples/rust/flappy_bird/`

If you need zip bundles, use the `git archive` commands in [Build Your First Game](build-your-first-game.md).
