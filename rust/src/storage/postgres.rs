use crate::error::{Result, WeChatBotError};
use crate::ingest::EventEnvelope;
use crate::storage::{ChatRepository, MediaRecord};
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub struct PostgresChatRepository {
    pool: PgPool,
}

impl PostgresChatRepository {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|error| WeChatBotError::Other(format!("postgres connect failed: {error}")))?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl ChatRepository for PostgresChatRepository {
    async fn upsert_session(
        &self,
        session_id: &str,
        tenant_id: &str,
        owner_id: &str,
        status: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bot_sessions (
              session_id, tenant_id, owner_id, status, created_at, updated_at
            ) VALUES ($1,$2,$3,$4,NOW(),NOW())
            ON CONFLICT (session_id) DO UPDATE
            SET status = EXCLUDED.status, updated_at = NOW()
            "#,
        )
        .bind(session_id)
        .bind(tenant_id)
        .bind(owner_id)
        .bind(status)
        .execute(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("upsert bot_sessions failed: {error}")))?;
        Ok(())
    }

    async fn save_message(&self, event: &EventEnvelope) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO chat_messages (
              message_id, event_id, session_id, tenant_id, from_user_id, to_user_id,
              content_type, text_content, raw_payload_json, received_at
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,to_timestamp($10::double precision / 1000.0))
            "#,
        )
        .bind(&event.message_id)
        .bind(&event.event_id)
        .bind(&event.session_id)
        .bind(&event.tenant_id)
        .bind(&event.from_user_id)
        .bind(&event.to_user_id)
        .bind(&event.content_type)
        .bind(&event.text_content)
        .bind(&event.raw_payload_json)
        .bind(event.received_at_ms)
        .execute(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("insert chat_messages failed: {error}")))?;
        Ok(())
    }

    async fn save_media(&self, media_record: &MediaRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO chat_media (
              media_id, message_id, media_type, storage_backend, storage_key,
              mime_type, size_bytes, sha256, encrypt_meta_json
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            "#,
        )
        .bind(&media_record.media_id)
        .bind(&media_record.message_id)
        .bind(&media_record.media_type)
        .bind(&media_record.storage_backend)
        .bind(&media_record.storage_key)
        .bind(&media_record.mime_type)
        .bind(media_record.size_bytes)
        .bind(&media_record.sha256)
        .bind(&media_record.encrypt_meta_json)
        .execute(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("insert chat_media failed: {error}")))?;
        Ok(())
    }
}
