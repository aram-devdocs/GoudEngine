#!/bin/bash

# Define paths
PROJECT_NAME="GoudEngine"
PACKAGE_OUTPUT_PATH="./nuget_package_output"
LOCAL_NUGET_FEED="$HOME/nuget-local"
NUPKG_FILES=$(find $PACKAGE_OUTPUT_PATH -name "*.nupkg")

# Function to deploy locally
deploy_local() {
    echo "Deploying packages to local NuGet feed..."

    # Ensure packages exist
    if [ -z "$NUPKG_FILES" ]; then
        echo "No NuGet packages found in $PACKAGE_OUTPUT_PATH. Please build the project first."
        exit 1
    fi

    # Create local NuGet feed if it doesn't exist
    mkdir -p "$LOCAL_NUGET_FEED"

    # Add local feed to NuGet sources
    dotnet nuget add source "$LOCAL_NUGET_FEED" --name LocalGoudFeed --configfile NuGet.config

    # Push all packages to local feed at once
    dotnet nuget push "$PACKAGE_OUTPUT_PATH/*.nupkg" --source "$LOCAL_NUGET_FEED"

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
    ./build.sh

    deploy_local
elif [ "$1" == "--prod" ]; then
    # run ./build.sh first
    ./build.sh

    deploy_prod
else
    echo "Usage: ./package.sh [--local | --prod]"
    exit 1
fi
