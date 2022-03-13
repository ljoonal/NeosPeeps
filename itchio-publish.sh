#!/bin/sh
set -e

# Get current version
VERSION=`sed -n -r 's/version = "(.*)"/\1/p' Cargo.toml | head -n 1`
# Upload linux v
butler push target/release/neos_peeps ljoonal/neospeeps:linux --userversion $VERSION
# Upload win v
butler push target/x86_64-pc-windows-gnu/release/win-neos_peeps.exe ljoonal/neospeeps:win --userversion $VERSION
