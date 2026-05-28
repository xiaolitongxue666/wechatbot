#![allow(dead_code)]

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct BotData {
    pub bot_id: String,
    pub status: String,
    pub last_heartbeat_at: Option<chrono::DateTime<Utc>>,
}

pub struct BotSessionData {
    pub session_id: String,
    pub bot_id: String,
    pub user_id: String,
}

pub struct ChatMessageData {
    pub message_id: String,
    pub event_id: String,
    pub session_id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub content_type: String,
    pub text_content: String,
    pub raw_payload_json: String,
    pub received_at: chrono::DateTime<Utc>,
}

pub struct ForwardEventData {
    pub event_id: String,
    pub session_id: String,
    pub target_service: String,
    pub status: String,
    pub retry_count: i32,
    pub last_error: Option<String>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub struct DlqEntryData {
    pub event_id: String,
    pub session_id: String,
    pub payload_json: String,
    pub error_message: String,
}

pub struct TestFixtures<'a> {
    pool: &'a PgPool,
    bots: Vec<BotData>,
    sessions: Vec<BotSessionData>,
    messages: Vec<ChatMessageData>,
    forward_events: Vec<ForwardEventData>,
    dlq_entries: Vec<DlqEntryData>,
}

pub struct BotBuilder {
    data: BotData,
}

pub struct BotSessionBuilder {
    data: BotSessionData,
}

pub struct ChatMessageBuilder {
    data: ChatMessageData,
}

pub struct ForwardEventBuilder {
    data: ForwardEventData,
}

pub struct DlqEntryBuilder {
    data: DlqEntryData,
}

