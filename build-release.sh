#!/bin/sh
set -e

# Build for linux
cargo +stable build --release && strip target/release/neos_peeps && sstrip target/release/neos_peeps
# Create sha256 integrity hash
sha256sum target/release/neos_peeps > target/release/neos_peeps.sha256

# Build for windows
cargo +stable build --release --target x86_64-pc-windows-gnu && strip target/x86_64-pc-windows-gnu/release/neos_peeps.exe

# Create sha256 integrity hash
sha256sum target/x86_64-pc-windows-gnu/release/neos_peeps.exe > target/x86_64-pc-windows-gnu/release/neos_peeps.exe.sha256
