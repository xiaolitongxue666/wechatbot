use crate::error::{Result, WeChatBotError};
use async_trait::async_trait;
use redis::AsyncCommands;
use tokio::sync::{mpsc, Mutex};

#[async_trait]
pub trait EventQueue: Send + Sync {
    async fn publish(&self, payload: String) -> Result<()>;
    async fn consume(&self) -> Result<String>;
}

pub struct InMemoryEventQueue {
    sender: mpsc::Sender<String>,
    receiver: Mutex<mpsc::Receiver<String>>,
}

impl InMemoryEventQueue {
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

#[async_trait]
impl EventQueue for InMemoryEventQueue {
    async fn publish(&self, payload: String) -> Result<()> {
        self.sender
            .send(payload)
            .await
            .map_err(|error| WeChatBotError::Other(format!("publish failed: {error}")))?;
        Ok(())
    }

    async fn consume(&self) -> Result<String> {
        let mut receiver = self.receiver.lock().await;
        receiver
            .recv()
            .await
            .ok_or_else(|| WeChatBotError::Other("queue closed".into()))
    }
}

pub struct RedisEventQueue {
    client: redis::Client,
    queue_key: String,
}

impl RedisEventQueue {
    pub fn new(redis_url: &str, queue_key: impl Into<String>) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|error| WeChatBotError::Other(format!("invalid redis url: {error}")))?;
        Ok(Self {
            client,
            queue_key: queue_key.into(),
        })
    }
}

#[async_trait]
impl EventQueue for RedisEventQueue {
    async fn publish(&self, payload: String) -> Result<()> {
        let mut connection = self.client.get_multiplexed_async_connection().await.map_err(|error| {
            WeChatBotError::Other(format!("redis connect failed: {error}"))
        })?;
        let _: () = connection
            .rpush(&self.queue_key, payload)
            .await
            .map_err(|error| WeChatBotError::Other(format!("redis publish failed: {error}")))?;
        Ok(())
    }

    async fn consume(&self) -> Result<String> {
        let mut connection = self.client.get_multiplexed_async_connection().await.map_err(|error| {
            WeChatBotError::Other(format!("redis connect failed: {error}"))
        })?;
        let value: Option<[String; 2]> = connection
            .blpop(&self.queue_key, 0.0)
            .await
            .map_err(|error| WeChatBotError::Other(format!("redis consume failed: {error}")))?;
        value
            .map(|item| item[1].clone())
            .ok_or_else(|| WeChatBotError::Other("redis queue returned empty payload".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_queue_publish_and_consume() {
        let queue = InMemoryEventQueue::with_capacity(4);
        queue
            .publish("event-1".to_string())
            .await
            .expect("publish should succeed");
        let payload = queue.consume().await.expect("consume should succeed");
        assert_eq!(payload, "event-1");
    }
}
