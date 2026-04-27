#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${RUST_DIR}/docker-compose.test.yml"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker not found in PATH"
  exit 1
fi

if docker compose version >/dev/null 2>&1; then
  COMPOSE_CMD=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE_CMD=(docker-compose)
else
  echo "docker compose command not available"
  exit 1
fi

echo "Starting test services..."
"${COMPOSE_CMD[@]}" -f "${COMPOSE_FILE}" down -v --remove-orphans 2>/dev/null || true
"${COMPOSE_CMD[@]}" -f "${COMPOSE_FILE}" up -d

echo "Waiting for postgres to be healthy..."
until docker compose -f "${COMPOSE_FILE}" ps postgres 2>/dev/null | grep -q "(healthy)"; do
  sleep 0.5
done

echo "Running database migrations..."
MIGRATION_DIR="${RUST_DIR}/migrations"
DATABASE_URL="postgres://postgres:postgres@localhost:5433/wechatbot"

for file in "${MIGRATION_DIR}"/*.sql; do
  if [[ -f "${file}" ]]; then
    echo "  Applying $(basename "${file}")"
    psql "${DATABASE_URL}" -f "${file}" -q
  fi
done

echo "Test services ready."
echo "export WECHATBOT_TEST_DATABASE_URL=${DATABASE_URL}"
