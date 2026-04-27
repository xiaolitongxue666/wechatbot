#!/usr/bin/env bash
# ==============================================================================
# 单元测试：不依赖外部服务（Postgres/Redis），纯 cargo 测试
#
# Usage: test.sh [--nextest] [--nocapture]
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

USE_NEXTEST=true
NOCAPTURE=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --nextest)    USE_NEXTEST=true; shift ;;
        --no-nextest) USE_NEXTEST=false; shift ;;
        --nocapture)  NOCAPTURE=true; shift ;;
        --help|-h)
            echo "Usage: $(basename "$0") [--nextest|--no-nextest] [--nocapture]"
            echo "  --nextest     Use cargo-nextest (default, if installed)"
            echo "  --no-nextest  Force cargo test"
            echo "  --nocapture   Show test output (cargo test only)"
            exit 0
            ;;
        *) shift ;;
    esac
done

require_cmd cargo "install Rust from https://rustup.rs"

cd "$RUST_DIR"
log_step "=== Running Unit Tests ==="

if $USE_NEXTEST && cargo nextest --version &>/dev/null; then
    log_info "Using cargo-nextest"
    cargo nextest run
else
    if $USE_NEXTEST; then
        log_warn "cargo-nextest not installed (install: cargo install cargo-nextest --locked)"
    fi
    log_info "Using cargo test"
    if $NOCAPTURE; then
        cargo test -- --nocapture
    else
        cargo test
    fi
fi

log_ok "Unit tests complete"
