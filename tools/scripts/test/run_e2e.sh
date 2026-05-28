#!/usr/bin/env bash
# ==============================================================================
# Playwright E2E：后台启动 vite preview，跑完即清理。
# 避免 Playwright webServer 内嵌子进程在 Windows 上卡死/不退出。
#
# Usage: run_e2e.sh [--no-build] [playwright args...]
# 详见 docs/rust/troubleshooting.md § Playwright e2e 卡住
# ==============================================================================
set -euo pipefail

SCRIPT_DIR_THIS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR_THIS}/../../.." && pwd)"
# shellcheck source=../_common.sh
source "${PROJECT_ROOT}/tools/scripts/_common.sh"

E2E_PREVIEW_HOST="${E2E_PREVIEW_HOST:-127.0.0.1}"
E2E_PREVIEW_PORT="${E2E_PREVIEW_PORT:-4174}"
E2E_PREVIEW_URL="http://${E2E_PREVIEW_HOST}:${E2E_PREVIEW_PORT}"
E2E_PREVIEW_PID_FILE="${PROJECT_ROOT}/.e2e-preview.pid"
E2E_PREVIEW_LOG="${PROJECT_ROOT}/.e2e-preview.log"
E2E_OWNS_PREVIEW=false
SKIP_BUILD=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-build)
            SKIP_BUILD=true
            shift
            ;;
        *)
            break
            ;;
    esac
done

stop_owned_preview() {
    if [[ "$E2E_OWNS_PREVIEW" != true ]]; then
        return
    fi
    if [[ -f "$E2E_PREVIEW_PID_FILE" ]]; then
        local pid
        pid="$(cat "$E2E_PREVIEW_PID_FILE")"
        if kill "$pid" 2>/dev/null; then
            log_info "Stopped e2e preview (PID ${pid})"
        fi
        rm -f "$E2E_PREVIEW_PID_FILE"
    fi
    E2E_OWNS_PREVIEW=false
}
trap stop_owned_preview EXIT

ensure_build() {
    if [[ "$SKIP_BUILD" == true ]]; then
        return
    fi
    if [[ ! -f "${WEB_ADMIN_DIST_DIR}/index.html" ]]; then
        log_info "admin/web/dist missing — running build"
        (cd "$WEB_ADMIN_DIR" && npm run build)
    fi
}

preview_is_up() {
    curl -sSf -o /dev/null "${E2E_PREVIEW_URL}/admin/" 2>/dev/null
}

ensure_preview() {
    if preview_is_up; then
        log_info "Preview already reachable at ${E2E_PREVIEW_URL}/admin/"
        return
    fi

    log_info "Starting vite preview in background (${E2E_PREVIEW_URL})"
    : > "$E2E_PREVIEW_LOG"
    (
        cd "$WEB_ADMIN_DIR"
        exec bun run preview
    ) >> "$E2E_PREVIEW_LOG" 2>&1 &
    echo $! > "$E2E_PREVIEW_PID_FILE"
    E2E_OWNS_PREVIEW=true

    if ! wait_for_http "${E2E_PREVIEW_URL}/admin/" 30 1; then
        log_err "Preview failed to start — see ${E2E_PREVIEW_LOG}"
        tail -n 20 "$E2E_PREVIEW_LOG" 2>/dev/null || true
        exit 1
    fi
}

log_step "=== Admin Web E2E (Playwright) ==="
require_cmd curl "install curl or use Git Bash on Windows"
require_cmd bun "install Bun from https://bun.sh"

ensure_build
ensure_preview

cd "$WEB_ADMIN_DIR"
export E2E_SKIP_WEBSERVER=1
export WEB_ADMIN_BASE_URL="$E2E_PREVIEW_URL"

log_info "Running Playwright (external preview, no webServer subprocess)"
bunx playwright test "$@"
exit_code=$?

if [[ $exit_code -eq 0 ]]; then
    log_ok "E2E passed"
else
    log_err "E2E failed (exit ${exit_code})"
fi

exit "$exit_code"
