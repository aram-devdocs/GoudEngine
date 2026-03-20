# GitHub Secrets Setup for Release Pipeline

This document describes all GitHub secrets required by the release pipeline (`.github/workflows/release.yml`).

## Environments

The release pipeline uses six GitHub environments:

- **npm**: TypeScript/Node.js package distribution
- **nuget**: C# package distribution
- **pypi**: Python package distribution (uses OIDC)
- **maven**: Kotlin package distribution
- **crates-io**: Rust package distribution
- **luarocks**: Lua package distribution

## Required Secrets by Environment

### npm Environment

**NPM_TOKEN**
- Where to obtain: [npmjs.com > Account Settings > Auth Tokens](https://npmjs.com/settings)
- Type: Granular access token
- Scope: Allow publish to package `goudengine`
- Used by: `publish-npm` job to publish TypeScript SDK to npm registry

### nuget Environment

**NUGET_API_KEY**
- Where to obtain: [nuget.org > Account > API Keys](https://www.nuget.org/account/apikeys)
- Type: API key with "Push new packages and package versions" scope
- Scope: Package `GoudEngine`
- Used by: `publish-nuget` job to publish C# SDK to NuGet.org

### pypi Environment

**No secret required** — uses OIDC trusted publisher instead.

Setup:
1. Go to [PyPI > GoudEngine > Publishing](https://pypi.org/project/goudengine/)
2. Add trusted publisher for GitHub:
   - Repository name: `aramhammoudeh/GoudEngine`
   - Workflow filename: `.github/workflows/release.yml`
   - Environment name: `pypi`

When configured, the workflow will use OpenID Connect (OIDC) to obtain temporary credentials automatically on each publish.

Used by: `publish-pypi` job to publish Python SDK to PyPI

### maven Environment

**MAVEN_USERNAME**
- Where to obtain: [Maven Central Repository > Account > User Token](https://s01.oss.sonatype.org/)
- Type: JIRA/OSS account username or user token (username format)
- Scope: Publish to `com.goudengine:goud-engine-kotlin`
- Used by: `publish-kotlin` Gradle plugin configuration

**MAVEN_PASSWORD**
- Where to obtain: [Maven Central Repository > Account > User Token](https://s01.oss.sonatype.org/)
- Type: User token password (not your account password)
- Scope: Paired with `MAVEN_USERNAME`
- Used by: `publish-kotlin` Gradle plugin configuration

**GPG_PRIVATE_KEY**
- Where to obtain: Export from your local GPG keyring
  ```bash
  gpg --armor --export-secret-key YOUR_KEY_ID | base64 | tr -d '\n'
  ```
- Type: Base64-encoded ASCII-armored GPG private key
- Scope: Used to sign Maven artifacts before upload
- Used by: `publish-kotlin` Gradle plugin for artifact signing

**GPG_PASSPHRASE**
- Where to obtain: The passphrase you set when creating the GPG key
- Type: Plaintext passphrase (GitHub Actions will only expose it during the Maven publish step)
- Scope: Unlocks the GPG private key during signing
- Used by: `publish-kotlin` Gradle plugin

### crates-io Environment

**CRATES_IO_TOKEN**
- Where to obtain: [crates.io > Account Settings > API Tokens](https://crates.io/me)
- Type: API token with "publish-update" scope minimum
- Scope: Publish to crates: `goud-engine`, `goud-engine-core`, `goud_engine_macros`
- Used by: `publish-crates` job to publish Rust crates to crates.io

### luarocks Environment

**LUAROCKS_API_KEY**
- Where to obtain: [luarocks.org > Account > Settings](https://luarocks.org/settings)
- Type: API key with upload scope
- Scope: Publish to LuaRocks module `goudengine`
- Used by: `publish-luarocks` job in the release workflow

## Optional Secrets

### Conan Package Manager

For optional C++ Conan distribution:

**CONAN_LOGIN_USERNAME** (optional)
- Where to obtain: Conan Center Index account or self-hosted Conan server
- Type: Conan registry username
- Scope: Publish C++ bindings to Conan Center

**CONAN_PASSWORD** (optional)
- Where to obtain: Conan Center Index account or self-hosted Conan server
- Type: Conan registry password
- Scope: Paired with `CONAN_LOGIN_USERNAME`

## Swift Package Index

Setup is **not** automated in CI. Perform once:

1. Go to [swiftpm.co](https://swiftpm.co)
2. Paste the GitHub URL: `https://github.com/aramhammoudeh/GoudEngine`
3. Follow the prompts to register the Swift package
4. Swift Package Index will monitor the repo for tagged releases automatically

No CI secret needed.

## Go Modules

**No secret required** — uses GitHub's built-in Go module proxy.

Setup:
1. Tag a release: `git tag v0.0.832`
2. Push the tag: `git push origin v0.0.832`
3. Go module proxy automatically caches the release: `go get github.com/aramhammoudeh/GoudEngine/sdks/go@v0.0.832`

## vcpkg Distribution (Manual)

C/C++ packages are distributed as native tarballs attached to the GitHub Release (see `publish-native-tarballs` job). To also list the package in vcpkg:

1. Fork [microsoft/vcpkg](https://github.com/microsoft/vcpkg)
2. Create a port directory: `ports/goud-engine/`
3. Add `portfile.cmake` that downloads the tarball from the GitHub Release:
   ```cmake
   vcpkg_download_distfile(ARCHIVE
       URLS "https://github.com/aram-devdocs/GoudEngine/releases/download/v${VERSION}/goud-engine-v${VERSION}-${VCPKG_TARGET_TRIPLET}.tar.gz"
       FILENAME "goud-engine-v${VERSION}-${VCPKG_TARGET_TRIPLET}.tar.gz"
       SHA512 <sha512-of-tarball>
   )
   ```
4. Add `vcpkg.json` manifest with package metadata
5. Submit a PR to `microsoft/vcpkg` with the new port
6. vcpkg will review and merge; after that users can install with `vcpkg install goud-engine`

No CI secret is needed -- the tarballs are public GitHub Release assets.

## Implementation Checklist

Use this checklist when setting up the release pipeline in a new GitHub organization:

- [ ] Create `npm` environment; add `NPM_TOKEN`
- [ ] Create `nuget` environment; add `NUGET_API_KEY`
- [ ] Create `pypi` environment; configure OIDC trusted publisher at PyPI
- [ ] Create `maven` environment; add `MAVEN_USERNAME`, `MAVEN_PASSWORD`, `GPG_PRIVATE_KEY`, `GPG_PASSPHRASE`
- [ ] Create `crates-io` environment; add `CRATES_IO_TOKEN`
- [ ] Create `luarocks` environment; add `LUAROCKS_API_KEY`
- [ ] Register Swift Package at [swiftpm.co](https://swiftpm.co)
- [ ] Verify `publish-native-tarballs` job runs without secrets (uses `GITHUB_TOKEN` automatically)
- [ ] Optional: Create Conan secrets if C++ distribution is desired

## Troubleshooting

### "Secret not found" error

- Check that the secret is created in the correct environment
- Verify the secret name matches exactly (case-sensitive)
- Check that the workflow file references the secret with `${{ secrets.NAME }}`

### PyPI OIDC token fails

- Verify the trusted publisher is configured at [PyPI > GoudEngine](https://pypi.org/project/goudengine/settings/publishing/)
- Ensure environment name in workflow matches exactly: `pypi`
- Ensure repository name matches: `aramhammoudeh/GoudEngine`

### Maven Central publishes but artifacts don't appear

- Run `./sdks/kotlin/gradlew publishToSonatype closeAndReleaseSonatypeStagingRepository` locally to verify credentials
- Check that GPG key ID is correctly configured in `sdks/kotlin/build.gradle.kts`
- Verify GPG key is registered at Sonatype: https://central.sonatype.org/publish/requirements/gpg/

## References

- GitHub Environments: https://docs.github.com/en/actions/deployment/targeting-different-environments/using-environments-for-deployment
- OIDC Trusted Publishers: https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/about-security-hardening-with-openid-connect
- Release workflow source: `.github/workflows/release.yml`
