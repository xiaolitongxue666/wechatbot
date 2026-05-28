#!/usr/bin/env bash
# 后端门禁：cargo test --lib → test_all.sh（Docker 集成）
# 注意：admin_frontend 集成测试依赖 admin/web/dist，请先 npm run build
set -euo pipefail

SCRIPT_DIR_THIS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# tools/scripts/test → 上三级 = 仓库根（勿改成 ../..，会落在 tools/）
PROJECT_ROOT="$(cd "${SCRIPT_DIR_THIS}/../../.." && pwd)"
source "${PROJECT_ROOT}/tools/scripts/_common.sh"

log_step "Running backend tests"
cd "$PROJECT_ROOT"
cargo test --lib
bash "${PROJECT_ROOT}/tools/scripts/test_all.sh"
