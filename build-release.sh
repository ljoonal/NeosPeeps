#!/bin/sh
set -e

# Build for linux
cargo +stable build --release
# Create sha256 integrity hash
cd target/release && sha256sum neos_peeps > neos_peeps.sha256 && cd ../..

# Build for windows
cargo +stable build --release --target x86_64-pc-windows-gnu

# Windows AV pseudo-requires code to be signed. 
# So doing that with a self-signed cert.
# Also provide a sha256 hash.
rm "target/x86_64-pc-windows-gnu/release/win-neos_peeps.exe" \
	& echo "Signing windows executable" \
	&& osslsigncode sign -h sha256 \
	-in "target/x86_64-pc-windows-gnu/release/neos_peeps.exe" \
	-out "target/x86_64-pc-windows-gnu/release/win-neos_peeps.exe" \
	-pkcs12 "ljoonal.pfx" -askpass \
	&& cd target/x86_64-pc-windows-gnu/release \
	&& sha256sum win-neos_peeps.exe > win-neos_peeps.exe.sha256 \
	&& cd ../../..
