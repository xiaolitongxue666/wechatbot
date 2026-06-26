#!/usr/bin/env bash
# ==============================================================================
# 全栈启动：容器 → 迁移 → [可选 mock 种子] → admin + worker
#
# 两种启动模式：
#   --dev     测试/本地演示：迁移 + 灌入 mock 数据（默认）
#   --deploy  部署/生产：仅迁移，不灌 mock 数据
#
# Usage: start_all.sh [--dev|--deploy]
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

DO_SEED=true

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dev|--with-seed) DO_SEED=true; shift ;;
        --deploy|--no-seed) DO_SEED=false; shift ;;
        --help|-h)
            echo "Usage: $(basename "$0") [--dev|--deploy]"
            echo ""
            echo "Modes:"
            start_mode_help
            exit 0
            ;;
        *) shift ;;
    esac
done

log_step "=== WeChatBot Full Startup ==="
log_start_mode "$DO_SEED"

require_cmd docker "install Docker Desktop"
require_cmd cargo "install Rust from https://rustup.rs"
require_cmd bun "install Bun from https://bun.sh"
require_cmd curl "needed for health checks"

require_free_port "$ADMIN_PORT"

bash "${SCRIPT_DIR}/services.sh" up
wait_for_pg "$DB_DEV_URL" 60 2
apply_db_migrate_and_optional_seed "$DO_SEED"
bash "${SCRIPT_DIR}/dev/start_backend.sh"
bash "${SCRIPT_DIR}/dev/start_worker.sh"
bash "${SCRIPT_DIR}/status.sh"
printf "\n"
echo "Stop:  bash tools/scripts/stop_all.sh"
printf "\n"
