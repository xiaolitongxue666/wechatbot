#!/usr/bin/env bash
# ==============================================================================
# 全栈关闭：worker → admin → 容器（start_all.sh 的逆序）
#
# Usage: stop_all.sh [-v|--volumes]
#   -v, --volumes   Stop containers and remove Docker volumes
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

REMOVE_VOLUMES=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -v|--volumes) REMOVE_VOLUMES=true; shift ;;
        --help|-h)
            echo "Usage: $(basename "$0") [-v|--volumes]"
            echo ""
            echo "  default       Stop worker, admin, and containers (keep volumes)"
            echo "  -v, --volumes Also remove Docker volumes"
            exit 0
            ;;
        *) shift ;;
    esac
done

log_step "=== WeChatBot Full Shutdown ==="

stop_dev_stack "$REMOVE_VOLUMES"

printf "\n"
log_ok "All services stopped"
echo "Start: bash tools/scripts/start_all.sh"
printf "\n"
