//! Frontend HTTP integration tests for the admin dashboard.
//! Requires `WECHATBOT_TEST_DATABASE_URL` pointing to a Postgres with migrations applied.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Mutex;
use tower::ServiceExt;
use wechatbot::admin_router;

use common::db::setup_test_db;
use common::fixtures::seed_medium_dataset;

static SETUP_MUTEX: Mutex<()> = Mutex::new(());
static mut SEEDED: bool = false;

async fn get_app() -> axum::Router {
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

    admin_router(pool)
}

#[tokio::test]
async fn healthz_ok() {
    let app = get_app().await;
    let res = app
        .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body.as_ref(), b"ok");
}

#[tokio::test]
async fn root_redirects_to_admin() {
    let app = get_app().await;
    let res = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::TEMPORARY_REDIRECT);
}

#[tokio::test]
async fn dashboard_default_zh_dark() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("theme-dark"),
        "expected dark theme, got: {html}"
    );
    assert!(
        html.contains("zh-CN") || html.contains("仪表盘"),
        "expected Chinese UI, got: {html}"
    );
}

#[tokio::test]
async fn dashboard_en_light_query() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin?lang=en&theme=light")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("theme-light"),
        "expected light theme, got: {html}"
    );
    assert!(
        html.contains("lang=\"en\""),
        "expected English lang attribute, got: {html}"
    );
    assert!(
        html.contains("Dashboard"),
        "expected English content, got: {html}"
    );
}

#[tokio::test]
async fn bot_list_has_seeded_bots() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/bots")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert_eq!(status, StatusCode::OK, "unexpected status: {status}, body: {html}");
    assert!(
        html.contains("bot-001"),
        "expected bot-001 in bot list: {html}"
    );
    assert!(
        html.contains("bot-003"),
        "expected bot-003 in bot list: {html}"
    );
    assert!(
        html.contains("bot-005"),
        "expected bot-005 in bot list: {html}"
    );
    assert!(html.contains("在线") || html.contains("Online"), "expected status text: {html}");
    assert!(html.contains("离线") || html.contains("Offline"), "expected offline text: {html}");
}

#[tokio::test]
async fn bot_detail_valid_bot() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/bots/bot-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert_eq!(status, StatusCode::OK, "unexpected status: {status}, body: {html}");
    assert!(
        html.contains("bot-001"),
        "expected bot-001 in detail: {html}"
    );
    assert!(html.contains("在线") || html.contains("online"), "expected online status: {html}");
}

#[tokio::test]
async fn bot_detail_nonexistent_404() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/bots/nonexistent-bot-xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn bot_history_with_messages() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/bots/sess-001/history")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("sess-001") || html.contains("hello world") || html.contains("user_alice"),
        "expected seed data in history: {html}"
    );
}

#[tokio::test]
async fn bot_history_nonexistent_404() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/bots/ghost-session/history")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn api_overview_json() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/api/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();

    let parsed: serde_json::Value =
        serde_json::from_slice(&body).expect("overview should be valid JSON");
    let total_bots = parsed["total_bots"].as_i64().unwrap();
    let online_bots = parsed["online_bots"].as_i64().unwrap();
    let messages_today = parsed["messages_today"].as_i64().unwrap();
    let dlq_count = parsed["forward_dlq_count"].as_i64().unwrap();
    let non_success = parsed["forward_not_success_count"].as_i64().unwrap();

    assert!(total_bots >= 1, "expected >= 1 total bots, got: {total_bots}");
    assert!(online_bots >= 1, "expected >= 1 online bots, got: {online_bots}");
    assert!(messages_today >= 25, "expected >= 25 messages, got: {messages_today}");
    assert!(dlq_count >= 2, "expected >= 2 DLQ entries, got: {dlq_count}");
    assert!(non_success >= 2, "expected >= 2 non-success forwards, got: {non_success}");
}
