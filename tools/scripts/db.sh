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
    echo "  seed      Insert sample data (5 bots, 30 msgs, 5 fwd, 2 dlq)"
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

-- 6 个 bots（多数在线，便于管理台演示）
INSERT INTO bots (bot_id, bot_name, status, last_heartbeat_at, created_at, updated_at)
VALUES
  ('bot-001', 'demo-1', 'online',  NOW() - INTERVAL '20 seconds',  NOW(), NOW()),
  ('bot-002', 'demo-2', 'online',  NOW() - INTERVAL '40 seconds',  NOW(), NOW()),
  ('bot-003', 'demo-3', 'online',  NOW() - INTERVAL '15 seconds',  NOW(), NOW()),
  ('bot-004', 'demo-4', 'online',  NOW() - INTERVAL '10 seconds',  NOW(), NOW()),
  ('bot-005', 'demo-5', 'online',  NOW() - INTERVAL '25 seconds',  NOW(), NOW()),
  ('bot-006', 'demo-6', 'offline', NULL,                            NOW(), NOW())
ON CONFLICT (bot_id) DO UPDATE SET
  bot_name = EXCLUDED.bot_name,
  status = EXCLUDED.status,
  last_heartbeat_at = EXCLUDED.last_heartbeat_at,
  updated_at = NOW();

-- bot_sessions（bot-001 双会话；bot-003/005 各 1 会话）
INSERT INTO bot_sessions (session_id, bot_id, user_id, status, created_at, updated_at)
VALUES
  ('sess-001', 'bot-001', 'wx_alice',   'active', NOW(), NOW()),
  ('sess-002', 'bot-001', 'wx_bob',     'active', NOW(), NOW()),
  ('sess-003', 'bot-002', 'wx_charlie', 'active', NOW(), NOW()),
  ('sess-004', 'bot-003', 'wx_dave',    'active', NOW(), NOW()),
  ('sess-005', 'bot-004', 'wx_eve',     'active', NOW(), NOW()),
  ('sess-006', 'bot-005', 'wx_frank',   'active', NOW(), NOW())
ON CONFLICT (session_id) DO UPDATE SET
  bot_id = EXCLUDED.bot_id,
  user_id = EXCLUDED.user_id,
  status = EXCLUDED.status,
  updated_at = NOW();

-- 演示：消息与转发完整链路（sess-005 / bot-004 / coze）
DELETE FROM chat_messages WHERE session_id IN ('sess-004', 'sess-005', 'sess-006');

INSERT INTO chat_messages (
  message_id, event_id, session_id, from_user_id, to_user_id, content_type, text_content, raw_payload_json, received_at
) VALUES
  ('msg-s5-001', 'evt-s5-001', 'sess-005', 'wx_eve',   'bot-004', 'text', '给我列出明天的安排', '{}'::jsonb, NOW() - INTERVAL '3 minutes'),
  ('msg-s5-002', 'evt-s5-002', 'sess-005', 'bot-004',  'coze',    'text', '发送请求',         '{}'::jsonb, NOW() - INTERVAL '2 minutes 50 seconds'),
  ('msg-s5-003', 'evt-s5-003', 'sess-005', 'coze',     'bot-004', 'text', '明后天的安排是：会议、复盘、发布', '{}'::jsonb, NOW() - INTERVAL '2 minutes 40 seconds'),
  ('msg-s5-004', 'evt-s5-004', 'sess-005', 'bot-004',  'wx_eve',  'text', '明后天的安排是：会议、复盘、发布', '{}'::jsonb, NOW() - INTERVAL '2 minutes 30 seconds'),
  ('msg-s4-001', 'evt-s4-001', 'sess-004', 'wx_dave',  'bot-003', 'text', '今天天气怎么样', '{}'::jsonb, NOW() - INTERVAL '8 minutes'),
  ('msg-s4-002', 'evt-s4-002', 'sess-004', 'bot-003',  'wx_dave', 'text', '今天晴，18~26℃', '{}'::jsonb, NOW() - INTERVAL '7 minutes 40 seconds'),
  ('msg-s6-001', 'evt-s6-001', 'sess-006', 'wx_frank', 'bot-005', 'text', '帮我总结这份文档', '{}'::jsonb, NOW() - INTERVAL '5 minutes'),
  ('msg-s6-002', 'evt-s6-002', 'sess-006', 'bot-005',  'coze',    'text', '发送请求',       '{}'::jsonb, NOW() - INTERVAL '4 minutes 50 seconds'),
  ('msg-s6-003', 'evt-s6-003', 'sess-006', 'coze',     'bot-005', 'text', '文档要点：目标、风险、排期', '{}'::jsonb, NOW() - INTERVAL '4 minutes 40 seconds'),
  ('msg-s6-004', 'evt-s6-004', 'sess-006', 'bot-005',  'wx_frank','text', '文档要点：目标、风险、排期', '{}'::jsonb, NOW() - INTERVAL '4 minutes 30 seconds')
ON CONFLICT (message_id) DO UPDATE SET
  session_id = EXCLUDED.session_id,
  from_user_id = EXCLUDED.from_user_id,
  to_user_id = EXCLUDED.to_user_id,
  text_content = EXCLUDED.text_content,
  received_at = EXCLUDED.received_at;

