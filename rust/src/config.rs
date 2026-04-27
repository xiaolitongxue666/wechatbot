use crate::error::{Result, WeChatBotError};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseMode {
    Local,
    Container,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub mode: DatabaseMode,
    pub local_url: Option<String>,
    pub container_url: Option<String>,
    pub remote_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub mode: DatabaseMode,
    pub local_url: Option<String>,
    pub container_url: Option<String>,
    pub remote_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaConfig {
    pub backend: String,
    pub local_root: Option<String>,
    pub bucket: Option<String>,
    pub endpoint: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwarderConfig {
    pub endpoint: String,
    pub hmac_secret: String,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    #[serde(default = "default_admin_bind")]
    pub bind: String,
}

fn default_admin_bind() -> String {
    "127.0.0.1:8787".into()
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            bind: default_admin_bind(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub media: MediaConfig,
    pub forwarder: ForwarderConfig,
    #[serde(default)]
    pub admin: AdminConfig,
}

impl AppConfig {
    pub async fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let mut config: AppConfig = toml::from_str(&content)
            .map_err(|error| WeChatBotError::Other(format!("invalid config: {error}")))?;
        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    pub fn database_url(&self) -> Result<&str> {
        self.pick_url(
            &self.database.mode,
            self.database.local_url.as_deref(),
            self.database.container_url.as_deref(),
            self.database.remote_url.as_deref(),
            "database",
        )
    }

    pub fn redis_url(&self) -> Result<&str> {
        self.pick_url(
            &self.redis.mode,
            self.redis.local_url.as_deref(),
            self.redis.container_url.as_deref(),
            self.redis.remote_url.as_deref(),
            "redis",
        )
    }

    fn pick_url<'a>(
        &self,
        mode: &DatabaseMode,
        local: Option<&'a str>,
        container: Option<&'a str>,
        remote: Option<&'a str>,
        component: &str,
    ) -> Result<&'a str> {
        let selected = match mode {
            DatabaseMode::Local => local,
            DatabaseMode::Container => container,
            DatabaseMode::Remote => remote,
        };
        selected.ok_or_else(|| {
            WeChatBotError::Other(format!(
                "missing {component} url for mode {:?}",
                mode
            ))
        })
    }

    fn validate(&self) -> Result<()> {
        if self.forwarder.endpoint.trim().is_empty() {
            return Err(WeChatBotError::Other("forwarder.endpoint is required".into()));
        }
        if self.forwarder.hmac_secret.trim().is_empty() {
            return Err(WeChatBotError::Other("forwarder.hmac_secret is required".into()));
        }
        if self.forwarder.max_retries == 0 {
            return Err(WeChatBotError::Other("forwarder.max_retries must be > 0".into()));
        }
        let _ = self.database_url()?;
        if self.redis_url().is_err() {
            tracing::warn!("redis url not configured; redis-based features (event queue, session state) will be unavailable");
        }
        Ok(())
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(value) = std::env::var("WECHATBOT_DATABASE_MODE") {
            self.database.mode = parse_mode(&value);
        }
        if let Ok(value) = std::env::var("WECHATBOT_DATABASE_LOCAL_URL") {
            self.database.local_url = Some(value);
        }
        if let Ok(value) = std::env::var("WECHATBOT_DATABASE_CONTAINER_URL") {
            self.database.container_url = Some(value);
        }
        if let Ok(value) = std::env::var("WECHATBOT_DATABASE_REMOTE_URL") {
            self.database.remote_url = Some(value);
        }

        if let Ok(value) = std::env::var("WECHATBOT_REDIS_MODE") {
            self.redis.mode = parse_mode(&value);
        }
        if let Ok(value) = std::env::var("WECHATBOT_REDIS_LOCAL_URL") {
            self.redis.local_url = Some(value);
        }
        if let Ok(value) = std::env::var("WECHATBOT_REDIS_CONTAINER_URL") {
            self.redis.container_url = Some(value);
        }
        if let Ok(value) = std::env::var("WECHATBOT_REDIS_REMOTE_URL") {
            self.redis.remote_url = Some(value);
        }
    }
}

fn parse_mode(raw_mode: &str) -> DatabaseMode {
    match raw_mode.trim().to_lowercase().as_str() {
        "container" => DatabaseMode::Container,
        "remote" => DatabaseMode::Remote,
        _ => DatabaseMode::Local,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> AppConfig {
        AppConfig {
            database: DatabaseConfig {
                mode: DatabaseMode::Local,
                local_url: Some("postgres://local".into()),
                container_url: Some("postgres://container".into()),
                remote_url: Some("postgres://remote".into()),
            },
            redis: RedisConfig {
                mode: DatabaseMode::Local,
                local_url: Some("redis://local".into()),
                container_url: Some("redis://container".into()),
                remote_url: Some("redis://remote".into()),
            },
            media: MediaConfig {
                backend: "localfs".into(),
                local_root: Some("./data/media".into()),
                bucket: None,
                endpoint: None,
                access_key: None,
                secret_key: None,
            },
            forwarder: ForwarderConfig {
                endpoint: "http://127.0.0.1/webhook".into(),
                hmac_secret: "secret".into(),
                max_retries: 3,
                timeout_ms: 1000,
            },
            admin: AdminConfig::default(),
        }
    }

    #[test]
    fn database_url_uses_selected_mode() {
        let mut config = sample_config();
        config.database.mode = DatabaseMode::Container;
        assert_eq!(
            config.database_url().expect("database url"),
            "postgres://container"
        );
    }

    #[test]
    fn redis_url_uses_selected_mode() {
        let mut config = sample_config();
        config.redis.mode = DatabaseMode::Remote;
        assert_eq!(config.redis_url().expect("redis url"), "redis://remote");
    }
}