impl<'a> TestFixtures<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            bots: Vec::new(),
            sessions: Vec::new(),
            messages: Vec::new(),
            forward_events: Vec::new(),
            dlq_entries: Vec::new(),
        }
    }

    pub fn bot(bot_id: &str) -> BotBuilder {
        BotBuilder {
            data: BotData {
                bot_id: bot_id.to_string(),
                status: "offline".to_string(),
                last_heartbeat_at: None,
            },
        }
    }

    pub fn bot_session(session_id: &str, bot_id: &str, user_id: &str) -> BotSessionBuilder {
        BotSessionBuilder {
            data: BotSessionData {
                session_id: session_id.to_string(),
                bot_id: bot_id.to_string(),
                user_id: user_id.to_string(),
            },
        }
    }

    pub fn chat_message(session_id: &str, from_user_id: &str) -> ChatMessageBuilder {
        let message_id = Uuid::new_v4().to_string();
        ChatMessageBuilder {
            data: ChatMessageData {
                message_id,
                event_id: Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                from_user_id: from_user_id.to_string(),
                to_user_id: String::new(),
                content_type: "text".to_string(),
                text_content: String::new(),
                raw_payload_json: "{}".to_string(),
                received_at: Utc::now(),
            },
        }
    }

    pub fn forward_event(event_id: &str, session_id: &str) -> ForwardEventBuilder {
        ForwardEventBuilder {
            data: ForwardEventData {
                event_id: event_id.to_string(),
                session_id: session_id.to_string(),
                target_service: "http://test-webhook/wechat".to_string(),
                status: "retrying".to_string(),
                retry_count: 0,
                last_error: None,
                updated_at: Utc::now(),
            },
        }
    }

    pub fn dlq_entry(event_id: &str, session_id: &str) -> DlqEntryBuilder {
        DlqEntryBuilder {
            data: DlqEntryData {
                event_id: event_id.to_string(),
                session_id: session_id.to_string(),
                payload_json: "{\"test\":true}".to_string(),
                error_message: "test error".to_string(),
            },
        }
    }

    pub fn add_bot(&mut self, b: BotData) -> &mut Self {
        self.bots.push(b);
        self
    }

    pub fn add_bot_session(&mut self, s: BotSessionData) -> &mut Self {
        self.sessions.push(s);
        self
    }

    pub fn add_message(&mut self, m: ChatMessageData) -> &mut Self {
        self.messages.push(m);
        self
    }

    pub fn add_forward_event(&mut self, fe: ForwardEventData) -> &mut Self {
        self.forward_events.push(fe);
        self
    }

    pub fn add_dlq_entry(&mut self, dlq: DlqEntryData) -> &mut Self {
        self.dlq_entries.push(dlq);
        self
    }

    pub async fn apply(self) -> &'a PgPool {
        for b in &self.bots {
            sqlx::query(
                r#"
                INSERT INTO bots (
                    bot_id, status, last_heartbeat_at, created_at, updated_at
                ) VALUES ($1,$2,$3,NOW(),NOW())
                ON CONFLICT (bot_id) DO NOTHING
                "#,
            )
            .bind(&b.bot_id)
            .bind(&b.status)
            .bind(b.last_heartbeat_at)
            .execute(self.pool)
            .await
            .expect("failed to insert bot fixture");
        }

        for s in &self.sessions {
            sqlx::query(
                r#"
                INSERT INTO bot_sessions (
                    session_id, bot_id, user_id, status, created_at, updated_at
                ) VALUES ($1,$2,$3,'active',NOW(),NOW())
                ON CONFLICT (session_id) DO NOTHING
                "#,
            )
            .bind(&s.session_id)
            .bind(&s.bot_id)
            .bind(&s.user_id)
            .execute(self.pool)
            .await
            .expect("failed to insert bot session fixture");
        }

        for m in &self.messages {
            sqlx::query(
                r#"
                INSERT INTO chat_messages (
                    message_id, event_id, session_id, from_user_id, to_user_id,
                    content_type, text_content, raw_payload_json, received_at
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8::jsonb,$9)
                ON CONFLICT (message_id) DO NOTHING
                "#,
            )
            .bind(&m.message_id)
            .bind(&m.event_id)
            .bind(&m.session_id)
            .bind(&m.from_user_id)
            .bind(&m.to_user_id)
            .bind(&m.content_type)
            .bind(&m.text_content)
            .bind(&m.raw_payload_json)
            .bind(m.received_at)
            .execute(self.pool)
            .await
            .expect("failed to insert chat message fixture");
        }

        for fe in &self.forward_events {
            sqlx::query(
                r#"
                INSERT INTO forward_events (
                    event_id, session_id, target_service, status, retry_count, last_error, updated_at
                ) VALUES ($1,$2,$3,$4,$5,$6,$7)
                ON CONFLICT (event_id) DO NOTHING
                "#,
            )
            .bind(&fe.event_id)
            .bind(&fe.session_id)
            .bind(&fe.target_service)
            .bind(&fe.status)
            .bind(fe.retry_count)
            .bind(&fe.last_error)
            .bind(fe.updated_at)
            .execute(self.pool)
            .await
            .expect("failed to insert forward event fixture");
        }

        for dlq in &self.dlq_entries {
            sqlx::query(
                r#"
                INSERT INTO forward_dlq (event_id, session_id, payload_json, error_message, failed_at)
                VALUES ($1,$2,$3::jsonb,$4,NOW())
                ON CONFLICT (event_id) DO NOTHING
                "#,
            )
            .bind(&dlq.event_id)
            .bind(&dlq.session_id)
            .bind(&dlq.payload_json)
            .bind(&dlq.error_message)
            .execute(self.pool)
            .await
            .expect("failed to insert dlq fixture");
        }

        self.pool
    }
}

impl BotBuilder {
    pub fn finish(self) -> BotData {
        self.data
    }

    pub fn status(mut self, status: &str) -> Self {
        self.data.status = status.to_string();
        self
    }

    pub fn heartbeat_now(mut self) -> Self {
        self.data.last_heartbeat_at = Some(Utc::now());
        self
    }

    pub fn heartbeat_ago_seconds(mut self, seconds: i64) -> Self {
        self.data.last_heartbeat_at = Some(Utc::now() - Duration::seconds(seconds));
        self
    }
}

