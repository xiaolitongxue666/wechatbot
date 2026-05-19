#!/usr/bin/env bash
# ==============================================================================
# 全局状态检查：Docker 容器、数据库、管理后台
#
# Usage: status.sh
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

printf "\n"
printf "%b\n" "${COLOR_BOLD}=== WeChatBot Status ===${COLOR_NC}"
printf "\n"

# ── Docker 容器 ───────────────────────────────────────────────────────────────
printf "%b\n" "${COLOR_BOLD}Docker containers:${COLOR_NC}"
if command -v docker &>/dev/null; then
    detect_compose_cmd
    for compose_file in "$COMPOSE_DEV" "$COMPOSE_TEST"; do
        if "${COMPOSE_CMD[@]}" -f "$compose_file" ps 2>/dev/null | grep -q "Up"; then
            "${COMPOSE_CMD[@]}" -f "$compose_file" ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || echo "  (no services running)"
        fi
    done
else
    echo "  docker: not found"
fi
printf "\n"

# ── 数据库 ─────────────────────────────────────────────────────────────────────
printf "%b\n" "${COLOR_BOLD}Database ($DB_DEV_URL):${COLOR_NC}"
if psql_exec_select "$DB_DEV_URL" "SELECT 1" &>/dev/null 2>&1; then
    echo "  connected: yes"
    for table in bot_sessions chat_messages chat_media forward_events forward_dlq; do
        count=$(psql_exec_select "$DB_DEV_URL" "SELECT count(*) FROM ${table}" 2>/dev/null || echo "?")
        printf "  %-18s %s\n" "${table}:" "$count"
    done
else
    echo "  connected: no"
fi
printf "\n"

# ── Redis ──────────────────────────────────────────────────────────────────────
printf "%b\n" "${COLOR_BOLD}Redis (${REDIS_DEV_URL}):${COLOR_NC}"
if command -v redis-cli &>/dev/null; then
    if redis-cli -u "$REDIS_DEV_URL" PING &>/dev/null 2>&1; then
        echo "  connected: yes (PONG)"
    else
        echo "  connected: no"
    fi
else
    echo "  connected: unknown (redis-cli not found)"
fi
printf "\n"

# ── Admin 服务 ─────────────────────────────────────────────────────────────────
printf "%b\n" "${COLOR_BOLD}Admin server ($ADMIN_URL):${COLOR_NC}"
if curl -sSf -o /dev/null "$ADMIN_URL/healthz" 2>/dev/null; then
    echo "  reachable: yes"
    if [[ -f "$ADMIN_PID_FILE" ]]; then
        pid=$(cat "$ADMIN_PID_FILE")
        echo "  pid:       $pid"
    fi
    echo "  dashboard: ${ADMIN_URL}/admin"
    echo "  api:       ${ADMIN_URL}/admin/api/overview"
else
    echo "  reachable: no"
    if [[ -f "$ADMIN_PID_FILE" ]]; then
        echo "  pid file exists but server not responding, may have crashed"
    fi
fi
printf "\n"

# ── Forwarder Worker ──────────────────────────────────────────────────────────
printf "%b\n" "${COLOR_BOLD}Forwarder worker:${COLOR_NC}"
if [[ -f "$WORKER_PID_FILE" ]]; then
    worker_pid=$(cat "$WORKER_PID_FILE")
    if [[ -n "$worker_pid" ]] && kill -0 "$worker_pid" 2>/dev/null; then
        echo "  running:   yes"
        echo "  pid:       $worker_pid"
        echo "  logs:      $WORKER_LOG_FILE"
    else
        echo "  running:   no (stale pid file)"
    fi
else
    echo "  running:   no"
fi
printf "\n"

# ── Cargo / 编译状态 ───────────────────────────────────────────────────────────
printf "%b\n" "${COLOR_BOLD}Build:${COLOR_NC}"
if [[ -f "$ADMIN_BIN" ]]; then
    echo "  admin binary: $ADMIN_BIN ($(stat -c%s "$ADMIN_BIN" 2>/dev/null || stat -f%z "$ADMIN_BIN" 2>/dev/null || echo "?") bytes)"
else
    echo "  admin binary: not built"
fi
printf "\n"

printf "%b\n" "${COLOR_GREEN}Status check complete${COLOR_NC}"
