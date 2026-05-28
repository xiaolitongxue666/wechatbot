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
    echo "  seed      Insert sample data (6 bot states, sessions, messages, forwards)"
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
    log_step "Seeding development mock data..."
    log_warn "DEV/TEST ONLY — do not run seed on production deploy databases"

    cat <<'SQL' | psql_exec_file "$DB_URL" /dev/stdin

-- 6 种典型 Bot 状态（在线 / 离线 / 待扫码 / 已过期 / 心跳超时）
INSERT INTO bots (bot_id, bot_name, status, last_heartbeat_at, created_at, updated_at)
VALUES
  ('bot-001', 'demo-online-1',   'online',     NOW() - INTERVAL '20 seconds',  NOW(), NOW()),
  ('bot-002', 'demo-online-2',   'online',     NOW() - INTERVAL '40 seconds',  NOW(), NOW()),
  ('bot-003', 'demo-offline',    'offline',    NULL,                             NOW(), NOW()),
  ('bot-004', 'demo-pending-qr', 'pending_qr', NULL,                             NOW(), NOW()),
  ('bot-005', 'demo-expired',    'expired',    NOW() - INTERVAL '2 hours',       NOW(), NOW()),
  ('bot-006', 'demo-stale-hb',   'online',     NOW() - INTERVAL '4000 seconds',  NOW(), NOW())
ON CONFLICT (bot_id) DO UPDATE SET
  bot_name = EXCLUDED.bot_name,
  status = EXCLUDED.status,
  last_heartbeat_at = EXCLUDED.last_heartbeat_at,
  updated_at = NOW();

-- bot-004 待扫码：无会话；其余 bot 各 1~2 个会话
DELETE FROM bot_sessions WHERE bot_id IN ('bot-001','bot-002','bot-003','bot-004','bot-005','bot-006');

INSERT INTO bot_sessions (session_id, bot_id, user_id, status, created_at, updated_at)
VALUES
  ('sess-001', 'bot-001', 'wx_alice',   'active', NOW(), NOW()),
  ('sess-002', 'bot-001', 'wx_bob',     'active', NOW(), NOW()),
  ('sess-003', 'bot-002', 'wx_charlie', 'active', NOW(), NOW()),
  ('sess-004', 'bot-003', 'wx_dave',    'active', NOW(), NOW()),
  ('sess-005', 'bot-005', 'wx_eve',     'active', NOW(), NOW()),
  ('sess-006', 'bot-006', 'wx_frank',   'active', NOW(), NOW())
ON CONFLICT (session_id) DO UPDATE SET
  bot_id = EXCLUDED.bot_id,
  user_id = EXCLUDED.user_id,
  status = EXCLUDED.status,
  updated_at = NOW();

DELETE FROM chat_messages WHERE session_id IN ('sess-001','sess-002','sess-003','sess-004','sess-005','sess-006');

-- bot-001：双会话文本
INSERT INTO chat_messages (message_id, event_id, session_id, from_user_id, to_user_id, content_type, text_content, raw_payload_json, received_at) VALUES
  ('msg-001-01', 'evt-001-01', 'sess-001', 'wx_alice', 'bot-001', 'text', '你好',               '{}'::jsonb, NOW() - INTERVAL '30 minutes'),
  ('msg-001-02', 'evt-001-02', 'sess-001', 'bot-001',  'wx_alice','text', '在的，请说',         '{}'::jsonb, NOW() - INTERVAL '29 minutes'),
  ('msg-001-03', 'evt-001-03', 'sess-001', 'wx_alice', 'bot-001', 'text', '帮我查订单',         '{}'::jsonb, NOW() - INTERVAL '28 minutes'),
  ('msg-001-04', 'evt-001-04', 'sess-001', 'bot-001',  'wx_alice','text', '请提供订单号',       '{}'::jsonb, NOW() - INTERVAL '27 minutes'),
  ('msg-001-05', 'evt-001-05', 'sess-001', 'wx_alice', 'bot-001', 'text', 'ORD-10086',          '{}'::jsonb, NOW() - INTERVAL '26 minutes'),
  ('msg-001-06', 'evt-001-06', 'sess-001', 'bot-001',  'wx_alice','text', '订单已发货',         '{}'::jsonb, NOW() - INTERVAL '25 minutes'),
  ('msg-002-01', 'evt-002-01', 'sess-002', 'wx_bob',   'bot-001', 'text', '下午开会吗？',       '{}'::jsonb, NOW() - INTERVAL '24 minutes'),
  ('msg-002-02', 'evt-002-02', 'sess-002', 'bot-001',  'wx_bob',  'text', '三点产品评审',       '{}'::jsonb, NOW() - INTERVAL '23 minutes'),
  ('msg-002-03', 'evt-002-03', 'sess-002', 'wx_bob',   'bot-001', 'text', '收到',               '{}'::jsonb, NOW() - INTERVAL '22 minutes'),
  ('msg-002-04', 'evt-002-04', 'sess-002', 'bot-001',  'wx_bob',  'text', '已同步日历',         '{}'::jsonb, NOW() - INTERVAL '21 minutes'),
  ('msg-002-05', 'evt-002-05', 'sess-002', 'wx_bob',   'bot-001', 'text', '谢谢',               '{}'::jsonb, NOW() - INTERVAL '20 minutes'),
  ('msg-002-06', 'evt-002-06', 'sess-002', 'bot-001',  'wx_bob',  'text', '不客气',             '{}'::jsonb, NOW() - INTERVAL '19 minutes');

