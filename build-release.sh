#!/bin/sh
set -e

# Build for linux
cargo +stable build --release \
	&& strip target/release/neos_peeps \
	&& sstrip target/release/neos_peeps
# Create sha256 integrity hash
cd target/release \
	&& sha256sum neos_peeps > neos_peeps.sha256 \
	&& cd ../..

# Build for windows
cargo +stable build --release --target x86_64-pc-windows-gnu \
	&& strip target/x86_64-pc-windows-gnu/release/neos_peeps.exe

# Must package into a .zip due to ms defender not liking unsigned executables.
# Also create a sha256 integrity hash
cd target/x86_64-pc-windows-gnu/release \
	&& zip -u windows_neos_peeps.zip neos_peeps.exe \
	&& sha256sum windows_neos_peeps.zip > windows_neos_peeps.zip.sha256 \
	&& cd ../../..
