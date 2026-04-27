//! Backend integration tests: repository CRUD operations against Postgres.

mod common;

use std::sync::Mutex;
use uuid::Uuid;
use wechatbot::admin::repository::AdminRepository;
use wechatbot::ingest::EventEnvelope;
use wechatbot::storage::postgres::PostgresChatRepository;
use wechatbot::storage::ChatRepository;
use wechatbot::storage::MediaRecord;
use serde_json::json;

use common::db::setup_test_db;
use common::fixtures::seed_medium_dataset;

static SETUP_MUTEX: Mutex<()> = Mutex::new(());
static mut SEEDED: bool = false;

#[tokio::test]
async fn upsert_bot_insert() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    let repo = PostgresChatRepository::from_pool(pool.clone());
    let bot_id = format!("test-bot-{}", Uuid::new_v4());

    repo.upsert_bot(&bot_id, "pending_qr").await.unwrap();

    let admin = AdminRepository::new(pool);
    let bot = admin.get_bot(&bot_id).await.unwrap().unwrap();
    assert_eq!(bot.bot_id, bot_id);
    assert_eq!(bot.status, "pending_qr");
}

#[tokio::test]
async fn upsert_bot_update() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    let repo = PostgresChatRepository::from_pool(pool.clone());
    let bot_id = format!("test-bot-{}", Uuid::new_v4());

    repo.upsert_bot(&bot_id, "offline").await.unwrap();
    repo.upsert_bot(&bot_id, "online").await.unwrap();

    let admin = AdminRepository::new(pool);
    let bot = admin.get_bot(&bot_id).await.unwrap().unwrap();
    assert_eq!(bot.status, "online");
}

#[tokio::test]
async fn create_session_and_read_back() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    let repo = PostgresChatRepository::from_pool(pool.clone());
    let bot_id = format!("test-bot-{}", Uuid::new_v4());
    let session_id = format!("test-session-{}", Uuid::new_v4());

    repo.upsert_bot(&bot_id, "online").await.unwrap();
    repo.create_session(&session_id, &bot_id, "wx_user_x").await.unwrap();

    let admin = AdminRepository::new(pool);
    let sessions = admin.list_sessions(&bot_id).await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, session_id);
    assert_eq!(sessions[0].user_id, "wx_user_x");
    assert_eq!(sessions[0].bot_id, bot_id);
}

#[tokio::test]
async fn save_message_and_read_back() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    let repo = PostgresChatRepository::from_pool(pool.clone());
    let bot_id = format!("test-bot-{}", Uuid::new_v4());
    let session_id = format!("test-session-{}", Uuid::new_v4());

    repo.upsert_bot(&bot_id, "online").await.unwrap();
    repo.create_session(&session_id, &bot_id, "user_x").await.unwrap();

    let event = EventEnvelope {
        event_id: Uuid::new_v4().to_string(),
        message_id: Uuid::new_v4().to_string(),
        session_id: session_id.clone(),
        bot_id: bot_id.clone(),
        from_user_id: "user_x".to_string(),
        to_user_id: String::new(),
        content_type: "text".to_string(),
        text_content: "hello integration test".to_string(),
        raw_payload_json: json!({"type": "text", "content": "hello integration test"}),
        received_at_ms: 0,
    };

    repo.save_message(&event).await.unwrap();

    let admin = AdminRepository::new(pool);
    let (rows, total) = admin.list_messages_page(&session_id, 1, 10).await.unwrap();
    assert!(total >= 1, "expected at least 1 message");
    assert_eq!(rows[0].text_content, "hello integration test");
    assert_eq!(rows[0].from_user_id, "user_x");
    assert_eq!(rows[0].content_type, "text");
}

#[tokio::test]
async fn save_media_read() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    let repo = PostgresChatRepository::from_pool(pool.clone());
    let bot_id = format!("test-bot-{}", Uuid::new_v4());
    let session_id = format!("test-session-{}", Uuid::new_v4());

    repo.upsert_bot(&bot_id, "online").await.unwrap();
    repo.create_session(&session_id, &bot_id, "user_img").await.unwrap();

    let msg_id = Uuid::new_v4().to_string();
    let event = EventEnvelope {
        event_id: Uuid::new_v4().to_string(),
        message_id: msg_id.clone(),
        session_id,
        bot_id,
        from_user_id: "user_img".to_string(),
        to_user_id: String::new(),
        content_type: "image".to_string(),
        text_content: String::new(),
        raw_payload_json: json!({}),
        received_at_ms: 0,
    };
    repo.save_message(&event).await.unwrap();

    let media = MediaRecord {
        media_id: Uuid::new_v4().to_string(),
        message_id: msg_id,
        media_type: "image".to_string(),
        storage_backend: "localfs".to_string(),
        storage_key: "test/session/image/test_file".to_string(),
        mime_type: Some("image/png".to_string()),
        size_bytes: 1024,
        sha256: "abc123".to_string(),
        encrypt_meta_json: json!({"format": "png"}),
    };
    repo.save_media(&media).await.unwrap();

    let row: (String,) = sqlx::query_as(
        "SELECT storage_key FROM chat_media WHERE message_id = $1 LIMIT 1",
    )
    .bind(&media.message_id)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(row.0, "test/session/image/test_file");
}

#[tokio::test]
async fn overview_with_seeded_data() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    {
        let _guard = SETUP_MUTEX.lock().unwrap();
        unsafe {
            if !SEEDED {
                seed_medium_dataset(&pool).await;
                SEEDED = true;
            }
        }
    }

    let admin = AdminRepository::new(pool);
    let overview = admin.overview().await.unwrap();

    assert!(overview.total_bots >= 1, "total_bots should be >= 1");
    assert!(overview.messages_today >= 25, "messages_today should be >= 25");
    assert!(overview.forward_dlq_count >= 2, "forward_dlq_count should be >= 2");
    assert!(
        overview.forward_not_success_count >= 2,
        "forward_not_success_count should be >= 2"
    );
}

#[tokio::test]
async fn list_bots_returns_all() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    {
        let _guard = SETUP_MUTEX.lock().unwrap();
        unsafe {
            if !SEEDED {
                seed_medium_dataset(&pool).await;
                SEEDED = true;
            }
        }
    }

    let admin = AdminRepository::new(pool);
    let bots = admin.list_bots().await.unwrap();
    assert!(
        bots.len() >= 1,
        "expected >= 1 bots, got: {}",
        bots.len()
    );
}

#[tokio::test]
async fn list_messages_pagination() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    {
        let _guard = SETUP_MUTEX.lock().unwrap();
        unsafe {
            if !SEEDED {
                seed_medium_dataset(&pool).await;
                SEEDED = true;
            }
        }
    }

    let admin = AdminRepository::new(pool);

    let sessions = admin.list_sessions("bot-001").await.unwrap();
    assert!(!sessions.is_empty(), "bot-001 should have sessions");
    let session_id = &sessions[0].session_id;

    let (page1, total) = admin
        .list_messages_page(session_id, 1, 5)
        .await
        .unwrap();
    assert!(total >= 1, "should have messages total");

    if page1.len() > 0 && total > 5 {
        let (page2, _) = admin
            .list_messages_page(session_id, 2, 5)
            .await
            .unwrap();
        let page1_ids: Vec<&str> = page1.iter().map(|m| m.message_id.as_str()).collect();
        let page2_ids: Vec<&str> = page2.iter().map(|m| m.message_id.as_str()).collect();
        for id in &page1_ids {
            assert!(!page2_ids.contains(id), "pages should not overlap");
        }
    }
}
