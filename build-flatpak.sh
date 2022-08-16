#!/bin/sh
set -e

# Get current version
VERSION=`sed -n -r 's/version = "(.*)"/\1/p' Cargo.toml | head -n 1`

flatpak-builder --user --install --force-clean "target/flatpak/$VERSION" flatpak.yml
