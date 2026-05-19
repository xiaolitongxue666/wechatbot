//! HTTP smoke tests for admin dashboard.
//! Requires Postgres with migrations applied.
//! Set `WECHATBOT_TEST_DATABASE_URL` to run, e.g.:
//!   WECHATBOT_TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5433/wechatbot

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use wechatbot::admin_router;

use common::db::setup_test_db;
use common::fixtures::seed_admin_rbac;

const ADMIN_AUTH_HEADER: &str = "Bearer dev-admin-token";

async fn test_pool() -> sqlx::PgPool {
    let db = setup_test_db().await;
    let pool = db.pool().clone();
    seed_admin_rbac(&pool).await;
    pool
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
async fn admin_spa_shell_200() {
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
        html.contains("<!doctype html>") || html.contains("<!DOCTYPE html>"),
        "expected html shell: {html}"
    );
}

#[tokio::test]
async fn admin_spa_history_fallback_200() {
    let pool = test_pool().await;
    let app = admin_router(pool);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/random/nested/route")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_api_bots_200() {
    let pool = test_pool().await;
    let app = admin_router(pool);
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
}

#[tokio::test]
async fn admin_api_system_logs_200() {
    let pool = test_pool().await;
    let app = admin_router(pool);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/api/system-logs/admin?lines=10")
                .header("authorization", ADMIN_AUTH_HEADER)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_api_rejects_invalid_token() {
    let pool = test_pool().await;
    let app = admin_router(pool);
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

#[tokio::test]
async fn admin_api_unknown_path_404() {
    let pool = test_pool().await;
    let app = admin_router(pool);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/admin/api/not-exists")
                .header("authorization", ADMIN_AUTH_HEADER)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
