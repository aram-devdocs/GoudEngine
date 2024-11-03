#!/bin/bash
# install.sh

echo "Installing Rust dependencies and tools..."

# Install Rust components if not installed
if ! command -v cargo &> /dev/null
then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi

# Install cbindgen for generating bindings
cargo install cbindgen

echo "Installation complete."