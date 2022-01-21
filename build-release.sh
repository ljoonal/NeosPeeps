#!/bin/sh
set -e

# Build for linux
cargo +stable build --release && strip target/release/neos_peeps && sstrip target/release/neos_peeps
# Create sha256 integrity hash
cd target/release && sha256sum neos_peeps > neos_peeps.sha256 && cd ../..

# Build for windows
cargo +stable build --release --target x86_64-pc-windows-gnu && strip target/x86_64-pc-windows-gnu/release/neos_peeps.exe

# Create sha256 integrity hash
cd target/x86_64-pc-windows-gnu/release && sha256sum neos_peeps.exe > neos_peeps.exe.sha256 && cd ../../..
