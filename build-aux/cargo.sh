#!/usr/bin/env bash

set -e

SOURCE_ROOT="$1"
BUILD_ROOT="$2"
OUTPUT="$3"
BUILDTYPE="$4"
APP_BIN="$5"

export CARGO_TARGET_DIR="$BUILD_ROOT/target"

CARGO_ARGS=(--manifest-path "$SOURCE_ROOT/Cargo.toml" --target-dir "$CARGO_TARGET_DIR")

# If Flatpak cargo vendor directory exists, use offline vendored builds
if [ -f "$SOURCE_ROOT/cargo/config.toml" ]; then
    export CARGO_HOME="$SOURCE_ROOT/cargo"
    CARGO_ARGS+=(--offline)
elif [ -d "$SOURCE_ROOT/cargo-home" ]; then
    export CARGO_HOME="$SOURCE_ROOT/cargo-home"
else
    export CARGO_HOME="${CARGO_HOME:-$BUILD_ROOT/cargo-home}"
fi

if [ "$BUILDTYPE" = "release" ]; then
    CARGO_ARGS+=(--release)
    BIN_DIR="release"
else
    BIN_DIR="debug"
fi

cargo build "${CARGO_ARGS[@]}"

cp "$CARGO_TARGET_DIR/$BIN_DIR/$APP_BIN" "$OUTPUT"