-- bot-002：多媒体
INSERT INTO chat_messages (message_id, event_id, session_id, from_user_id, to_user_id, content_type, text_content, raw_payload_json, received_at) VALUES
  ('msg-003-01', 'evt-003-01', 'sess-003', 'wx_charlie', 'bot-002', 'text',  '这是现场照片',                          '{}'::jsonb, NOW() - INTERVAL '18 minutes'),
  ('msg-003-02', 'evt-003-02', 'sess-003', 'wx_charlie', 'bot-002', 'image', '[image] https://example.com/photo.jpg','{}'::jsonb, NOW() - INTERVAL '17 minutes'),
  ('msg-003-03', 'evt-003-03', 'sess-003', 'wx_charlie', 'bot-002', 'voice', '[voice] 语音 12s',                      '{}'::jsonb, NOW() - INTERVAL '16 minutes'),
  ('msg-003-04', 'evt-003-04', 'sess-003', 'bot-002',    'wx_charlie','text','收到，稍后回复',                        '{}'::jsonb, NOW() - INTERVAL '15 minutes'),
  ('msg-003-05', 'evt-003-05', 'sess-003', 'wx_charlie', 'bot-002', 'video', '[video] 会议录屏',                       '{}'::jsonb, NOW() - INTERVAL '14 minutes'),
  ('msg-003-06', 'evt-003-06', 'sess-003', 'bot-002',    'wx_charlie','text','已转存',                                '{}'::jsonb, NOW() - INTERVAL '13 minutes');

-- bot-003：离线历史
INSERT INTO chat_messages (message_id, event_id, session_id, from_user_id, to_user_id, content_type, text_content, raw_payload_json, received_at) VALUES
  ('msg-004-01', 'evt-004-01', 'sess-004', 'wx_dave', 'bot-003', 'text', '今天天气怎么样', '{}'::jsonb, NOW() - INTERVAL '12 minutes'),
  ('msg-004-02', 'evt-004-02', 'sess-004', 'bot-003', 'wx_dave', 'text', '今天晴，18~26℃', '{}'::jsonb, NOW() - INTERVAL '11 minutes'),
  ('msg-004-03', 'evt-004-03', 'sess-004', 'wx_dave', 'bot-003', 'text', '需要带伞吗',     '{}'::jsonb, NOW() - INTERVAL '10 minutes'),
  ('msg-004-04', 'evt-004-04', 'sess-004', 'bot-003', 'wx_dave', 'text', '傍晚有小雨',     '{}'::jsonb, NOW() - INTERVAL '9 minutes');

-- bot-005：已过期 + Coze 转发链路
INSERT INTO chat_messages (message_id, event_id, session_id, from_user_id, to_user_id, content_type, text_content, raw_payload_json, received_at) VALUES
  ('msg-005-01', 'evt-005-01', 'sess-005', 'wx_eve',   'bot-005', 'text', '给我列出明天的安排',                     '{}'::jsonb, NOW() - INTERVAL '8 minutes'),
  ('msg-005-02', 'evt-005-02', 'sess-005', 'bot-005',  'coze',    'text', '发送请求',                               '{}'::jsonb, NOW() - INTERVAL '7 minutes 50 seconds'),
  ('msg-005-03', 'evt-005-03', 'sess-005', 'coze',     'bot-005', 'text', '明后天的安排是：会议、复盘、发布',       '{}'::jsonb, NOW() - INTERVAL '7 minutes 40 seconds'),
  ('msg-005-04', 'evt-005-04', 'sess-005', 'bot-005',  'wx_eve',  'text', '明后天的安排是：会议、复盘、发布',       '{}'::jsonb, NOW() - INTERVAL '7 minutes 30 seconds');

