#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR_THIS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_ROOT_DIR="$(cd "${SCRIPT_DIR_THIS}/../.." && pwd)"
source "${SCRIPT_DIR_THIS}/../_common.sh"

bash "${RUST_ROOT_DIR}/scripts/worker.sh" start
