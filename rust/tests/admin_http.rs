//! HTTP smoke tests for admin dashboard.
//! Requires Postgres with migrations applied.
//! Set `WECHATBOT_TEST_DATABASE_URL` to run, e.g.:
//!   WECHATBOT_TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5433/wechatbot

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use wechatbot::admin_router;

async fn test_pool() -> sqlx::PgPool {
    let url = std::env::var("WECHATBOT_TEST_DATABASE_URL")
        .expect("WECHATBOT_TEST_DATABASE_URL is required for integration tests. Set it to e.g. postgres://postgres:postgres@localhost:5433/wechatbot");
    PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("failed to connect to test database")
}

#[tokio::test]
async fn healthz_ok() {
    let pool = test_pool().await;
    let app = admin_router(pool);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_dashboard_200_default_zh_dark() {
    let pool = test_pool().await;
    let app = admin_router(pool);
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
        "expected dark theme in html: {html}"
    );
    assert!(
        html.contains("zh-CN") || html.contains("仪表盘"),
        "expected chinese UI: {html}"
    );
}

#[tokio::test]
async fn admin_history_pagination_query() {
    let pool = test_pool().await;
    let app = admin_router(pool);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/bots/nonexistent-session-xyz/history?page=2&lang=en&theme=dark")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
