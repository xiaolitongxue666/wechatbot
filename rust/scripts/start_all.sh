#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

log_step "=== WeChatBot Full Startup ==="
require_cmd docker "install Docker Desktop"
require_cmd cargo "install Rust from https://rustup.rs"
require_cmd bun "install Bun from https://bun.sh"
require_cmd curl "needed for health checks"

require_free_port "$ADMIN_PORT"

bash "${SCRIPT_DIR}/services.sh" up
wait_for_pg "$DB_DEV_URL" 60 2
bash "${SCRIPT_DIR}/db.sh" migrate
bash "${SCRIPT_DIR}/db.sh" seed
bash "${SCRIPT_DIR}/dev/start_backend.sh"
bash "${SCRIPT_DIR}/dev/start_worker.sh"
bash "${SCRIPT_DIR}/status.sh"