-- bot-006：心跳超时演示
INSERT INTO chat_messages (message_id, event_id, session_id, from_user_id, to_user_id, content_type, text_content, raw_payload_json, received_at) VALUES
  ('msg-006-01', 'evt-006-01', 'sess-006', 'wx_frank', 'bot-006', 'text', '还在吗？',       '{}'::jsonb, NOW() - INTERVAL '6 minutes'),
  ('msg-006-02', 'evt-006-02', 'sess-006', 'bot-006',  'wx_frank','text', '刚才断线了',     '{}'::jsonb, NOW() - INTERVAL '5 minutes'),
  ('msg-006-03', 'evt-006-03', 'sess-006', 'wx_frank', 'bot-006', 'text', '好的',           '{}'::jsonb, NOW() - INTERVAL '4 minutes');

DELETE FROM forward_events WHERE event_id IN (
  'evt-dlq-001','evt-success-001','evt-retrying-001','evt-blocked-001','evt-failed-yesterday-001'
);

INSERT INTO forward_events (event_id, session_id, target_service, status, retry_count, last_error, updated_at)
VALUES
  ('evt-dlq-001',              'sess-001', 'http://localhost:8081/webhook/wechat', 'failed',   5, 'connection timeout',           NOW()),
  ('evt-success-001',          'sess-001', 'http://localhost:8081/webhook/wechat', 'success',  1, NULL,                           NOW()),
  ('evt-retrying-001',         'sess-002', 'http://localhost:8081/webhook/wechat', 'retrying', 2, '500 internal server error',   NOW()),
  ('evt-blocked-001',          'sess-005', 'http://localhost:8081/webhook/wechat', 'blocked',  1, 'target blocked by policy',    NOW()),
  ('evt-failed-yesterday-001', 'sess-003', 'http://localhost:8081/webhook/wechat', 'failed',   1, 'yesterday failure',            NOW() - INTERVAL '30 hours')
ON CONFLICT (event_id) DO UPDATE SET
  session_id = EXCLUDED.session_id,
  status = EXCLUDED.status,
  retry_count = EXCLUDED.retry_count,
  last_error = EXCLUDED.last_error,
  updated_at = EXCLUDED.updated_at;

DELETE FROM forward_dlq WHERE event_id IN ('evt-dlq-permanent-001','evt-dlq-permanent-002');

INSERT INTO forward_dlq (event_id, session_id, payload_json, error_message, failed_at)
VALUES
  ('evt-dlq-permanent-001', 'sess-001', '{"type":"text","text":"hello"}'::jsonb, 'permanent failure after 5 retries', NOW()),
  ('evt-dlq-permanent-002', 'sess-002', '{"type":"image","url":"x"}'::jsonb,    'webhook unreachable',               NOW())
ON CONFLICT (event_id) DO NOTHING;

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
  ('bot-004', TRUE,  ARRAY['webhook']::TEXT[], NOW()),
  ('bot-005', TRUE,  ARRAY['coze']::TEXT[], NOW()),
  ('bot-006', TRUE,  ARRAY['webhook']::TEXT[], NOW())
ON CONFLICT (bot_id) DO UPDATE SET
  forwarding_enabled = EXCLUDED.forwarding_enabled,
  allowed_targets = EXCLUDED.allowed_targets,
  updated_at = NOW();

SQL

    cat > "${RUST_DIR}/.worker.log" <<'EOF'
2026-05-28T10:00:00.100000Z INFO wechatbot::infra::logging: tracing initialized service="forwarder_worker"
2026-05-28T10:00:01.200000Z INFO wechatbot::forwarder: forward event consumed event_id=evt-005-02 session_id=sess-005 bot_id=bot-005
2026-05-28T10:00:01.800000Z INFO wechatbot::forwarder: forward endpoint returned 200 bot_id=bot-005 session_id=sess-005
2026-05-28T10:00:02.300000Z INFO wechatbot::forwarder: forward event consumed event_id=evt-dlq-001 session_id=sess-001 bot_id=bot-001
2026-05-28T10:00:02.900000Z WARN wechatbot::forwarder: forward failed after retries bot_id=bot-001 session_id=sess-001
EOF

    log_ok "Seed data inserted (6 bot states, sessions, messages, forwards)"
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
