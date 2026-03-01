#!/bin/bash

# increment_version.sh
#
# DEPRECATED: Versioning is now handled automatically by release-please.
# Use conventional commits (feat:, fix:, etc.) and merge to main.
# release-please will create a Release PR with the correct version bumps.
#
# This script is kept for local development convenience only.
# It increments version numbers across ALL GoudEngine project files:
# - goud_engine/Cargo.toml (source of truth)
# - sdks/csharp/GoudEngine.csproj
# - sdks/typescript/package.json
# - sdks/typescript/native/Cargo.toml
# - sdks/rust/Cargo.toml
# - codegen/goud_sdk.schema.json
# - All .csproj files in /examples directory
#
# Usage:
#   ./increment_version.sh         # Increments patch version (0.0.X)
#   ./increment_version.sh --major # Increments major version (X.0.0)
#   ./increment_version.sh --minor # Increments minor version (0.X.0)

# Function to increment version based on flag
increment_version() {
    local version=$1
    local flag=$2
    
    IFS='.' read -r major minor patch <<< "$version"
    
    case $flag in
        "--major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "--minor")
            minor=$((minor + 1))
            patch=0
            ;;
        *)
            patch=$((patch + 1))
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Get the flag if provided
FLAG=$1

# Read current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' goud_engine/Cargo.toml | sed 's/version = "\(.*\)"/\1/')
NEW_VERSION=$(increment_version "$CURRENT_VERSION" "$FLAG")

echo "Updating version from $CURRENT_VERSION to $NEW_VERSION"

# Update Cargo.toml
sed -i '' "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" goud_engine/Cargo.toml

# Update GoudEngine.csproj
sed -i '' "s/<Version>$CURRENT_VERSION<\/Version>/<Version>$NEW_VERSION<\/Version>/" sdks/csharp/GoudEngine.csproj

# Update all .csproj files in examples directory - handle both formats
find examples -name "*.csproj" -type f -exec sed -i '' -e "s/<PackageReference Include=\"GoudEngine\" Version=\"[0-9]*\.[0-9]*\.[0-9]*\" \/>/<PackageReference Include=\"GoudEngine\" Version=\"$NEW_VERSION\" \/>/" -e "s/<PackageReference Include=\"GoudEngine\" Version=\"[0-9]*\.[0-9]*\.[0-9]*\">/<PackageReference Include=\"GoudEngine\" Version=\"$NEW_VERSION\">/" {} \;

# Update codegen schema version
sed -i '' "s/\"version\": \"[0-9]*\.[0-9]*\.[0-9]*\"/\"version\": \"$NEW_VERSION\"/" codegen/goud_sdk.schema.json

# Update TypeScript package.json version
sed -i '' "s/\"version\": \"[0-9]*\.[0-9]*\.[0-9]*\"/\"version\": \"$NEW_VERSION\"/" sdks/typescript/package.json

# Update TypeScript native Cargo.toml version
sed -i '' "s/^version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$NEW_VERSION\"/" sdks/typescript/native/Cargo.toml

# Update Rust SDK Cargo.toml version
sed -i '' "s/^version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$NEW_VERSION\"/" sdks/rust/Cargo.toml

echo "Version update complete!"
