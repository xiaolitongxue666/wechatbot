#!/usr/bin/env bash
# ==============================================================================
# 数据库管理：迁移 / 灌种 / 清空 / 重置 / 状态查询
#
# Usage: db.sh {migrate|seed|clear|reset|status}
# Env:   DATABASE_URL  (default: dev Postgres)
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_common.sh"

CMD="${1:-}"
# DATABASE_URL 环境变量优先，否则用 dev 默认值
DB_URL="${DATABASE_URL:-$DB_DEV_URL}"

# 显示用法
usage() {
    echo "Usage: $(basename "$0") {migrate|seed|clear|reset|status}"
    echo ""
    echo "  migrate   Apply SQL migrations to create tables"
    echo "  seed      Insert sample data (5 bots, 30 msgs, 3 fwd, 2 dlq)"
    echo "  clear     Truncate all tables (keep schema)"
    echo "  reset     Clear + migrate (reset to fresh schema)"
    echo "  status    Show row counts for each table"
    echo ""
    echo "Env: DATABASE_URL (default: $DB_DEV_URL)"
    exit 0
}

# ── migrate ────────────────────────────────────────────────────────────────────
cmd_migrate() {
    log_step "Running database migrations..."
    if [[ ! -d "$MIGRATION_DIR" ]]; then
        log_err "Migration directory not found: $MIGRATION_DIR"
        exit 1
    fi

    for file in "${MIGRATION_DIR}"/*.sql; do
        if [[ -f "$file" ]]; then
            log_info "Applying $(basename "$file")"
            psql_exec_file "$DB_URL" "$file"
        fi
    done
    log_ok "All migrations applied"
}

# ── seed ───────────────────────────────────────────────────────────────────────
# 种子数据与 tests/common/fixtures.rs 中 seed_medium_dataset() 一致
cmd_seed() {
    log_step "Seeding development data..."

    cat <<'SQL' | psql_exec_file "$DB_URL" /dev/stdin

-- 5 个 bot_sessions
INSERT INTO bot_sessions (session_id, tenant_id, owner_id, wx_user_id, status, last_heartbeat_at, created_at, updated_at)
VALUES
  ('session-001', 'tenant-a', 'owner-alice',  'wx_alice',   'online',  NOW() - INTERVAL '30 seconds',  NOW(), NOW()),
  ('session-002', 'tenant-a', 'owner-alice',  'wx_bob',     'online',  NOW() - INTERVAL '60 seconds',  NOW(), NOW()),
  ('session-003', 'tenant-b', 'owner-charlie','wx_charlie', 'offline', NULL,                            NOW(), NOW()),
  ('session-004', 'tenant-b', 'owner-dave',   'wx_dave',    'online',  NOW(),                            NOW(), NOW()),
  ('session-005', 'tenant-a', 'owner-eve',    NULL,         'expired', NOW() - INTERVAL '400 seconds',  NOW(), NOW())
ON CONFLICT (session_id) DO NOTHING;

-- 30 条 chat_messages，平均分配到 5 个 session
INSERT INTO chat_messages (message_id, event_id, session_id, tenant_id, from_user_id, to_user_id, content_type, text_content, raw_payload_json, received_at)
SELECT
  'msg-' || lpad(i::text, 3, '0'),
  'evt-' || lpad(i::text, 3, '0'),
  'session-' || lpad(((i - 1) % 5 + 1)::text, 3, '0'),
  'tenant-a',
  CASE (i % 6)
    WHEN 0 THEN 'user_alice'
    WHEN 1 THEN 'user_bob'
    WHEN 2 THEN 'user_charlie'
    WHEN 3 THEN 'user_dave'
    WHEN 4 THEN 'user_eve'
    WHEN 5 THEN 'user_frank'
  END,
  'bot-user',
  CASE (i % 6)
    WHEN 0 THEN 'text'  WHEN 1 THEN 'text'  WHEN 2 THEN 'image'
    WHEN 3 THEN 'voice' WHEN 4 THEN 'video' WHEN 5 THEN 'text'
  END,
  CASE (i % 6)
    WHEN 0 THEN 'hello world'
    WHEN 1 THEN 'How are you?'
    WHEN 2 THEN 'Check this image'
    WHEN 3 THEN 'Voice message'
    WHEN 4 THEN 'Video call later?'
    WHEN 5 THEN 'OK'
  END,
  '{}'::jsonb,
  NOW() - (i * INTERVAL '1 minute')
FROM generate_series(1, 30) AS s(i)
ON CONFLICT (message_id) DO NOTHING;

-- 3 条 forward_events (1 success + 1 failed + 1 retrying)
INSERT INTO forward_events (event_id, session_id, target_service, status, retry_count, last_error, updated_at)
VALUES
  ('evt-dlq-001',      'session-001', 'http://localhost:8081/webhook/wechat', 'failed',    5, 'connection timeout',           NOW()),
  ('evt-success-001',  'session-001', 'http://localhost:8081/webhook/wechat', 'success',   1, NULL,                           NOW()),
  ('evt-retrying-001', 'session-002', 'http://localhost:8081/webhook/wechat', 'retrying',  2, '500 internal server error',   NOW())
ON CONFLICT (event_id) DO NOTHING;

-- 2 条 forward_dlq (永久失败)
INSERT INTO forward_dlq (event_id, session_id, payload_json, error_message, failed_at)
VALUES
  ('evt-dlq-permanent-001', 'session-001', '{"type":"text","text":"hello"}'::jsonb, 'permanent failure after 5 retries', NOW()),
  ('evt-dlq-permanent-002', 'session-002', '{"type":"image","url":"x"}'::jsonb,    'webhook unreachable',               NOW())
ON CONFLICT (event_id) DO NOTHING;

SQL

    log_ok "Seed data inserted (5 bots, 30 messages, 3 forward events, 2 DLQ entries)"
}

# ── clear ──────────────────────────────────────────────────────────────────────
cmd_clear() {
    log_step "Clearing all table data..."
    psql_exec "$DB_URL" "TRUNCATE TABLE forward_dlq, forward_events, chat_media, chat_messages, bot_sessions CASCADE;"
    log_ok "All tables cleared"
}

# ── reset ──────────────────────────────────────────────────────────────────────
cmd_reset() {
    log_warn "This will DROP all data and recreate the schema."
    echo -n "Are you sure? [y/N] "
    read -r confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        log_info "Cancelled."
        exit 0
    fi
    cmd_clear
    cmd_migrate
    log_ok "Database reset complete"
}

# ── status ─────────────────────────────────────────────────────────────────────
cmd_status() {
    echo "--- Database status (${DB_URL}) ---"
    local connected=false
    if psql_exec_select "$DB_URL" "SELECT 1" &>/dev/null; then
        connected=true
        echo "  connected: yes"
    else
        echo "  connected: no"
        return 1
    fi

    for table in bot_sessions chat_messages chat_media forward_events forward_dlq; do
        local count
        count=$(psql_exec_select "$DB_URL" "SELECT count(*) FROM ${table}" 2>/dev/null || echo "?")
        # 对齐输出
        printf "  %-18s %s\n" "${table}:" "$count"
    done
}

# ── 入口 ──────────────────────────────────────────────────────────────────────
case "${CMD}" in
    migrate)  cmd_migrate ;;
    seed)     cmd_seed ;;
    clear)    cmd_clear ;;
    reset)    cmd_reset ;;
    status)   cmd_status ;;
    help|--help|-h) usage ;;
    *)
        echo "Unknown command: ${CMD}"
        usage
        exit 1
        ;;
esac
