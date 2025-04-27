#!/bin/bash

# increment_version.sh
#
# This script increments version numbers across the GoudEngine project files:
# - goud_engine/Cargo.toml (source of truth)
# - sdks/GoudEngine/GoudEngine.csproj
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
sed -i '' "s/<Version>$CURRENT_VERSION<\/Version>/<Version>$NEW_VERSION<\/Version>/" sdks/GoudEngine/GoudEngine.csproj

# Update all .csproj files in examples directory - handle both formats
find examples -name "*.csproj" -type f -exec sed -i '' -e "s/<PackageReference Include=\"GoudEngine\" Version=\"[0-9]*\.[0-9]*\.[0-9]*\" \/>/<PackageReference Include=\"GoudEngine\" Version=\"$NEW_VERSION\" \/>/" -e "s/<PackageReference Include=\"GoudEngine\" Version=\"[0-9]*\.[0-9]*\.[0-9]*\">/<PackageReference Include=\"GoudEngine\" Version=\"$NEW_VERSION\">/" {} \;

echo "Version update complete!"
