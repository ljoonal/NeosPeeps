#!/bin/sh
set -e

# Build for linux
cargo build --release && strip target/release/neos_peeps && sstrip target/release/neos_peeps
# Create sha256 integrity hash
sha256sum target/release/neos_peeps > target/release/neos_peeps.sha256

# Build for windows
cargo build --release --target x86_64-pc-windows-gnu && strip x86_64-pc-windows-gnu/release/neos_peeps.exe
# Set the icon

# Create sha256 integrity hash
sha256sum x86_64-pc-windows-gnu/release/neos_peeps.exe > x86_64-pc-windows-gnu/release/neos_peeps.exe.sha256
