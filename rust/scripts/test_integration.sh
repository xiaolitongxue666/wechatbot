#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${RUST_DIR}/docker-compose.test.yml"

echo "============================================"
echo "  WeChatBot Integration Test Suite"
echo "============================================"

cleanup() {
  echo ""
  echo "Cleaning up test services..."
  if docker compose version >/dev/null 2>&1; then
    docker compose -f "${COMPOSE_FILE}" down -v --remove-orphans 2>/dev/null || true
  elif command -v docker-compose >/dev/null 2>&1; then
    docker-compose -f "${COMPOSE_FILE}" down -v --remove-orphans 2>/dev/null || true
  fi
}
trap cleanup EXIT

echo "Step 1/4: Starting test services..."
bash "${SCRIPT_DIR}/test_services.sh"

echo ""
echo "Step 2/4: Building project..."
(cd "${RUST_DIR}" && cargo build)

echo ""
echo "Step 3/4: Running integration tests..."
export WECHATBOT_TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5433/wechatbot"

cd "${RUST_DIR}"
if cargo nextest --version >/dev/null 2>&1; then
  cargo nextest run --profile ci
else
  echo "cargo-nextest not found; install with: cargo install cargo-nextest --locked"
  echo "Falling back to cargo test..."
  cargo test -- --test-threads=2
fi

echo ""
echo "Step 4/4: Tests complete."
