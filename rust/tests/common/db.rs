//! Test database lifecycle management.

#![allow(dead_code)]

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

static TEST_DB_MIGRATION_LOCK: Mutex<()> = Mutex::const_new(());
static TEST_DB_MIGRATED: AtomicBool = AtomicBool::new(false);

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
        let drop_statements = [
            "DROP TABLE IF EXISTS admin_user_bot_scopes CASCADE",
            "DROP TABLE IF EXISTS admin_permissions CASCADE",
            "DROP TABLE IF EXISTS admin_users CASCADE",
            "DROP TABLE IF EXISTS bot_forward_policies CASCADE",
            "DROP TABLE IF EXISTS forward_dlq CASCADE",
            "DROP TABLE IF EXISTS forward_events CASCADE",
            "DROP TABLE IF EXISTS chat_media CASCADE",
            "DROP TABLE IF EXISTS chat_messages CASCADE",
            "DROP TABLE IF EXISTS bot_sessions CASCADE",
            "DROP TABLE IF EXISTS bots CASCADE",
        ];
        for drop_statement in drop_statements {
            sqlx::query(drop_statement)
                .execute(&self.pool)
                .await
                .unwrap_or_else(|error| panic!("failed to execute drop statement {drop_statement}: {error}"));
        }

        let migration_sql_list = [
            include_str!("../../migrations/001_init.sql"),
            include_str!("../../migrations/002_restructure.sql"),
            include_str!("../../migrations/003_rbac_forward_policy.sql"),
        ];
        for migration_sql in migration_sql_list {
            sqlx::raw_sql(migration_sql)
                .execute(&self.pool)
                .await
                .unwrap_or_else(|error| panic!("failed to run migration batch:\n{migration_sql}\n{error}"));
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn cleanup(&self) {
        let tables = [
            "admin_user_bot_scopes",
            "admin_permissions",
            "admin_users",
            "bot_forward_policies",
            "forward_dlq",
            "forward_events",
            "chat_media",
            "chat_messages",
            "bot_sessions",
            "bots",
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
    if !TEST_DB_MIGRATED.load(Ordering::Acquire) {
        let _guard = TEST_DB_MIGRATION_LOCK.lock().await;
        if !TEST_DB_MIGRATED.load(Ordering::Relaxed) {
            db.migrate().await;
            TEST_DB_MIGRATED.store(true, Ordering::Release);
        }
    }
    db
}

