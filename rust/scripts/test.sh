#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found in PATH"
  exit 1
fi

cd "${RUST_DIR}"
if cargo nextest --version >/dev/null 2>&1; then
  echo "Running tests (cargo nextest)..."
  cargo nextest run
else
  echo "cargo-nextest not found; install: cargo install cargo-nextest --locked"
  echo "Falling back to cargo test..."
  cargo test
fi
