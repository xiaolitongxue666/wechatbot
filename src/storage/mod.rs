pub mod media;
pub mod postgres;
pub mod redis_state;

use crate::error::Result;
use crate::ingest::EventEnvelope;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct MediaRecord {
    pub media_id: String,
    pub message_id: String,
    pub media_type: String,
    pub storage_backend: String,
    pub storage_key: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub sha256: String,
    pub encrypt_meta_json: serde_json::Value,
}

#[async_trait]
pub trait ChatRepository: Send + Sync {
    async fn upsert_bot(&self, bot_id: &str, status: &str) -> Result<()>;
    async fn create_session(&self, session_id: &str, bot_id: &str, user_id: &str) -> Result<()>;
    async fn save_message(&self, event: &EventEnvelope) -> Result<()>;
    async fn save_media(&self, media_record: &MediaRecord) -> Result<()>;
}

#[async_trait]
pub trait SessionStateRepository: Send + Sync {
    async fn set_online(&self, session_id: &str, online: bool) -> Result<()>;
    async fn touch_heartbeat(&self, session_id: &str) -> Result<()>;
}
