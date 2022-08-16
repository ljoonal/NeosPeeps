#!/bin/sh
set -e

# Get current version
VERSION=`sed -n -r 's/version = "(.*)"/\1/p' Cargo.toml | head -n 1`
# Generate deps list for flatpak since flathub doesn't want networking during build
python3 flatpak-cargo-generator.py Cargo.lock
# Do the actual build & install locally for testing
flatpak-builder --user --install --force-clean "target/flatpak/$VERSION" xyz.ljoonal.neospeeps.yml
