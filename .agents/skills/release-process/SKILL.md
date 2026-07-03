---
name: release-process
description: How versioning and releases work via release-please, version.txt, and the version pins across the workspace
user-invocable: true
---

# Release Process

GoudEngine releases are automated with [release-please](https://github.com/googleapis/release-please).
You do not bump versions by hand. Land conventional-commit changes on `main`, and
release-please opens (and maintains) a Release PR that bumps the version, updates the
changelog, and — once merged — tags the release.

## When to Use

Read this when you need to understand how a version bump propagates, why a Release PR
looks the way it does, where a version string lives, or how to recover a stuck release.

## How It Works

- **Conventional commits drive the bump.** `feat:` → minor, `fix:`/`perf:` → patch,
  `feat!:`/breaking → major. Config lives in `release-please-config.json`
  (`release-type: simple`, pre-major bumping enabled).
- **`version.txt` is the source of truth** for the current released version, mirrored by
  `.release-please-manifest.json`. release-please reads and advances both.
- **Version pins are updated automatically.** `release-please-config.json` lists every
  file that carries a version under `extra-files`: `goud_engine/Cargo.toml`,
  `sdks/rust/Cargo.toml`, `sdks/typescript/native/Cargo.toml`, `goud_engine_macros/Cargo.toml`,
  `sdks/csharp/GoudEngine.csproj`, `sdks/python/pyproject.toml`,
  `sdks/typescript/package.json`, `codegen/goud_sdk.schema.json`, `ports/vcpkg/vcpkg.json`,
  the `sdks/kotlin/build.gradle.kts`, and every copied `goud_engine.h` / `goud.g.hpp`.
- **`x-release-please-version` markers.** In files where the version is not the obvious
  first string, a `// x-release-please-version` (or language-appropriate) comment marks
  the exact line release-please rewrites. They exist in `goud_engine/Cargo.toml`,
  `goud_engine_macros/Cargo.toml`, `sdks/rust/Cargo.toml`, `sdks/typescript/native/Cargo.toml`,
  `sdks/python/pyproject.toml`, `sdks/kotlin/build.gradle.kts`, and
  `goud_engine/build_support/c_header.rs`. Keep these comments intact — deleting one
  silently drops that file from future bumps.
- **Example projects are synced separately.** release-please cannot glob the example
  `.csproj` files, so `scripts/sync-version.sh` propagates the version to
  `examples/**/*.csproj` in CI after the bump.

## Steps (normal release)

1. Merge conventional-commit PRs to `main`.
2. release-please opens/updates a Release PR titled with the next version.
3. Review the changelog and the version-pin diff in that PR.
4. Merge the Release PR. release-please tags the release and the publish workflows run.

## Verification

- Confirm `version.txt` and `.release-please-manifest.json` agree after a release.
- Spot-check that the `extra-files` pins all moved to the new version (a stale pin means
  a missing/broken `x-release-please-version` marker).

## Manual Recovery

`increment_version.sh` is **deprecated** and kept only for local convenience — it bumps
`goud_engine/Cargo.toml`, the SDK manifests, `codegen/goud_sdk.schema.json`, the example
`.csproj` files, and `version.txt` in one shot. Prefer the release-please flow. Reach for
the script only to reproduce a version state locally, never as the path to ship a release.