impl BotSessionBuilder {
    pub fn finish(self) -> BotSessionData {
        self.data
    }
}

impl ChatMessageBuilder {
    pub fn finish(self) -> ChatMessageData {
        self.data
    }

    pub fn text(mut self, text: &str) -> Self {
        self.data.text_content = text.to_string();
        self
    }

    pub fn content_type(mut self, ct: &str) -> Self {
        self.data.content_type = ct.to_string();
        self
    }

    pub fn received_at_now(mut self) -> Self {
        self.data.received_at = Utc::now();
        self
    }

    pub fn received_at_minutes_ago(mut self, minutes: i64) -> Self {
        self.data.received_at = Utc::now() - Duration::minutes(minutes);
        self
    }

    pub fn to_user(mut self, to_user: &str) -> Self {
        self.data.to_user_id = to_user.to_string();
        self
    }
}

impl ForwardEventBuilder {
    pub fn finish(self) -> ForwardEventData {
        self.data
    }

    pub fn status(mut self, status: &str) -> Self {
        self.data.status = status.to_string();
        self
    }

    pub fn retry_count(mut self, count: i32) -> Self {
        self.data.retry_count = count;
        self
    }

    pub fn error(mut self, error: &str) -> Self {
        self.data.last_error = Some(error.to_string());
        self
    }

    pub fn updated_at_minutes_ago(mut self, minutes: i64) -> Self {
        self.data.updated_at = Utc::now() - Duration::minutes(minutes);
        self
    }

    pub fn updated_at_hours_ago(mut self, hours: i64) -> Self {
        self.data.updated_at = Utc::now() - Duration::hours(hours);
        self
    }
}

impl DlqEntryBuilder {
    pub fn finish(self) -> DlqEntryData {
        self.data
    }

    pub fn error(mut self, error: &str) -> Self {
        self.data.error_message = error.to_string();
        self
    }
}