-- 其他会话保留通用样例消息
INSERT INTO chat_messages (message_id, event_id, session_id, from_user_id, to_user_id, content_type, text_content, raw_payload_json, received_at)
SELECT
  'msg-gen-' || lpad(i::text, 3, '0'),
  'evt-gen-' || lpad(i::text, 3, '0'),
  CASE ((i - 1) % 3)
    WHEN 0 THEN 'sess-001'
    WHEN 1 THEN 'sess-002'
    ELSE 'sess-003'
  END,
  CASE (i % 2) WHEN 0 THEN 'wx_alice' ELSE 'bot-001' END,
  CASE (i % 2) WHEN 0 THEN 'bot-001' ELSE 'wx_alice' END,
  'text',
  CASE (i % 4)
    WHEN 0 THEN '你好'
    WHEN 1 THEN '在的，请说'
    WHEN 2 THEN '收到'
    ELSE '好的'
  END,
  '{}'::jsonb,
  NOW() - (i * INTERVAL '90 seconds')
FROM generate_series(1, 12) AS s(i)
ON CONFLICT (message_id) DO NOTHING;

-- 5 条 forward_events（其中 1 条为昨日失败，用于校验“今日失败”过滤）
INSERT INTO forward_events (event_id, session_id, target_service, status, retry_count, last_error, updated_at)
VALUES
  ('evt-dlq-001',              'sess-001', 'http://localhost:8081/webhook/wechat', 'failed',   5, 'connection timeout',           NOW()),
  ('evt-success-001',          'sess-001', 'http://localhost:8081/webhook/wechat', 'success',  1, NULL,                           NOW()),
  ('evt-retrying-001',         'sess-002', 'http://localhost:8081/webhook/wechat', 'retrying', 2, '500 internal server error',   NOW()),
  ('evt-blocked-001',          'sess-005', 'http://localhost:8081/webhook/wechat', 'blocked',  1, 'target blocked by policy',    NOW()),
  ('evt-failed-yesterday-001', 'sess-003', 'http://localhost:8081/webhook/wechat', 'failed',   1, 'yesterday failure',            NOW() - INTERVAL '30 hours')
ON CONFLICT (event_id) DO NOTHING;

-- 2 条 forward_dlq (永久失败)
INSERT INTO forward_dlq (event_id, session_id, payload_json, error_message, failed_at)
VALUES
  ('evt-dlq-permanent-001', 'sess-001', '{"type":"text","text":"hello"}'::jsonb, 'permanent failure after 5 retries', NOW()),
  ('evt-dlq-permanent-002', 'sess-002', '{"type":"image","url":"x"}'::jsonb,    'webhook unreachable',               NOW())
ON CONFLICT (event_id) DO NOTHING;

-- admin users + scopes + forwarding policies
INSERT INTO admin_users (user_id, display_name, role, api_token_hash, is_active, created_at, updated_at)
VALUES
  ('admin-default', 'Default Admin', 'admin', '1734d503f6aa6a047c36d113cbad769f719c93784b469b771c4c3e7c63adbefd', TRUE, NOW(), NOW()),
  ('viewer-demo',   'Viewer Demo',   'viewer', 'd036bd6d01a1cae081d39a2f8dab751dc042de814fd60df31fcb553170950f29', TRUE, NOW(), NOW())
ON CONFLICT (user_id) DO UPDATE SET
  role = EXCLUDED.role,
  api_token_hash = EXCLUDED.api_token_hash,
  is_active = TRUE,
  updated_at = NOW();

INSERT INTO admin_user_bot_scopes (user_id, bot_id)
VALUES
  ('viewer-demo', 'bot-001'),
  ('viewer-demo', 'bot-002')
ON CONFLICT (user_id, bot_id) DO NOTHING;

INSERT INTO bot_forward_policies (bot_id, forwarding_enabled, allowed_targets, updated_at)
VALUES
  ('bot-001', TRUE,  ARRAY['webhook']::TEXT[], NOW()),
  ('bot-002', FALSE, ARRAY['webhook']::TEXT[], NOW()),
  ('bot-003', TRUE,  ARRAY['coze']::TEXT[], NOW()),
  ('bot-004', TRUE,  ARRAY['coze']::TEXT[], NOW()),
  ('bot-005', TRUE,  ARRAY['coze']::TEXT[], NOW())
ON CONFLICT (bot_id) DO UPDATE SET
  forwarding_enabled = EXCLUDED.forwarding_enabled,
  allowed_targets = EXCLUDED.allowed_targets,
  updated_at = NOW();

SQL

    cat > "${RUST_DIR}/.worker.log" <<'EOF'
2026-05-28T10:00:00.100000Z INFO wechatbot::infra::logging: tracing initialized service="forwarder_worker"
2026-05-28T10:00:01.200000Z INFO wechatbot::forwarder: forward event consumed event_id=evt-s5-002 session_id=sess-005 bot_id=bot-004
2026-05-28T10:00:01.800000Z INFO wechatbot::forwarder: forward endpoint returned 200 bot_id=bot-004 session_id=sess-005
2026-05-28T10:00:02.300000Z INFO wechatbot::forwarder: forward event consumed event_id=evt-s6-002 session_id=sess-006 bot_id=bot-005
2026-05-28T10:00:02.900000Z INFO wechatbot::forwarder: forward endpoint returned 200 bot_id=bot-005 session_id=sess-006
EOF

    log_ok "Seed data inserted (bots/messages/forwards/admin users/policies)"
}

# ── clear ──────────────────────────────────────────────────────────────────────
cmd_clear() {
    log_step "Clearing all table data..."
    psql_exec "$DB_URL" "TRUNCATE TABLE admin_user_bot_scopes, admin_users, bot_forward_policies, forward_dlq, forward_events, chat_media, chat_messages, bot_sessions, bots CASCADE;"
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

    for table in bots bot_sessions chat_messages chat_media forward_events forward_dlq admin_users admin_user_bot_scopes bot_forward_policies; do
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
