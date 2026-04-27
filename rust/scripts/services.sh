#!/usr/bin/env bash
# ==============================================================================
# 后台容器管理：启动/停止/状态/重启开发依赖服务
# 使用 docker-compose.dev.yml (postgres:5432, redis:6379, minio:9000)
#
# Usage: services.sh {up|down|down -v|status|restart}
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

CMD="${1:-}"

# 显示用法
usage() {
    echo "Usage: $(basename "$0") {up|down|down -v|status|restart}"
    echo ""
    echo "  up        Start PostgreSQL, Redis, MinIO containers"
    echo "  down      Stop containers (keep volumes)"
    echo "  down -v   Stop containers and remove volumes"
    echo "  status    Show container running status"
    echo "  restart   Stop then start containers"
    exit 0
}

# 执行 compose 命令
compose_cmd() {
    local args=("$@")
    detect_compose_cmd
    "${COMPOSE_CMD[@]}" -f "${COMPOSE_DEV}" "${args[@]}"
}

# ── up ────────────────────────────────────────────────────────────────────────
cmd_up() {
    log_step "Starting development services..."
    require_cmd docker "install Docker Desktop or Docker Engine"
    detect_compose_cmd

    compose_cmd up -d

    log_info "Services starting:"
    echo "  - PostgreSQL : 5432 (user: postgres, db: wechatbot)"
    echo "  - Redis      : 6379"
    echo "  - MinIO      : 9000 (console: 9001)"
    log_ok "Services are up"
}

# ── down ──────────────────────────────────────────────────────────────────────
cmd_down() {
    local remove_volumes=false
    if [[ "${2:-}" == "-v" ]]; then
        remove_volumes=true
    fi

    log_step "Stopping development services..."
    detect_compose_cmd

    if $remove_volumes; then
        compose_cmd down -v --remove-orphans
        log_ok "Services stopped, volumes removed"
    else
        compose_cmd down --remove-orphans
        log_ok "Services stopped (volumes preserved)"
    fi
}

# ── status ────────────────────────────────────────────────────────────────────
cmd_status() {
    detect_compose_cmd
    echo "--- Development services ---"
    compose_cmd ps
}

# ── restart ───────────────────────────────────────────────────────────────────
cmd_restart() {
    cmd_down
    cmd_up
}

# ── 入口 ──────────────────────────────────────────────────────────────────────
case "${CMD}" in
    up)
        cmd_up
        ;;
    down)
        cmd_down "$@"
        ;;
    status)
        cmd_status
        ;;
    restart)
        cmd_restart
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        echo "Unknown command: ${CMD}"
        usage
        exit 1
        ;;
esac
