#!/bin/bash
#
# sync-version.sh
#
# Propagates the current version to example .csproj files.
# release-please handles the main version files (Cargo.toml, package.json, etc.)
# but can't do recursive globs for example projects. Run this in CI after
# release-please updates the version.
#
# Usage:
#   ./scripts/sync-version.sh

set -euo pipefail

VERSION=$(grep '^version = ' goud_engine/Cargo.toml | head -1 | cut -d'"' -f2)

if [ -z "$VERSION" ]; then
  echo "ERROR: Could not read version from goud_engine/Cargo.toml"
  exit 1
fi

echo "Syncing version $VERSION to example .csproj files"

find examples -name "*.csproj" -type f | while read -r csproj; do
  if grep -q 'PackageReference Include="GoudEngine"' "$csproj"; then
    sed -i.bak \
      -e "s/\(PackageReference Include=\"GoudEngine\" Version=\"\)[0-9]*\.[0-9]*\.[0-9]*/\1$VERSION/" \
      "$csproj"
    rm -f "${csproj}.bak"
    echo "  Updated: $csproj"
  fi
done

echo "Done."
