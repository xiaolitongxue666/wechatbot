use crate::config::ForwarderConfig;
use crate::error::{Result, WeChatBotError};
use crate::ingest::EventEnvelope;
use crate::queue::EventQueue;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardEvent {
    pub event_id: String,
    pub session_id: String,
    pub tenant_id: String,
    pub status: String,
    pub retry_count: i32,
    pub last_error: Option<String>,
}

pub struct ForwarderWorker {
    queue: Arc<dyn EventQueue>,
    config: ForwarderConfig,
    http_client: reqwest::Client,
    pg_pool: Option<PgPool>,
}

impl ForwarderWorker {
    pub fn new(queue: Arc<dyn EventQueue>, config: ForwarderConfig) -> Self {
        Self {
            queue,
            http_client: reqwest::Client::new(),
            config,
            pg_pool: None,
        }
    }

    pub fn with_postgres_pool(mut self, pg_pool: PgPool) -> Self {
        self.pg_pool = Some(pg_pool);
        self
    }

    pub async fn run_forever(&self) -> Result<()> {
        loop {
            let payload = self.queue.consume().await?;
            let envelope: EventEnvelope = serde_json::from_str(&payload)?;
            if let Err(error) = self.forward_with_retry(&envelope).await {
                self.write_dlq(&envelope, &payload, &error.to_string()).await?;
            }
        }
    }

    pub async fn forward_with_retry(&self, envelope: &EventEnvelope) -> Result<()> {
        if self.is_already_succeeded(&envelope.event_id).await? {
            return Ok(());
        }
        let mut retry_count = 0_u32;
        let mut delay = Duration::from_millis(500);
        loop {
            match self.forward_once(envelope).await {
                Ok(()) => {
                    self.write_forward_state(&envelope.event_id, &envelope.session_id, "success", retry_count as i32, None).await?;
                    return Ok(());
                }
                Err(error) => {
                    retry_count += 1;
                    if retry_count >= self.config.max_retries {
                        self.write_forward_state(
                            &envelope.event_id,
                            &envelope.session_id,
                            "failed",
                            retry_count as i32,
                            Some(error.to_string()),
                        )
                        .await?;
                        return Err(WeChatBotError::Other(format!(
                            "forward failed after {} retries: {}",
                            retry_count, error
                        )));
                    }
                    sleep(delay).await;
                    delay = std::cmp::min(delay.saturating_mul(2), Duration::from_secs(8));
                    self.write_forward_state(
                        &envelope.event_id,
                        &envelope.session_id,
                        "retrying",
                        retry_count as i32,
                        Some(error.to_string()),
                    )
                    .await?;
                }
            }
        }
    }

    async fn forward_once(&self, envelope: &EventEnvelope) -> Result<()> {
        let payload = serde_json::to_vec(envelope)?;
        let signature = sign_hmac(&self.config.hmac_secret, &payload)?;
        let response = self
            .http_client
            .post(&self.config.endpoint)
            .header("content-type", "application/json")
            .header("x-event-id", &envelope.event_id)
            .header("x-signature-sha256", signature)
            .timeout(Duration::from_millis(self.config.timeout_ms))
            .body(payload)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(WeChatBotError::Other(format!(
                "forward endpoint returned {}",
                response.status()
            )));
        }
        Ok(())
    }

    async fn write_forward_state(
        &self,
        event_id: &str,
        session_id: &str,
        status: &str,
        retry_count: i32,
        last_error: Option<String>,
    ) -> Result<()> {
        let Some(pool) = &self.pg_pool else {
            return Ok(());
        };
        sqlx::query(
            r#"
            INSERT INTO forward_events (
              event_id, session_id, target_service, status, retry_count, last_error, updated_at
            ) VALUES ($1,$2,$3,$4,$5,$6,NOW())
            ON CONFLICT (event_id) DO UPDATE
            SET status = EXCLUDED.status,
                retry_count = EXCLUDED.retry_count,
                last_error = EXCLUDED.last_error,
                updated_at = NOW()
            "#,
        )
        .bind(event_id)
        .bind(session_id)
        .bind(&self.config.endpoint)
        .bind(status)
        .bind(retry_count)
        .bind(last_error)
        .execute(pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("upsert forward_events failed: {error}")))?;
        Ok(())
    }

    pub async fn write_dlq(&self, envelope: &EventEnvelope, payload: &str, error_message: &str) -> Result<()> {
        let Some(pool) = &self.pg_pool else {
            return Ok(());
        };
        sqlx::query(
            r#"
            INSERT INTO forward_dlq (event_id, session_id, payload_json, error_message, failed_at)
            VALUES ($1,$2,$3::jsonb,$4,NOW())
            ON CONFLICT (event_id) DO UPDATE
            SET error_message = EXCLUDED.error_message,
                payload_json = EXCLUDED.payload_json,
                failed_at = NOW()
            "#,
        )
        .bind(&envelope.event_id)
        .bind(&envelope.session_id)
        .bind(payload)
        .bind(error_message)
        .execute(pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("insert forward_dlq failed: {error}")))?;
        Ok(())
    }

    async fn is_already_succeeded(&self, event_id: &str) -> Result<bool> {
        let Some(pool) = &self.pg_pool else {
            return Ok(false);
        };
        let status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM forward_events WHERE event_id = $1 LIMIT 1",
        )
        .bind(event_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("query forward status failed: {error}")))?;
        Ok(matches!(status.as_deref(), Some("success")))
    }
}

fn sign_hmac(secret: &str, payload: &[u8]) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|error| WeChatBotError::Other(format!("hmac init failed: {error}")))?;
    mac.update(payload);
    Ok(hex::encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_signature_is_stable() {
        let signature = sign_hmac("secret", br#"{"hello":"world"}"#).expect("sign");
        assert_eq!(signature.len(), 64);
    }
}
