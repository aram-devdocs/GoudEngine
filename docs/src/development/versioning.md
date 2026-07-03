# Versioning

GoudEngine follows [Semantic Versioning](https://semver.org/). Version, git
tags, and changelog are managed by tooling, not by hand.

## Alpha Caveat

The project is in the `0.0.x` series. Under SemVer, a `0.0.x` package makes no
compatibility promise: breaking changes MAY land in any release. Pin an exact
version if you need stability, and read the changelog before upgrading.

## Who Owns the Version

[release-please](https://github.com/googleapis/release-please) owns the version
bump and the changelog. It reads
[Conventional Commits](https://www.conventionalcommits.org/) on the default
branch, computes the next version, and opens a release PR that updates the
version and `CHANGELOG.md`. Do not hand-edit the version or the changelog; write
correct commit messages and let the release PR carry them.

The release configuration lives in `release-please-config.json` and
`.release-please-manifest.json` at the repo root.

## MSRV

The minimum supported Rust version is pinned in `rust-toolchain.toml`. Treat an
MSRV bump as a notable change: it belongs in the changelog and MUST be raised
deliberately. When you bump the channel in `rust-toolchain.toml`, update the
MSRV note in `CONTRIBUTING.md` in the same change.
