#!/usr/bin/env bash
# Verify an uncached SDK setup can link a static-CRT ARM64 Windows executable.
#
# SPDX-License-Identifier: MIT

set -euo pipefail

work="$(mktemp -d)"
trap 'rm -rf -- "$work"' EXIT
export XWIN_CACHE_DIR="$work/cache"
export CARGO_TARGET_DIR="$work/target"
export RUSTFLAGS='-C target-feature=+crt-static'

mkdir -p "$work/src"
cat > "$work/Cargo.toml" << 'EOF'
[package]
name = "xwin-setup-probe"
version = "0.0.0"
edition = "2021"
EOF
printf 'fn main() { println!("Windows SDK setup probe"); }\n' > "$work/src/main.rs"
cd "$work"
cargo generate-lockfile --offline
cargo xwin build --locked --release --target aarch64-pc-windows-msvc

binary="$CARGO_TARGET_DIR/aarch64-pc-windows-msvc/release/xwin-setup-probe.exe"
llvm-readobj --file-headers "$binary" > pe.txt
grep -qx 'Format: COFF-ARM64' pe.txt
grep -qx 'ImageOptionalHeader {' pe.txt
grep -q 'Machine: IMAGE_FILE_MACHINE_ARM64 (0xAA64)' pe.txt
llvm-readobj --coff-imports "$binary" > imports.txt
if grep -qiE '^[[:space:]]*Name: (VCRUNTIME140|MSVCP140|ucrtbase)[.]dll[[:space:]]*$' imports.txt; then
    cat imports.txt >&2
    echo 'SDK setup probe unexpectedly imports a dynamic CRT' >&2
    exit 1
fi
echo 'PASS: uncached Windows SDK setup and static-CRT ARM64 link'
