#!/usr/bin/env bash
# ==============================================================================
# Echo Bot 开发模式：运行协议层面的回环验证
# 启动 echo_bot 示例，扫码后自动回复用户消息
#
# Usage: dev.sh
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

require_cmd cargo "install Rust from https://rustup.rs"

log_step "=== Echo Bot (Protocol Verification) ==="
log_info "This will print a QR code URL — scan it with WeChat to connect."
log_info "Press Ctrl+C to stop."
echo ""

cd "$RUST_DIR"
cargo run --example echo_bot
