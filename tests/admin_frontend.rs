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

const ADMIN_AUTH_HEADER: &str = "Bearer dev-admin-token";

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
async fn admin_root_serves_spa_shell() {
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
    let content = String::from_utf8_lossy(&body);
    assert!(
        content.contains("<!doctype html>") || content.contains("<!DOCTYPE html>"),
        "unexpected admin shell payload: {content}"
    );
}

#[tokio::test]
async fn spa_history_fallback_route_works() {
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
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("<!doctype html>") || html.contains("<!DOCTYPE html>"),
        "unexpected fallback payload: {html}"
    );
}

#[tokio::test]
async fn api_bot_list_json() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/api/bots")
                .header("authorization", ADMIN_AUTH_HEADER)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&body).expect("should be valid JSON");
    let rows = parsed["bots"].as_array().expect("bots should be array");
    assert!(!rows.is_empty(), "expected non-empty bots list");
    for row in rows {
        assert!(row["bot_id"].as_str().is_some(), "bot_id should exist");
        assert!(
            row["messages_today"].as_i64().is_some(),
            "messages_today should be number"
        );
        assert!(
            row["forward_failures_today"].as_i64().is_some(),
            "forward_failures_today should be number"
        );
    }
}

#[tokio::test]
async fn api_bot_detail_json() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/api/bots/bot-001")
                .header("authorization", ADMIN_AUTH_HEADER)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&body).expect("should be valid JSON");
    assert_eq!(parsed["bot_id"].as_str(), Some("bot-001"));
    assert!(parsed["sessions"].is_array(), "sessions should be array");
}

#[tokio::test]
async fn api_overview_json() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/api/overview")
                .header("authorization", ADMIN_AUTH_HEADER)
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
    let forward_failures_today = parsed["forward_failures_today"].as_i64().unwrap();

    assert!(total_bots >= 1, "expected >= 1 total bots, got: {total_bots}");
    assert!(online_bots >= 1, "expected >= 1 online bots, got: {online_bots}");
    assert!(messages_today >= 25, "expected >= 25 messages, got: {messages_today}");
    assert!(
        forward_failures_today >= 3,
        "expected >= 3 today non-success forwards, got: {forward_failures_today}"
    );
}

#[tokio::test]
async fn api_forward_policy_json() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/api/bots/bot-001/forward-policy")
                .header("authorization", ADMIN_AUTH_HEADER)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value =
        serde_json::from_slice(&body).expect("forward policy should be valid JSON");
    assert_eq!(parsed["bot_id"].as_str(), Some("bot-001"));
}

#[tokio::test]
async fn api_session_history_json() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/api/sessions/sess-001/history?page=1&page_size=10")
                .header("authorization", ADMIN_AUTH_HEADER)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&body).expect("history should be valid JSON");
    assert_eq!(parsed["session_id"].as_str(), Some("sess-001"));
    assert!(parsed["rows"].is_array(), "rows should be array");
}

#[tokio::test]
async fn api_overview_rejects_invalid_token() {
    let app = get_app().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/api/overview")
                .header("authorization", "Bearer invalid-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
