use crate::bot::WeChatBot;
use crate::config::AppConfig;
use crate::error::{Result, WeChatBotError};
use crate::forwarder::ForwarderWorker;
use crate::ingest::MessageIngestor;
use crate::queue::{EventQueue, RedisEventQueue};
use crate::session::{BotSession, BotSessionManager};
use crate::storage::media::{LocalFsMediaStore, MediaStore, S3CompatibleMediaStore};
use crate::storage::postgres::PostgresChatRepository;
use crate::storage::redis_state::RedisSessionStateRepository;
use crate::storage::{ChatRepository, SessionStateRepository};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

pub struct MultiBotRuntime {
    pub config: AppConfig,
    pub session_manager: Arc<BotSessionManager>,
    pub session_state_repo: Arc<dyn SessionStateRepository>,
    pub chat_repository: Arc<dyn ChatRepository>,
    pub ingestor: Arc<MessageIngestor>,
    pub forwarder: Arc<ForwarderWorker>,
    heartbeat_tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

impl MultiBotRuntime {
    pub async fn from_config(config: AppConfig) -> Result<Self> {
        let postgres_repository =
            Arc::new(PostgresChatRepository::connect(config.database_url()?).await?);
        let chat_repository: Arc<dyn ChatRepository> = postgres_repository.clone();
        let media_store: Arc<dyn MediaStore> = if config.media.backend == "localfs" {
            let local_root = config
                .media
                .local_root
                .clone()
                .unwrap_or_else(|| "./data/media".into());
            Arc::new(LocalFsMediaStore::new(local_root))
        } else {
            let bucket = config
                .media
                .bucket
                .clone()
                .ok_or_else(|| WeChatBotError::Other("media.bucket is required".into()))?;
            let endpoint = config
                .media
                .endpoint
                .clone()
                .ok_or_else(|| WeChatBotError::Other("media.endpoint is required".into()))?;
            Arc::new(S3CompatibleMediaStore::new(bucket, endpoint))
        };
        let event_queue: Arc<dyn EventQueue> =
            Arc::new(RedisEventQueue::new(config.redis_url()?, "wechatbot:events")?);
        let session_state_repo: Arc<dyn SessionStateRepository> = Arc::new(
            RedisSessionStateRepository::new(config.redis_url()?, "wechatbot")?,
        );
        let ingestor = Arc::new(MessageIngestor::new(
            Arc::clone(&chat_repository),
            media_store,
            Arc::clone(&event_queue),
        ));
        let forwarder = Arc::new(
            ForwarderWorker::new(event_queue, config.forwarder.clone())
                .with_postgres_pool(postgres_repository.pool().clone()),
        );
        Ok(Self {
            config,
            session_manager: Arc::new(BotSessionManager::new()),
            session_state_repo,
            chat_repository: Arc::clone(&chat_repository),
            ingestor,
            forwarder,
            heartbeat_tasks: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn register_bot(
        &self,
        tenant_id: &str,
        owner_id: &str,
        session_id: &str,
        bot: Arc<WeChatBot>,
    ) -> Result<()> {
        self.chat_repository
            .upsert_session(session_id, tenant_id, owner_id, "offline")
            .await?;
        let tenant_id_owned = tenant_id.to_string();
        let session_id_owned = session_id.to_string();
        let bot_for_ingest = Arc::clone(&bot);
        let ingestor = Arc::clone(&self.ingestor);
        bot.on_message(Box::new(move |message| {
            let message = message.clone();
            let tenant_id_owned = tenant_id_owned.clone();
            let session_id_owned = session_id_owned.clone();
            let bot_for_ingest = Arc::clone(&bot_for_ingest);
            let ingestor = Arc::clone(&ingestor);
            tokio::spawn(async move {
                if let Err(error) = ingestor
                    .ingest(
                        bot_for_ingest,
                        &tenant_id_owned,
                        &session_id_owned,
                        &message,
                    )
                    .await
                {
                    tracing::error!("ingest failed: {}", error);
                }
            });
        }))
        .await;
        self.session_manager
            .register_session(BotSession {
                session_id: session_id.to_string(),
                tenant_id: tenant_id.to_string(),
                owner_id: owner_id.to_string(),
                bot,
            })
            .await
    }

    pub async fn start_session(&self, session_id: &str, force_login: bool) -> Result<()> {
        self.session_manager.start_session(session_id, force_login).await?;
        self.session_state_repo.set_online(session_id, true).await?;
        self.start_heartbeat_task(session_id).await;
        Ok(())
    }

    pub async fn stop_session(&self, session_id: &str) -> Result<()> {
        self.session_manager.stop_session(session_id).await?;
        if let Some(task) = self.heartbeat_tasks.write().await.remove(session_id) {
            task.abort();
        }
        self.session_state_repo.set_online(session_id, false).await?;
        Ok(())
    }

    async fn start_heartbeat_task(&self, session_id: &str) {
        let session_id_owned = session_id.to_string();
        let session_state_repo = Arc::clone(&self.session_state_repo);
        let task_handle = tokio::spawn(async move {
            loop {
                if let Err(error) = session_state_repo.touch_heartbeat(&session_id_owned).await {
                    tracing::warn!("touch heartbeat failed for {}: {}", session_id_owned, error);
                }
                sleep(Duration::from_secs(10)).await;
            }
        });
        self.heartbeat_tasks
            .write()
            .await
            .insert(session_id.to_string(), task_handle);
    }
}
