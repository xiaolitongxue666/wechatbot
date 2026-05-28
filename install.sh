#!/usr/bin/env bash
# Wrapper — real script lives in tools/scripts/install/
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "${ROOT}/tools/scripts/install/install.sh" "$@"
