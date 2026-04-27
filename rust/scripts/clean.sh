#!/usr/bin/env bash
# ==============================================================================
# 全量清理：停止所有容器、删除数据卷、清理编译产物和运行时文件
#
# Usage: clean.sh [--all]
#   --all    Also remove Docker volumes, target/, data/media/, .admin.* files
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

CLEAN_ALL=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --all)   CLEAN_ALL=true; shift ;;
        --help|-h)
            echo "Usage: $(basename "$0") [--all]"
            echo "  default  Stop all containers"
            echo "  --all    Also remove volumes, build artifacts, runtime files"
            exit 0
            ;;
        *) shift ;;
    esac
done

if $CLEAN_ALL; then
    log_warn "This will remove ALL data (volumes, build artifacts, logs)."
    echo -n "${COLOR_RED}Are you sure? [y/N] ${COLOR_NC}"
    read -r confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        log_info "Cancelled."
        exit 0
    fi
fi

log_step "=== Cleaning Up ==="

# ── 停止 dev 和 test 容器 ──────────────────────────────────────────────────────
if command -v docker &>/dev/null; then
    detect_compose_cmd

    log_info "Stopping dev services..."
    "${COMPOSE_CMD[@]}" -f "${COMPOSE_DEV}" down 2>/dev/null || true

    log_info "Stopping test services..."
    "${COMPOSE_CMD[@]}" -f "${COMPOSE_TEST}" down 2>/dev/null || true

    if $CLEAN_ALL; then
        log_info "Removing dev volumes..."
        "${COMPOSE_CMD[@]}" -f "${COMPOSE_DEV}" down -v --remove-orphans 2>/dev/null || true

        log_info "Removing test volumes..."
        "${COMPOSE_CMD[@]}" -f "${COMPOSE_TEST}" down -v --remove-orphans 2>/dev/null || true
    fi

    log_ok "Containers stopped"
fi

# ── 停止 admin 进程 ────────────────────────────────────────────────────────────
if [[ -f "$ADMIN_PID_FILE" ]]; then
    log_info "Stopping admin server..."
    bash "${SCRIPT_DIR}/admin.sh" stop 2>/dev/null || true
fi

# ── 深度清理 (--all) ──────────────────────────────────────────────────────────
if $CLEAN_ALL; then
    log_info "Removing build artifacts..."
    rm -rf "${RUST_DIR}/target"

    log_info "Removing runtime files..."
    rm -f "${RUST_DIR}/.admin.pid" "${RUST_DIR}/.admin.log"

    log_info "Removing media data..."
    rm -rf "${RUST_DIR}/data/media"

    log_ok "Deep clean complete"
else
    log_info "Use --all to also remove volumes, build artifacts, and runtime files"
fi

echo ""
log_ok "Cleanup complete"
