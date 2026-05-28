use crate::error::{Result, WeChatBotError};
use crate::storage::SessionStateRepository;
use async_trait::async_trait;
use redis::AsyncCommands;

pub struct RedisSessionStateRepository {
    client: redis::Client,
    key_prefix: String,
}

impl RedisSessionStateRepository {
    pub fn new(redis_url: &str, key_prefix: impl Into<String>) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|error| WeChatBotError::Other(format!("invalid redis url: {error}")))?;
        Ok(Self {
            client,
            key_prefix: key_prefix.into(),
        })
    }

    fn online_key(&self, session_id: &str) -> String {
        format!("{}:session:{}:online", self.key_prefix, session_id)
    }

    fn heartbeat_key(&self, session_id: &str) -> String {
        format!("{}:session:{}:heartbeat", self.key_prefix, session_id)
    }
}

#[async_trait]
impl SessionStateRepository for RedisSessionStateRepository {
    async fn set_online(&self, session_id: &str, online: bool) -> Result<()> {
        let mut connection = self.client.get_multiplexed_async_connection().await.map_err(|error| {
            WeChatBotError::Other(format!("redis connect failed: {error}"))
        })?;
        let _: () = connection
            .set(self.online_key(session_id), if online { "1" } else { "0" })
            .await
            .map_err(|error| WeChatBotError::Other(format!("set online status failed: {error}")))?;
        Ok(())
    }

    async fn touch_heartbeat(&self, session_id: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| WeChatBotError::Other(format!("system time error: {error}")))?
            .as_secs() as i64;
        let mut connection = self.client.get_multiplexed_async_connection().await.map_err(|error| {
            WeChatBotError::Other(format!("redis connect failed: {error}"))
        })?;
        let _: () = connection
            .set(self.heartbeat_key(session_id), now)
            .await
            .map_err(|error| WeChatBotError::Other(format!("set heartbeat failed: {error}")))?;
        Ok(())
    }
}
