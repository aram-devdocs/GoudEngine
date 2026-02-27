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
}

# Function to handle production deployment (TODO)
deploy_prod() {
    echo "TODO: Implement production deployment to nuget.org."
    echo "You need to ensure the following:"
    echo "1. Have a nuget.org API key."
    echo "2. Add the key to your environment or configure it securely."
    echo "3. Use the following command to publish:"
    echo "   dotnet nuget push <nupkg-file> --api-key <your-api-key> --source https://api.nuget.org/v3/index.json"
}

# Parse command-line arguments
if [ "$1" == "--local" ]; then
    # run ./build.sh first
    ./build.sh --release

    deploy_local
elif [ "$1" == "--prod" ]; then
    # run ./build.sh first
    ./build.sh --prod

    deploy_prod
else
    echo "Usage: ./package.sh [--local | --prod]"
    exit 1
fi
