#!/usr/bin/env bash
# ==============================================================================
# 一键全量测试：启动测试容器 → 建库 → 编译 → 运行测试 → 清理
# 使用 docker-compose.test.yml (pg:5433, redis:6380)
# trap EXIT 保证容器必定清理
#
# Usage: test_all.sh
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

# 清理函数 — trap EXIT 保证必定执行
cleanup() {
    echo ""
    log_info "Cleaning up test services..."
    detect_compose_cmd
    "${COMPOSE_CMD[@]}" -f "${COMPOSE_TEST}" down -v --remove-orphans 2>/dev/null || true
}
trap cleanup EXIT

# ══════════════════════════════════════════════════════════════════════════════
log_step "=== WeChatBot Full Test Suite ==="

# ── 前置检查 ────────────────────────────────────────────────────────────────────
require_cmd docker "install Docker Desktop"
require_cmd cargo  "install Rust from https://rustup.rs"

# ── 第1步：启动测试容器 ────────────────────────────────────────────────────────
log_step "Step 1/5: Starting test services"
detect_compose_cmd
"${COMPOSE_CMD[@]}" -f "${COMPOSE_TEST}" down -v --remove-orphans 2>/dev/null || true
"${COMPOSE_CMD[@]}" -f "${COMPOSE_TEST}" up -d

# ── 第2步：等待 PostgreSQL 就绪 ────────────────────────────────────────────────
log_step "Step 2/5: Waiting for PostgreSQL"
# 使用测试数据库的健康检查
elapsed=0
timeout=60
while [[ $elapsed -lt $timeout ]]; do
    if docker compose -f "${COMPOSE_TEST}" ps postgres 2>/dev/null | grep -q "(healthy)"; then
        break
    fi
    sleep 1
    elapsed=$((elapsed + 1))
done
log_ok "Test PostgreSQL is healthy"

# ── 第3步：数据库迁移 ──────────────────────────────────────────────────────────
log_step "Step 3/5: Running database migrations"
DATABASE_URL="${DB_TEST_URL}" bash "${SCRIPT_DIR}/db.sh" migrate

# ── 第4步：编译项目 ────────────────────────────────────────────────────────────
log_step "Step 4/5: Building project"
cd "$RUST_DIR"
cargo build
log_ok "Build complete"

# ── 第5步：运行测试 ────────────────────────────────────────────────────────────
log_step "Step 5/5: Running tests"
export WECHATBOT_TEST_DATABASE_URL="$DB_TEST_URL"

cd "$RUST_DIR"
if cargo nextest --version &>/dev/null; then
    log_info "Using cargo-nextest"
    cargo nextest run --profile ci
else
    log_info "Using cargo test (install cargo-nextest for faster: cargo install cargo-nextest --locked)"
    cargo test
fi

# ── 完成 ────────────────────────────────────────────────────────────────────────
echo ""
log_ok "All tests complete"
echo ""
echo "  ${COLOR_GREEN}unit tests      - always run${COLOR_NC}"
echo "  ${COLOR_GREEN}integration tests - run against test DB${COLOR_NC}"
echo "  ${COLOR_GREEN}frontend tests   - run against test DB${COLOR_NC}"
echo ""

# cleanup 在 trap EXIT 时自动执行
