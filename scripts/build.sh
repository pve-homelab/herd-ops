#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p bin
cargo build --release
TARGET_ROOT="${CARGO_TARGET_DIR:-target}"
cp -f "${TARGET_ROOT}/release/dev-team" bin/dev-team
chmod +x bin/dev-team
echo "Installed bin/dev-team"
