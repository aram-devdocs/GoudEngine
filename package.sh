#!/bin/bash

# Define paths
PROJECT_NAME="GoudEngine"
PACKAGE_OUTPUT_PATH="./sdks/nuget_package_output"
LOCAL_NUGET_FEED="$HOME/nuget-local"

# Function to deploy locally
deploy_local() {
    echo "Deploying packages to local NuGet feed..."

    # Ensure package output directory exists
    if [ ! -d "$PACKAGE_OUTPUT_PATH" ]; then
        echo "No package output directory found. Build will create it."
    fi

    # Find packages after build
    NUPKG_FILES=$(find "$PACKAGE_OUTPUT_PATH" -name "*.nupkg" 2>/dev/null)

    # Ensure packages exist
    if [ -z "$NUPKG_FILES" ]; then
        echo "No NuGet packages found in $PACKAGE_OUTPUT_PATH. Please build the project first."
        exit 1
    fi

    # Show all packages that will be deployed
    echo "Found the following packages to deploy:"
    ls -l "$PACKAGE_OUTPUT_PATH"/*.nupkg

    # Create local NuGet feed if it doesn't exist
    mkdir -p "$LOCAL_NUGET_FEED"

    # Push all packages to local feed individually
    for package in "$PACKAGE_OUTPUT_PATH"/*.nupkg; do
        dotnet nuget push "$package" --source "$LOCAL_NUGET_FEED"
    done

    echo "Packages deployed to local NuGet feed at $LOCAL_NUGET_FEED."

    # Prune old NuGet packages — keep latest 3 versions
    echo "Pruning old NuGet packages..."
    KEEP_COUNT=3
    TOTAL_PKGS=$(ls -1 "$LOCAL_NUGET_FEED"/GoudEngine.*.nupkg 2>/dev/null | wc -l | tr -d ' ')
    REMOVE_COUNT=$((TOTAL_PKGS - KEEP_COUNT))
    if [ "$REMOVE_COUNT" -gt 0 ]; then
        ls -1 "$LOCAL_NUGET_FEED"/GoudEngine.*.nupkg | sort -V | head -n "$REMOVE_COUNT" | while read -r pkg; do
            echo "  Removing: $(basename "$pkg")"
            rm -f "$pkg"
        done
        echo "Pruned to latest $KEEP_COUNT versions."
    else
        echo "No old packages to prune."
    fi
}

# Production publishing to nuget.org is NOT handled by this script.
# It runs from CI: release-please cuts the version bump and tag, and the
# release workflow publishes the signed package. This is a deliberate no-op
# so local `--prod` invocations do not push unofficial builds to nuget.org.
deploy_prod() {
    echo "Production publishing is handled by the release workflow (release-please + CI),"
    echo "not by this script. Merge conventional commits to main and let the Release PR"
    echo "drive the nuget.org publish. Use './package.sh --local' for local iteration."
}

# Parse command-line arguments
if [ "$1" == "--local" ]; then
    # run ./build.sh first
    ./build.sh --release

    deploy_local
elif [ "$1" == "--prod" ]; then
    deploy_prod
else
    echo "Usage: ./package.sh [--local | --prod]"
    exit 1
fi