pub async fn seed_medium_dataset(pool: &PgPool) {
    let mut fixtures = TestFixtures::new(pool);

    // 六种典型状态，便于管理台演示
    fixtures.add_bot(
        TestFixtures::bot("bot-001")
            .status("online")
            .heartbeat_ago_seconds(20)
            .finish(),
    );
    fixtures.add_bot(
        TestFixtures::bot("bot-002")
            .status("online")
            .heartbeat_ago_seconds(40)
            .finish(),
    );
    fixtures.add_bot(TestFixtures::bot("bot-003").status("offline").finish());
    fixtures.add_bot(TestFixtures::bot("bot-004").status("pending_qr").finish());
    fixtures.add_bot(
        TestFixtures::bot("bot-005")
            .status("expired")
            .heartbeat_ago_seconds(7_200)
            .finish(),
    );
    fixtures.add_bot(
        TestFixtures::bot("bot-006")
            .status("online")
            .heartbeat_ago_seconds(4_000)
            .finish(),
    );

    fixtures.add_bot_session(TestFixtures::bot_session("sess-001", "bot-001", "wx_alice").finish());
    fixtures.add_bot_session(TestFixtures::bot_session("sess-002", "bot-001", "wx_bob").finish());
    fixtures.add_bot_session(
        TestFixtures::bot_session("sess-003", "bot-002", "wx_charlie").finish(),
    );
    fixtures.add_bot_session(TestFixtures::bot_session("sess-004", "bot-003", "wx_dave").finish());
    fixtures.add_bot_session(TestFixtures::bot_session("sess-005", "bot-005", "wx_eve").finish());
    fixtures.add_bot_session(
        TestFixtures::bot_session("sess-006", "bot-006", "wx_frank").finish(),
    );

    let mut add_msg = |session_id: &str, from: &str, to: &str, content: &str, ct: &str, mins_ago: i64| {
        fixtures.add_message(
            TestFixtures::chat_message(session_id, from)
                .to_user(to)
                .text(content)
                .content_type(ct)
                .received_at_minutes_ago(mins_ago)
                .finish(),
        );
    };

    // bot-001：双会话文本对话（各 6 条 → messages_today = 12）
    for (i, (from, to, text)) in [
        ("wx_alice", "bot-001", "你好"),
        ("bot-001", "wx_alice", "在的，请说"),
        ("wx_alice", "bot-001", "帮我查订单"),
        ("bot-001", "wx_alice", "请提供订单号"),
        ("wx_alice", "bot-001", "ORD-10086"),
        ("bot-001", "wx_alice", "订单已发货"),
    ]
    .into_iter()
    .enumerate()
    {
        add_msg("sess-001", from, to, text, "text", 30 - i as i64);
    }
    for (i, (from, to, text)) in [
        ("wx_bob", "bot-001", "下午开会吗？"),
        ("bot-001", "wx_bob", "三点产品评审"),
        ("wx_bob", "bot-001", "收到"),
        ("bot-001", "wx_bob", "已同步日历"),
        ("wx_bob", "bot-001", "谢谢"),
        ("bot-001", "wx_bob", "不客气"),
    ]
    .into_iter()
    .enumerate()
    {
        add_msg("sess-002", from, to, text, "text", 28 - i as i64);
    }

    // bot-002：多媒体会话
    add_msg("sess-003", "wx_charlie", "bot-002", "这是现场照片", "text", 25);
    add_msg(
        "sess-003",
        "wx_charlie",
        "bot-002",
        "[image] https://example.com/photo.jpg",
        "image",
        24,
    );
    add_msg(
        "sess-003",
        "wx_charlie",
        "bot-002",
        "[voice] 语音 12s",
        "voice",
        23,
    );
    add_msg(
        "sess-003",
        "bot-002",
        "wx_charlie",
        "收到，稍后回复",
        "text",
        22,
    );
    add_msg(
        "sess-003",
        "wx_charlie",
        "bot-002",
        "[video] 会议录屏",
        "video",
        21,
    );
    add_msg(
        "sess-003",
        "bot-002",
        "wx_charlie",
        "已转存",
        "text",
        20,
    );

    // bot-003：离线 bot 的历史会话
    add_msg("sess-004", "wx_dave", "bot-003", "今天天气怎么样", "text", 18);
    add_msg("sess-004", "bot-003", "wx_dave", "今天晴，18~26℃", "text", 17);
    add_msg("sess-004", "wx_dave", "bot-003", "需要带伞吗", "text", 16);
    add_msg("sess-004", "bot-003", "wx_dave", "傍晚有小雨", "text", 15);

    // bot-005：已过期 + Coze 转发链路
    add_msg("sess-005", "wx_eve", "bot-005", "给我列出明天的安排", "text", 12);
    add_msg("sess-005", "bot-005", "coze", "发送请求", "text", 11);
    add_msg(
        "sess-005",
        "coze",
        "bot-005",
        "明后天的安排是：会议、复盘、发布",
        "text",
        10,
    );
    add_msg(
        "sess-005",
        "bot-005",
        "wx_eve",
        "明后天的安排是：会议、复盘、发布",
        "text",
        9,
    );

    // bot-006：心跳超时（DB 仍 online，API 展示离线）
    add_msg("sess-006", "wx_frank", "bot-006", "还在吗？", "text", 8);
    add_msg("sess-006", "bot-006", "wx_frank", "刚才断线了", "text", 7);
    add_msg("sess-006", "wx_frank", "bot-006", "好的", "text", 6);

    fixtures.add_forward_event(
        TestFixtures::forward_event("evt-dlq-001", "sess-001")
            .status("failed")
            .retry_count(5)
            .error("connection timeout")
            .finish(),
    );
    fixtures.add_forward_event(
        TestFixtures::forward_event("evt-success-001", "sess-001")
            .status("success")
            .retry_count(1)
            .finish(),
    );
    fixtures.add_forward_event(
        TestFixtures::forward_event("evt-retrying-001", "sess-002")
            .status("retrying")
            .retry_count(2)
            .error("500 internal server error")
            .finish(),
    );
    fixtures.add_forward_event(
        TestFixtures::forward_event("evt-blocked-001", "sess-005")
            .status("blocked")
            .retry_count(1)
            .error("target blocked by policy")
            .updated_at_minutes_ago(2)
            .finish(),
    );
    fixtures.add_forward_event(
        TestFixtures::forward_event("evt-failed-yesterday-001", "sess-003")
            .status("failed")
            .retry_count(1)
            .error("yesterday failure")
            .updated_at_hours_ago(30)
            .finish(),
    );

    fixtures.add_dlq_entry(
        TestFixtures::dlq_entry("evt-dlq-permanent-001", "sess-001")
            .error("permanent failure after 5 retries")
            .finish(),
    );
    fixtures.add_dlq_entry(
        TestFixtures::dlq_entry("evt-dlq-permanent-002", "sess-002")
            .error("webhook unreachable")
            .finish(),
    );

    fixtures.apply().await;
    seed_admin_rbac(pool).await;
}

