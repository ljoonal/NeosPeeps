#!/bin/sh
set -e

# Get current version
VERSION=`sed -n -r 's/version = "(.*)"/\1/p' Cargo.toml | head -n 1`
# Create a gitea release draft
tea release create --draft --target main \
  --tag v$VERSION --title v$VERSION \
  -a target/release/neos_peeps \
  -a target/release/neos_peeps.sha256\
  -a target/x86_64-pc-windows-gnu/release/neos_peeps.exe \
  -a target/x86_64-pc-windows-gnu/release/neos_peeps.exe.sha256
