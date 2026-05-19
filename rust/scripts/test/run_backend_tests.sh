#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR_THIS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_ROOT_DIR="$(cd "${SCRIPT_DIR_THIS}/../.." && pwd)"
source "${RUST_ROOT_DIR}/scripts/_common.sh"

log_step "Running backend tests"
cd "$RUST_DIR"
cargo test --lib
bash "${RUST_ROOT_DIR}/scripts/test_all.sh"