pub async fn seed_admin_rbac(pool: &PgPool) {
    sqlx::query(
        r#"
        INSERT INTO bots (bot_id, bot_name, status, created_at, updated_at)
        VALUES
            ('bot-001', 'seed-bot-001', 'online', NOW(), NOW()),
            ('bot-002', 'seed-bot-002', 'offline', NOW(), NOW()),
            ('bot-003', 'seed-bot-003', 'offline', NOW(), NOW())
        ON CONFLICT (bot_id) DO NOTHING
        "#,
    )
    .execute(pool)
    .await
    .expect("failed to insert bot rows for admin scope fixture");

    sqlx::query(
        r#"
        INSERT INTO admin_users (user_id, display_name, role, api_token_hash, is_active, created_at, updated_at)
        VALUES
            ('admin-default', 'Default Admin', 'admin', $1, TRUE, NOW(), NOW()),
            ('viewer-demo', 'Viewer Demo', 'viewer', $2, TRUE, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE
        SET role = EXCLUDED.role,
            api_token_hash = EXCLUDED.api_token_hash,
            is_active = TRUE,
            updated_at = NOW()
        "#,
    )
    .bind("1734d503f6aa6a047c36d113cbad769f719c93784b469b771c4c3e7c63adbefd")
    .bind("d036bd6d01a1cae081d39a2f8dab751dc042de814fd60df31fcb553170950f29")
    .execute(pool)
    .await
    .expect("failed to insert admin users fixture");

    sqlx::query(
        r#"
        INSERT INTO admin_user_bot_scopes (user_id, bot_id)
        VALUES ('viewer-demo', 'bot-001'), ('viewer-demo', 'bot-002')
        ON CONFLICT (user_id, bot_id) DO NOTHING
        "#,
    )
    .execute(pool)
    .await
    .expect("failed to insert admin scope fixture");

    sqlx::query(
        r#"
        INSERT INTO bot_forward_policies (bot_id, forwarding_enabled, allowed_targets, updated_at)
        VALUES
            ('bot-001', TRUE,  ARRAY['webhook']::TEXT[], NOW()),
            ('bot-002', FALSE, ARRAY['webhook']::TEXT[], NOW()),
            ('bot-003', TRUE,  ARRAY['coze']::TEXT[], NOW()),
            ('bot-004', TRUE,  ARRAY['webhook']::TEXT[], NOW()),
            ('bot-005', TRUE,  ARRAY['coze']::TEXT[], NOW()),
            ('bot-006', TRUE,  ARRAY['webhook']::TEXT[], NOW())
        ON CONFLICT (bot_id) DO UPDATE
        SET forwarding_enabled = EXCLUDED.forwarding_enabled,
            allowed_targets = EXCLUDED.allowed_targets,
            updated_at = NOW()
        "#,
    )
    .execute(pool)
    .await
    .expect("failed to insert forward policy fixture");
}
