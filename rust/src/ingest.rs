use crate::bot::WeChatBot;
use crate::error::Result;
use crate::queue::EventQueue;
use crate::storage::media::MediaStore;
use crate::storage::{ChatRepository, MediaRecord};
use crate::types::IncomingMessage;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub message_id: String,
    pub session_id: String,
    pub bot_id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub content_type: String,
    pub text_content: String,
    pub raw_payload_json: serde_json::Value,
    pub received_at_ms: i64,
}

pub struct MessageIngestor {
    repository: Arc<dyn ChatRepository>,
    media_store: Arc<dyn MediaStore>,
    event_queue: Arc<dyn EventQueue>,
}

impl MessageIngestor {
    pub fn new(
        repository: Arc<dyn ChatRepository>,
        media_store: Arc<dyn MediaStore>,
        event_queue: Arc<dyn EventQueue>,
    ) -> Self {
        Self {
            repository,
            media_store,
            event_queue,
        }
    }

    pub async fn ingest(
        &self,
        bot: Arc<WeChatBot>,
        bot_id: &str,
        session_id: &str,
        message: &IncomingMessage,
    ) -> Result<EventEnvelope> {
        let event = EventEnvelope {
            event_id: Uuid::new_v4().to_string(),
            message_id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            bot_id: bot_id.to_string(),
            from_user_id: message.user_id.clone(),
            to_user_id: String::new(),
            content_type: format!("{:?}", message.content_type).to_lowercase(),
            text_content: message.text.clone(),
            raw_payload_json: serde_json::to_value(&message.raw)?,
            received_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_millis() as i64)
                .unwrap_or_default(),
        };
        self.repository.save_message(&event).await?;

        if let Some(downloaded_media) = bot.download(message).await? {
            let stored_media = self
                .media_store
                .save(
                    session_id,
                    &event.message_id,
                    &downloaded_media.media_type,
                    &downloaded_media.data,
                )
                .await?;
            let media_record = MediaRecord {
                media_id: Uuid::new_v4().to_string(),
                message_id: event.message_id.clone(),
                media_type: downloaded_media.media_type,
                storage_backend: stored_media.storage_backend,
                storage_key: stored_media.storage_key,
                mime_type: None,
                size_bytes: stored_media.size_bytes,
                sha256: stored_media.sha256,
                encrypt_meta_json: json!({
                    "format": downloaded_media.format,
                    "file_name": downloaded_media.file_name,
                }),
            };
            self.repository.save_media(&media_record).await?;
        }

        self.event_queue
            .publish(serde_json::to_string(&event)?)
            .await?;
        Ok(event)
    }

    pub async fn ingest_sent(
        &self,
        session_id: &str,
        bot_id: &str,
        bot_user_id: &str,
        to_user_id: &str,
        text: &str,
    ) -> Result<EventEnvelope> {
        let event = EventEnvelope {
            event_id: Uuid::new_v4().to_string(),
            message_id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            bot_id: bot_id.to_string(),
            from_user_id: bot_user_id.to_string(),
            to_user_id: to_user_id.to_string(),
            content_type: "text".to_string(),
            text_content: text.to_string(),
            raw_payload_json: json!({"type": "sent", "text": text}),
            received_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_millis() as i64)
                .unwrap_or_default(),
        };
        self.repository.save_message(&event).await?;
        Ok(event)
    }
}
