#!/usr/bin/env bash
# ==============================================================================
# 一键启动开发环境：容器 → 迁移 → 种子 → 管理后台
# 默认会灌入种子数据，方便开发演示
#
# Usage: start.sh [--no-seed] [--no-admin] [--with-worker]
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

DO_SEED=true
DO_ADMIN=true
DO_WORKER=false

# 解析参数
while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-seed)  DO_SEED=false; shift ;;
        --no-admin) DO_ADMIN=false; shift ;;
        --with-worker) DO_WORKER=true; shift ;;
        --help|-h)
            echo "Usage: $(basename "$0") [--no-seed] [--no-admin] [--with-worker]"
            echo ""
            echo "  --no-seed    Skip seeding test data"
            echo "  --no-admin   Skip starting admin server"
            echo "  --with-worker Start forwarder worker after admin startup"
            exit 0
            ;;
        *) shift ;;
    esac
done

# ══════════════════════════════════════════════════════════════════════════════
log_step "=== WeChatBot Dev Environment Startup ==="
echo ""

# ── 第1步：检查前置条件 ────────────────────────────────────────────────────────
log_info "Checking prerequisites..."
require_cmd docker "install Docker Desktop"
require_cmd cargo  "install Rust from https://rustup.rs"
require_cmd bun "install Bun from https://bun.sh"
log_ok "Prerequisites satisfied"

# ── 第2步：启动后台容器 ────────────────────────────────────────────────────────
log_step "Step 1/4: Starting backend services"
bash "${SCRIPT_DIR}/services.sh" up

# ── 第3步：等待数据库就绪 ──────────────────────────────────────────────────────
log_step "Step 2/4: Waiting for PostgreSQL"
wait_for_pg "$DB_DEV_URL" 60 2

# ── 第4步：数据库迁移 ──────────────────────────────────────────────────────────
log_step "Step 3/4: Setting up database"
bash "${SCRIPT_DIR}/db.sh" migrate

if $DO_SEED; then
    bash "${SCRIPT_DIR}/db.sh" seed
else
    log_info "Seed data skipped (use --no-seed)"
fi

# ── 第5步：启动管理后台 ────────────────────────────────────────────────────────
if $DO_ADMIN; then
    log_step "Step 4/4: Starting admin server"
    bash "${SCRIPT_DIR}/dev/start_backend.sh"
else
    log_info "Admin server skipped (use --no-admin)"
fi

if $DO_WORKER; then
    log_step "Starting forwarder worker"
    bash "${SCRIPT_DIR}/dev/start_worker.sh"
fi

# ══════════════════════════════════════════════════════════════════════════════
echo ""
echo "========================================"
echo "  ${COLOR_BOLD}WeChatBot Dev Environment Ready${COLOR_NC}"
echo "========================================"
echo "  Admin:        ${COLOR_GREEN}${ADMIN_URL}/admin${COLOR_NC}"
echo "  Overview API: ${COLOR_CYAN}${ADMIN_URL}/admin/api/overview${COLOR_NC}"
echo "  Database:     postgres://localhost:5432/wechatbot"
echo "  Redis:        redis://localhost:6379"
echo "  MinIO:        http://localhost:9001 (console)"
echo "========================================"
echo ""
echo "Stop:  bash scripts/admin.sh stop && bash scripts/services.sh down"
echo "Clean: bash scripts/clean.sh --all"
echo ""
