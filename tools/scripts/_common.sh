#!/usr/bin/env bash
# ==============================================================================
# 公共基础库：路径常量、颜色输出、日志函数、工具函数
# 被所有其他脚本 source 引用
# ==============================================================================
set -euo pipefail

# ── 路径常量 ─────────────────────────────────────────────────────────────────
# 仓库根 = PROJECT_ROOT。本文件位于 tools/scripts/，上两级即为根。
# 注意：tools/scripts/test/ 下的脚本需上三级才能到根（见 run_backend_tests.sh）。
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
RUST_DIR="${PROJECT_ROOT}"  # backward-compat alias used by older scripts
MIGRATION_DIR="${PROJECT_ROOT}/migrations"
CONFIG_FILE="${PROJECT_ROOT}/config/app.toml"
NEXTEST_CONFIG_FILE="${PROJECT_ROOT}/config/nextest.toml"
WEB_ADMIN_DIR="${PROJECT_ROOT}/admin/web"
WEB_ADMIN_DIST_DIR="${WEB_ADMIN_DIR}/dist"

# Docker Compose 文件
COMPOSE_DEV="${PROJECT_ROOT}/deploy/docker-compose.dev.yml"
COMPOSE_TEST="${PROJECT_ROOT}/deploy/docker-compose.test.yml"

# ── 连接 URL ─────────────────────────────────────────────────────────────────
DB_DEV_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/wechatbot}"
DB_TEST_URL="postgres://postgres:postgres@localhost:5433/wechatbot"
REDIS_DEV_URL="redis://127.0.0.1:6379"

# Admin 服务
ADMIN_HOST="${ADMIN_HOST:-127.0.0.1}"
ADMIN_PORT="${ADMIN_PORT:-8787}"
ADMIN_URL="http://${ADMIN_HOST}:${ADMIN_PORT}"
ADMIN_PID_FILE="${PROJECT_ROOT}/.admin.pid"
ADMIN_LOG_FILE="${PROJECT_ROOT}/.admin.log"
WORKER_PID_FILE="${PROJECT_ROOT}/.worker.pid"
WORKER_LOG_FILE="${PROJECT_ROOT}/.worker.log"
ADMIN_BIN_REL="target/debug/admin"
WORKER_BIN_REL="target/debug/worker"
# Windows 下二进制文件带 .exe 后缀，unix 下不带
if [[ "${OSTYPE:-}" == msys || "${OSTYPE:-}" == cygwin || "${OSTYPE:-}" == win32 ]]; then
    ADMIN_BIN_REL="target/debug/admin.exe"
    WORKER_BIN_REL="target/debug/worker.exe"
fi
ADMIN_BIN="${PROJECT_ROOT}/${ADMIN_BIN_REL}"
WORKER_BIN="${PROJECT_ROOT}/${WORKER_BIN_REL}"

# ── 颜色定义 ─────────────────────────────────────────────────────────────────
if [[ -t 1 ]]; then
    COLOR_GREEN='\033[0;32m'
    COLOR_RED='\033[0;31m'
    COLOR_YELLOW='\033[0;33m'
    COLOR_BLUE='\033[0;34m'
    COLOR_CYAN='\033[0;36m'
    COLOR_BOLD='\033[1m'
    COLOR_NC='\033[0m'
else
    COLOR_GREEN=''; COLOR_RED=''; COLOR_YELLOW=''; COLOR_BLUE=''
    COLOR_CYAN=''; COLOR_BOLD=''; COLOR_NC=''
fi

# ── 日志函数 ─────────────────────────────────────────────────────────────────
log_info()  { echo -e "${COLOR_BLUE}[INFO]${COLOR_NC}  $*"; }
log_ok()    { echo -e "${COLOR_GREEN}[OK]${COLOR_NC}    $*"; }
log_warn()  { echo -e "${COLOR_YELLOW}[WARN]${COLOR_NC}  $*"; }
log_err()   { echo -e "${COLOR_RED}[ERROR]${COLOR_NC} $*"; }
log_step()  { echo -e "\n${COLOR_BOLD}${COLOR_CYAN}$*${COLOR_NC}"; }

# ── 工具函数 ─────────────────────────────────────────────────────────────────

# 检测 docker compose 命令，设置全局 COMPOSE_CMD 数组
detect_compose_cmd() {
    if docker compose version &>/dev/null; then
        COMPOSE_CMD=(docker compose)
    elif command -v docker-compose &>/dev/null; then
        COMPOSE_CMD=(docker-compose)
    else
        log_err "docker compose not found in PATH"
        exit 1
    fi
}

# 检查命令是否可用，否则报错退出
require_cmd() {
    local cmd="$1"
    local hint="${2:-}"
    if ! command -v "$cmd" &>/dev/null; then
        log_err "'$cmd' not found in PATH ${hint:+($hint)}"
        exit 1
    fi
}

