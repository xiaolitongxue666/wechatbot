//! Test database lifecycle management.

#![allow(dead_code)]

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub struct TestDb {
    pool: PgPool,
}

impl TestDb {
    pub async fn from_env() -> Self {
        let url = std::env::var("WECHATBOT_TEST_DATABASE_URL")
            .expect("WECHATBOT_TEST_DATABASE_URL is required for integration tests");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&url)
            .await
            .expect("failed to connect to test database");

        TestDb { pool }
    }

    pub async fn migrate(&self) {
        let sql = include_str!("../../migrations/001_init.sql");
        for stmt in split_sql_statements(sql) {
            sqlx::query(&stmt)
                .execute(&self.pool)
                .await
                .unwrap_or_else(|e| panic!("failed to run migration: {stmt}\n{e}"));
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn cleanup(&self) {
        let tables = [
            "forward_dlq",
            "forward_events",
            "chat_media",
            "chat_messages",
            "bot_sessions",
        ];
        for table in &tables {
            let _ = sqlx::query(&format!("DELETE FROM {table}"))
                .execute(&self.pool)
                .await;
        }
    }
}

/// Creates a test database pool from env, runs migrations, returns the pool.
/// This is the main entry point for integration tests.
pub async fn setup_test_db() -> TestDb {
    let db = TestDb::from_env().await;
    db.migrate().await;
    db
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();

    for line in sql.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            current.push_str(line);
            current.push('\n');
            continue;
        }
        if trimmed.ends_with(';') {
            current.push_str(line);
            let stmt = current.trim().trim_end_matches(';').to_string();
            if !stmt.is_empty() {
                statements.push(stmt);
            }
            current = String::new();
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }

    let remaining = current.trim().to_string();
    if !remaining.is_empty() {
        statements.push(remaining);
    }

    statements
}
