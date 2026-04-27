//! Test fixture builder for seeding mock data into the database.
//! Supports the "medium" data profile: 5 bots + 30 messages.

#![allow(dead_code)]

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct BotSessionData {
    pub session_id: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub wx_user_id: Option<String>,
    pub status: String,
    pub last_heartbeat_at: Option<chrono::DateTime<Utc>>,
}

pub struct ChatMessageData {
    pub message_id: String,
    pub event_id: String,
    pub session_id: String,
    pub tenant_id: String,
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
}

pub struct DlqEntryData {
    pub event_id: String,
    pub session_id: String,
    pub payload_json: String,
    pub error_message: String,
}

pub struct TestFixtures<'a> {
    pool: &'a PgPool,
    sessions: Vec<BotSessionData>,
    messages: Vec<ChatMessageData>,
    forward_events: Vec<ForwardEventData>,
    dlq_entries: Vec<DlqEntryData>,
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
            sessions: Vec::new(),
            messages: Vec::new(),
            forward_events: Vec::new(),
            dlq_entries: Vec::new(),
        }
    }

    pub fn bot_session(session_id: &str, tenant_id: &str, owner_id: &str) -> BotSessionBuilder {
        BotSessionBuilder {
            data: BotSessionData {
                session_id: session_id.to_string(),
                tenant_id: tenant_id.to_string(),
                owner_id: owner_id.to_string(),
                wx_user_id: None,
                status: "offline".to_string(),
                last_heartbeat_at: None,
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
                tenant_id: "tenant-a".to_string(),
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
        for s in &self.sessions {
            sqlx::query(
                r#"
                INSERT INTO bot_sessions (
                    session_id, tenant_id, owner_id, wx_user_id, status, token_ref,
                    last_heartbeat_at, created_at, updated_at
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,NOW(),NOW())
                ON CONFLICT (session_id) DO NOTHING
                "#,
            )
            .bind(&s.session_id)
            .bind(&s.tenant_id)
            .bind(&s.owner_id)
            .bind(&s.wx_user_id)
            .bind(&s.status)
            .bind::<Option<String>>(None)
            .bind(s.last_heartbeat_at)
            .execute(self.pool)
            .await
            .expect("failed to insert bot session fixture");
        }

        for m in &self.messages {
            sqlx::query(
                r#"
                INSERT INTO chat_messages (
                    message_id, event_id, session_id, tenant_id, from_user_id, to_user_id,
                    content_type, text_content, raw_payload_json, received_at
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::jsonb,$10)
                ON CONFLICT (message_id) DO NOTHING
                "#,
            )
            .bind(&m.message_id)
            .bind(&m.event_id)
            .bind(&m.session_id)
            .bind(&m.tenant_id)
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
                ) VALUES ($1,$2,$3,$4,$5,$6,NOW())
                ON CONFLICT (event_id) DO NOTHING
                "#,
            )
            .bind(&fe.event_id)
            .bind(&fe.session_id)
            .bind(&fe.target_service)
            .bind(&fe.status)
            .bind(fe.retry_count)
            .bind(&fe.last_error)
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

impl BotSessionBuilder {
    pub fn finish(self) -> BotSessionData {
        self.data
    }

    pub fn status(mut self, status: &str) -> Self {
        self.data.status = status.to_string();
        self
    }

    pub fn wx_user_id(mut self, wx_user_id: &str) -> Self {
        self.data.wx_user_id = Some(wx_user_id.to_string());
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

    pub fn tenant(mut self, tenant: &str) -> Self {
        self.data.tenant_id = tenant.to_string();
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

/// Seeds a "medium" dataset: 5 bots + 30 messages + forward events + DLQ entries.
pub async fn seed_medium_dataset(pool: &PgPool) {
    let mut fixtures = TestFixtures::new(pool);

    let s1 = TestFixtures::bot_session("session-001", "tenant-a", "owner-alice")
        .status("online")
        .wx_user_id("wx_alice")
        .heartbeat_ago_seconds(30)
        .finish();
    let s2 = TestFixtures::bot_session("session-002", "tenant-a", "owner-alice")
        .status("online")
        .wx_user_id("wx_bob")
        .heartbeat_ago_seconds(60)
        .finish();
    let s3 = TestFixtures::bot_session("session-003", "tenant-b", "owner-charlie")
        .status("offline")
        .finish();
    let s4 = TestFixtures::bot_session("session-004", "tenant-b", "owner-dave")
        .status("online")
        .wx_user_id("wx_dave")
        .heartbeat_now()
        .finish();
    let s5 = TestFixtures::bot_session("session-005", "tenant-a", "owner-eve")
        .status("expired")
        .heartbeat_ago_seconds(400)
        .finish();

    fixtures.add_bot_session(s1);
    fixtures.add_bot_session(s2);
    fixtures.add_bot_session(s3);
    fixtures.add_bot_session(s4);
    fixtures.add_bot_session(s5);

    let users = ["user_alice", "user_bob", "user_charlie", "user_dave", "user_eve"];
    let contents = [
        ("hello world", "text"),
        ("How are you?", "text"),
        ("Check this image", "image"),
        ("Voice message", "voice"),
        ("Video call later?", "video"),
        ("OK", "text"),
    ];

    for i in 0..30 {
        let session_id = format!("session-{:03}", (i % 5) + 1);
        let user = users[i % users.len()];
        let (content, ct) = contents[i % contents.len()];
        let m = TestFixtures::chat_message(&session_id, user)
            .text(content)
            .content_type(ct)
            .tenant("tenant-a")
            .received_at_minutes_ago((30 - i as i64).max(1))
            .finish();
        fixtures.add_message(m);
    }

    let fe1 = TestFixtures::forward_event("evt-dlq-001", "session-001")
        .status("failed")
        .retry_count(5)
        .error("connection timeout")
        .finish();
    let fe2 = TestFixtures::forward_event("evt-success-001", "session-001")
        .status("success")
        .retry_count(1)
        .finish();
    let fe3 = TestFixtures::forward_event("evt-retrying-001", "session-002")
        .status("retrying")
        .retry_count(2)
        .error("500 internal server error")
        .finish();

    fixtures.add_forward_event(fe1);
    fixtures.add_forward_event(fe2);
    fixtures.add_forward_event(fe3);

    let dlq1 = TestFixtures::dlq_entry("evt-dlq-permanent-001", "session-001")
        .error("permanent failure after 5 retries")
        .finish();
    let dlq2 = TestFixtures::dlq_entry("evt-dlq-permanent-002", "session-002")
        .error("webhook unreachable")
        .finish();

    fixtures.add_dlq_entry(dlq1);
    fixtures.add_dlq_entry(dlq2);

    fixtures.apply().await;
}