# 轮询等待 PostgreSQL 就绪
# 参数: db_url [timeout_sec=30] [interval_sec=1]
wait_for_pg() {
    local pg_url="${1:-$DB_DEV_URL}"
    local timeout="${2:-30}"
    local interval="${3:-1}"
    local elapsed=0

    log_info "Waiting for PostgreSQL... (timeout ${timeout}s)"
    while [[ $elapsed -lt $timeout ]]; do
        if psql_exec_select "$pg_url" "SELECT 1" &>/dev/null; then
            log_ok "PostgreSQL is ready"
            return 0
        fi
        sleep "$interval"
        elapsed=$((elapsed + interval))
    done
    log_err "PostgreSQL did not become ready within ${timeout}s"
    return 1
}

# 轮询等待 HTTP 端点响应
wait_for_http() {
    local url="${1:-$ADMIN_URL/healthz}"
    local timeout="${2:-60}"
    local interval="${3:-2}"
    local elapsed=0

    log_info "Waiting for HTTP endpoint... ($url)"
    while [[ $elapsed -lt $timeout ]]; do
        if curl -sSf -o /dev/null "$url" 2>/dev/null; then
            log_ok "HTTP endpoint is reachable"
            return 0
        fi
        sleep "$interval"
        elapsed=$((elapsed + interval))
    done
    log_err "HTTP endpoint not reachable within ${timeout}s: $url"
    return 1
}

# 在数据库已连接时执行 SQL 查询 (SELECT)，返回结果
psql_exec_select() {
    local pg_url="$1"
    local sql="$2"
    if command -v psql &>/dev/null; then
        psql "$pg_url" -Atc "$sql" 2>/dev/null
    else
        # fallback: 通过 docker exec 在容器内执行
        docker exec -i postgres psql -U postgres -d wechatbot -Atc "$sql" 2>/dev/null
    fi
}

# 执行 SQL 文件 (如迁移文件)
psql_exec_file() {
    local pg_url="$1"
    local file="$2"
    if [[ "$file" == "/dev/stdin" || "$file" == "/proc/self/fd/0" ]]; then
        if command -v psql &>/dev/null; then
            psql "$pg_url"
        else
            docker exec -i postgres psql -U postgres -d wechatbot
        fi
    elif command -v psql &>/dev/null; then
        psql "$pg_url" -f "$file"
    else
        docker exec -i postgres psql -U postgres -d wechatbot < "$file"
    fi
}

# 执行单条 SQL 语句 (DDL/DML)
psql_exec() {
    local pg_url="$1"
    local sql="$2"
    if command -v psql &>/dev/null; then
        psql "$pg_url" -c "$sql"
    else
        docker exec -i postgres psql -U postgres -d wechatbot -c "$sql"
    fi
}

require_free_port() {
    local port="$1"
    if command -v lsof &>/dev/null && lsof -iTCP:"${port}" -sTCP:LISTEN -t &>/dev/null; then
        local owner_pid
        owner_pid=$(lsof -iTCP:"${port}" -sTCP:LISTEN -t | head -n 1)
        log_err "port ${port} is already in use by pid ${owner_pid}"
        return 1
    fi
    return 0
}

# ── 启动模式：dev（带 mock 种子）vs deploy（不灌种）──────────────────────────
start_mode_help() {
    cat <<'EOF'
  --dev         Dev/test mode: migrate + seed mock data (default)
  --deploy      Deploy mode: migrate only, no mock seed
  --with-seed   Same as --dev
  --no-seed     Same as --deploy
EOF
}

log_start_mode() {
    local do_seed="$1"
    if [[ "$do_seed" == "true" ]]; then
        log_info "Start mode: DEV — mock seed (bot-001..006, sessions, messages)"
    else
        log_info "Start mode: DEPLOY — no mock seed (empty business data after migrate)"
    fi
}

apply_db_migrate_and_optional_seed() {
    local do_seed="$1"
    bash "${SCRIPT_DIR}/db.sh" migrate
    if [[ "$do_seed" == "true" ]]; then
        bash "${SCRIPT_DIR}/db.sh" seed
    else
        log_info "Skipping db.sh seed (deploy mode)"
    fi
}

# 停止 dev 栈：worker → admin → Docker（与 start_all.sh / start.sh 启动顺序相反）
# 参数: remove_volumes — true 时 services.sh down -v
stop_dev_stack() {
    local remove_volumes="${1:-false}"

    log_step "Stopping forwarder worker..."
    bash "${SCRIPT_DIR}/worker.sh" stop

    log_step "Stopping admin server..."
    bash "${SCRIPT_DIR}/admin.sh" stop

    log_step "Stopping development services..."
    if [[ "$remove_volumes" == "true" ]]; then
        bash "${SCRIPT_DIR}/services.sh" down -v
    else
        bash "${SCRIPT_DIR}/services.sh" down
    fi
}
