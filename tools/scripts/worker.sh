#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

CMD="${1:-}"

usage() {
    echo "Usage: $(basename "$0") {start|stop|logs}"
    exit 0
}

ensure_worker_built() {
    if [[ -f "$WORKER_BIN" ]]; then
        return 0
    fi
    log_step "Building worker binary..."
    cd "$RUST_DIR"
    cargo build --bin worker
}

cmd_start() {
    if [[ -f "$WORKER_PID_FILE" ]]; then
        local pid
        pid=$(cat "$WORKER_PID_FILE" 2>/dev/null || true)
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            log_warn "Worker already running (PID: $pid)"
            return 0
        fi
        rm -f "$WORKER_PID_FILE"
    fi

    ensure_worker_built
    log_step "Starting forwarder worker..."
    cd "$RUST_DIR"
    nohup "$WORKER_BIN" > "$WORKER_LOG_FILE" 2>&1 &
    local pid=$!
    echo "$pid" > "$WORKER_PID_FILE"
    log_ok "Worker started (PID: $pid)"
}

cmd_stop() {
    if [[ ! -f "$WORKER_PID_FILE" ]]; then
        log_info "Worker is not running"
        return 0
    fi
    local pid
    pid=$(cat "$WORKER_PID_FILE" 2>/dev/null || true)
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
        kill "$pid" || true
    fi
    rm -f "$WORKER_PID_FILE"
    log_ok "Worker stopped"
}

cmd_logs() {
    if [[ ! -f "$WORKER_LOG_FILE" ]]; then
        log_warn "Worker log file not found: $WORKER_LOG_FILE"
        exit 1
    fi
    tail -f "$WORKER_LOG_FILE"
}

case "${CMD}" in
    start) cmd_start ;;
    stop) cmd_stop ;;
    logs) cmd_logs ;;
    help|--help|-h) usage ;;
    *) usage; exit 1 ;;
esac
