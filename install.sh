#!/bin/bash
# install.sh

echo "Installing GoudEngine dependencies..."

# Detect OS and install system dependencies
if [[ "$OSTYPE" == "linux-gnu"* ]] || grep -q Microsoft /proc/version 2>/dev/null; then
    echo "Detected Linux/WSL. Installing system dependencies..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            cmake \
            pkg-config \
            libasound2-dev \
            libxrandr-dev \
            libxinerama-dev \
            libxcursor-dev \
            libxi-dev \
            libgl1-mesa-dev

        # Install .NET SDK if not present
        if ! command -v dotnet &> /dev/null; then
            echo "Installing .NET SDK..."
            # Install Microsoft package repository
            wget https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
            sudo dpkg -i packages-microsoft-prod.deb
            rm packages-microsoft-prod.deb
            sudo apt-get update
            sudo apt-get install -y dotnet-sdk-8.0
        fi
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y \
            gcc \
            gcc-c++ \
            cmake \
            pkgconfig \
            alsa-lib-devel \
            libXrandr-devel \
            libXinerama-devel \
            libXcursor-devel \
            libXi-devel \
            mesa-libGL-devel
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --needed \
            base-devel \
            cmake \
            pkgconf \
            alsa-lib \
            libxrandr \
            libxinerama \
            libxcursor \
            libxi \
            mesa
    else
        echo "Warning: Unknown package manager. Please install ALSA, X11, and OpenGL development libraries manually."
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Detected macOS. System dependencies should be available via Xcode Command Line Tools."
    if ! xcode-select -p &> /dev/null; then
        echo "Installing Xcode Command Line Tools..."
        xcode-select --install
    fi
fi

# Install Rust components if not installed
if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Install cbindgen for generating bindings
cargo install cbindgen

echo "Installation complete."