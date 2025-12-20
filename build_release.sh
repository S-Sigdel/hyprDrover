#!/bin/bash
set -e

VERSION=$(awk -F'"' '/^version\s*=\s*"/ { print $2; exit }' Cargo.toml)
if [[ -z "$VERSION" ]]; then
	echo "Failed to detect version from Cargo.toml" >&2
	exit 1
fi
echo "Building hyprdrover v$VERSION..."

# 1. Build for x86_64 (Host)
echo "Building for x86_64-unknown-linux-gnu..."
cargo build --release

# 2. Build for ARM64 (Cross-compile)
echo "Building for aarch64-unknown-linux-gnu..."
# Ensure the linker is set for ARM
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
cargo build --release --target aarch64-unknown-linux-gnu

# 3. Prepare artifacts
echo "Preparing artifacts..."
mkdir -p artifacts

# Copy and rename x86_64 binary
# Note: Default cargo build puts it in target/release, not target/x86_64...
cp target/release/hyprdrover "artifacts/hyprdrover-v$VERSION-linux-x86_64"
echo "Created artifacts/hyprdrover-v$VERSION-linux-x86_64"

# Copy and rename ARM64 binary
cp target/aarch64-unknown-linux-gnu/release/hyprdrover "artifacts/hyprdrover-v$VERSION-linux-aarch64"
echo "Created artifacts/hyprdrover-v$VERSION-linux-aarch64"

echo "Build complete! Binaries are in artifacts/"
ls -lh artifacts/
