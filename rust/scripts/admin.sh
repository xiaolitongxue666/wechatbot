#!/usr/bin/env bash
# ==============================================================================
# 管理后台进程：启动 / 停止 / 日志查看
# 智能编译：检测已有二进制则直接运行，否则先 cargo build
#
# Usage: admin.sh {start|stop|logs}
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

CMD="${1:-}"

usage() {
    echo "Usage: $(basename "$0") {start|stop|logs}"
    echo ""
    echo "  start   Build (if needed) and start admin server in background"
    echo "  stop    Stop the running admin server"
    echo "  logs    Tail the admin server log file"
    exit 0
}

# ── 构建 (如需要) ─────────────────────────────────────────────────────────────
# 检测二进制文件是否存在，不存在则编译
ensure_built() {
    if [[ -f "$ADMIN_BIN" ]]; then
        log_info "Admin binary found (skip build)"
        return 0
    fi

    log_step "Building admin binary..."
    require_cmd cargo "install Rust from https://rustup.rs"
    cd "$RUST_DIR"
    cargo build --bin admin
    if [[ ! -f "$ADMIN_BIN" ]]; then
        log_err "Build failed: binary not found at $ADMIN_BIN"
        exit 1
    fi
    log_ok "Admin binary built"
}

# ── start ──────────────────────────────────────────────────────────────────────
cmd_start() {
    # 检查是否已在运行
    if [[ -f "$ADMIN_PID_FILE" ]]; then
        local pid
        pid=$(cat "$ADMIN_PID_FILE" 2>/dev/null || true)
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            log_warn "Admin server already running (PID: $pid)"
            echo "  URL:  $ADMIN_URL/admin"
            echo "  Logs: $ADMIN_LOG_FILE"
            return 0
        else
            # PID 文件存在但进程已死，清理
            rm -f "$ADMIN_PID_FILE"
        fi
    fi

    # 确保二进制存在；需要数据库就绪
    ensure_built
    require_cmd curl "needed for health check"

    log_step "Starting admin server..."
    cd "$RUST_DIR"
    nohup "$ADMIN_BIN" > "$ADMIN_LOG_FILE" 2>&1 &
    local pid=$!
    echo "$pid" > "$ADMIN_PID_FILE"

    # 等待服务就绪
    if wait_for_http "$ADMIN_URL/healthz" 60 2; then
        echo ""
        echo "  ${COLOR_BOLD}${COLOR_GREEN}Admin:  ${ADMIN_URL}/admin${COLOR_NC}"
        echo "  ${COLOR_BOLD}${COLOR_GREEN}API:    ${ADMIN_URL}/api/overview${COLOR_NC}"
        echo "  ${COLOR_BOLD}${COLOR_CYAN}PID:    ${pid}${COLOR_NC}"
        echo "  ${COLOR_BOLD}Logs:   ${ADMIN_LOG_FILE}${COLOR_NC}"
        echo ""
    else
        log_err "Admin server failed to start"
        cmd_stop
        exit 1
    fi
}

# ── stop ───────────────────────────────────────────────────────────────────────
cmd_stop() {
    if [[ ! -f "$ADMIN_PID_FILE" ]]; then
        log_info "Admin server is not running"
        return 0
    fi

    local pid
    pid=$(cat "$ADMIN_PID_FILE" 2>/dev/null || true)
    if [[ -z "$pid" ]]; then
        rm -f "$ADMIN_PID_FILE"
        return 0
    fi

    log_info "Stopping admin server (PID: $pid)..."
    if kill -0 "$pid" 2>/dev/null; then
        kill "$pid"
        # 等待进程退出
        local waited=0
        while kill -0 "$pid" 2>/dev/null && [[ $waited -lt 10 ]]; do
            sleep 0.5
            waited=$((waited + 1))
        done
        # 如果还没退出，强制终止
        if kill -0 "$pid" 2>/dev/null; then
            kill -9 "$pid" 2>/dev/null || true
        fi
    fi

    rm -f "$ADMIN_PID_FILE"
    log_ok "Admin server stopped"
}

# ── logs ───────────────────────────────────────────────────────────────────────
cmd_logs() {
    if [[ ! -f "$ADMIN_LOG_FILE" ]]; then
        log_warn "Log file not found: $ADMIN_LOG_FILE"
        exit 1
    fi
    echo "--- Admin server logs (tail -f) ---"
    tail -f "$ADMIN_LOG_FILE"
}

# ── 入口 ──────────────────────────────────────────────────────────────────────
case "${CMD}" in
    start)  cmd_start ;;
    stop)   cmd_stop ;;
    logs)   cmd_logs ;;
    help|--help|-h) usage ;;
    *)
        echo "Unknown command: ${CMD}"
        usage
        exit 1
        ;;
esac
