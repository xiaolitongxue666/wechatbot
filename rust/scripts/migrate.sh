#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
MIGRATION_DIR="${RUST_DIR}/migrations"
DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/wechatbot}"

if ! command -v psql >/dev/null 2>&1; then
  echo "psql not found in PATH"
  exit 1
fi

if [[ ! -d "${MIGRATION_DIR}" ]]; then
  echo "migration directory not found: ${MIGRATION_DIR}"
  exit 1
fi

for file in "${MIGRATION_DIR}"/*.sql; do
  if [[ -f "${file}" ]]; then
    echo "Applying migration: $(basename "${file}")"
    psql "${DATABASE_URL}" -f "${file}"
  fi
done

echo "All migrations applied."
