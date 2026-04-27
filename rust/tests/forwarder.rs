//! Backend integration tests: forwarder with wiremock webhook.

mod common;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use uuid::Uuid;
use wechatbot::config::ForwarderConfig;
use wechatbot::forwarder::ForwarderWorker;
use wechatbot::ingest::EventEnvelope;
use wechatbot::queue::{EventQueue, InMemoryEventQueue};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::db::setup_test_db;

fn sample_envelope() -> EventEnvelope {
    EventEnvelope {
        event_id: Uuid::new_v4().to_string(),
        message_id: Uuid::new_v4().to_string(),
        session_id: format!("session-{}", Uuid::new_v4()),
        bot_id: "test-bot".to_string(),
        from_user_id: "user_x".to_string(),
        to_user_id: String::new(),
        content_type: "text".to_string(),
        text_content: "test message".to_string(),
        raw_payload_json: json!({"type": "text", "content": "test"}),
        received_at_ms: 0,
    }
}

#[tokio::test]
async fn forwarder_success_sets_forward_state() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let queue = Arc::new(InMemoryEventQueue::with_capacity(4));
    let config = ForwarderConfig {
        endpoint: format!("{}/webhook", mock_server.uri()),
        hmac_secret: "test-secret".to_string(),
        max_retries: 3,
        timeout_ms: 2000,
    };
    let forwarder = ForwarderWorker::new(queue.clone() as Arc<dyn EventQueue>, config)
        .with_postgres_pool(pool.clone());

    let envelope = sample_envelope();
    let event_id = envelope.event_id.clone();

    let result = forwarder.forward_with_retry(&envelope).await;
    assert!(result.is_ok(), "forward should succeed: {:?}", result.err());

    let status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM forward_events WHERE event_id = $1",
    )
    .bind(&event_id)
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(status.as_deref(), Some("success"));
}

#[tokio::test]
async fn forwarder_retry_then_success() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    let mock_server = MockServer::start().await;

    let call_count = Arc::new(AtomicU32::new(0));
    let call_count_clone = call_count.clone();

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(move |_req: &wiremock::Request| {
            let n = call_count_clone.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                ResponseTemplate::new(500)
            } else {
                ResponseTemplate::new(200)
            }
        })
        .mount(&mock_server)
        .await;

    let queue = Arc::new(InMemoryEventQueue::with_capacity(4));
    let config = ForwarderConfig {
        endpoint: format!("{}/webhook", mock_server.uri()),
        hmac_secret: "test-secret".to_string(),
        max_retries: 3,
        timeout_ms: 2000,
    };
    let forwarder = ForwarderWorker::new(queue.clone() as Arc<dyn EventQueue>, config)
        .with_postgres_pool(pool.clone());

    let envelope = sample_envelope();
    let event_id = envelope.event_id.clone();

    let result = forwarder.forward_with_retry(&envelope).await;
    assert!(result.is_ok(), "forward should succeed after retries: {:?}", result.err());

    let status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM forward_events WHERE event_id = $1",
    )
    .bind(&event_id)
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(status.as_deref(), Some("success"));
}

#[tokio::test]
async fn forwarder_all_fail_writes_dlq() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let queue = Arc::new(InMemoryEventQueue::with_capacity(4));
    let config = ForwarderConfig {
        endpoint: format!("{}/webhook", mock_server.uri()),
        hmac_secret: "test-secret".to_string(),
        max_retries: 2,
        timeout_ms: 2000,
    };
    let forwarder = ForwarderWorker::new(queue.clone() as Arc<dyn EventQueue>, config)
        .with_postgres_pool(pool.clone());

    let envelope = sample_envelope();
    let event_id = envelope.event_id.clone();
    let payload = serde_json::to_string(&envelope).unwrap();

    let result = forwarder.forward_with_retry(&envelope).await;
    assert!(result.is_err(), "forward should fail after max retries");

    let status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM forward_events WHERE event_id = $1",
    )
    .bind(&event_id)
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(status.as_deref(), Some("failed"));

    forwarder.write_dlq(&envelope, &payload, "test error").await.unwrap();

    let dlq_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM forward_dlq WHERE event_id = $1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(dlq_count.0, 1, "DLQ should have the failed event");
}

#[tokio::test]
async fn forwarder_skips_already_succeeded() {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let queue = Arc::new(InMemoryEventQueue::with_capacity(4));
    let config = ForwarderConfig {
        endpoint: format!("{}/webhook", mock_server.uri()),
        hmac_secret: "test-secret".to_string(),
        max_retries: 3,
        timeout_ms: 2000,
    };
    let forwarder = ForwarderWorker::new(queue.clone() as Arc<dyn EventQueue>, config)
        .with_postgres_pool(pool.clone());

    let envelope = sample_envelope();
    let event_id = envelope.event_id.clone();

    sqlx::query(
        "INSERT INTO forward_events (event_id, session_id, target_service, status, retry_count, updated_at) VALUES ($1,$2,$3,'success',0,NOW())",
    )
    .bind(&event_id)
    .bind("session-001")
    .bind("http://test/webhook")
    .execute(&pool)
    .await
    .unwrap();

    let result = forwarder.forward_with_retry(&envelope).await;
    assert!(result.is_ok(), "should skip already succeeded event");
}

#[tokio::test]
async fn hmac_signature_length_valid() {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let envelope = sample_envelope();
    let payload = serde_json::to_vec(&envelope).unwrap();

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(b"test-key").unwrap();
    mac.update(&payload);
    let sig = hex::encode(mac.finalize().into_bytes());
    assert_eq!(sig.len(), 64, "HMAC-SHA256 hex signature should be 64 chars");

    let mut mac2 = HmacSha256::new_from_slice(b"test-key").unwrap();
    mac2.update(&payload);
    let sig2 = hex::encode(mac2.finalize().into_bytes());
    assert_eq!(sig, sig2, "same payload + key should produce same signature");
}
