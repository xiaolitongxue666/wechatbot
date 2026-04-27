use crate::bot::{BotOptions, WeChatBot};
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
use tracing::{info, warn};
use uuid::Uuid;

pub struct MultiBotRuntime {
    pub config: AppConfig,
    pub session_manager: Arc<BotSessionManager>,
    pub session_state_repo: Arc<dyn SessionStateRepository>,
    pub chat_repository: Arc<dyn ChatRepository>,
    pub ingestor: Arc<MessageIngestor>,
    pub forwarder: Arc<ForwarderWorker>,
    pub bot_registry: Arc<RwLock<HashMap<String, Arc<WeChatBot>>>>,
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
            bot_registry: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_tasks: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn create_bot(
        &self,
        bot_id: &str,
        qr_callback: Box<dyn Fn(&str) + Send + Sync>,
    ) -> Result<()> {
        self.chat_repository
            .upsert_bot(bot_id, "pending_qr")
            .await?;

        let bot_id_owned = bot_id.to_string();
        let runtime = self.session_manager.clone();
        let bot_registry = self.bot_registry.clone();
        let chat_repository = self.chat_repository.clone();
        let ingestor = self.ingestor.clone();
        let session_state_repo = self.session_state_repo.clone();
        let heartbeat_tasks = self.heartbeat_tasks.clone();
        let qr_callback: Arc<dyn Fn(&str) + Send + Sync> = Arc::from(qr_callback);

        tokio::spawn(async move {
            loop {
                let cb = qr_callback.clone();
                let reg_bot = Arc::new(WeChatBot::new(BotOptions {
                    on_qr_url: Some(Box::new(move |url: &str| cb(url))),
                    ..Default::default()
                }));

                let credentials = match reg_bot.login(true).await {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("registration login failed for {}: {}", bot_id_owned, e);
                        chat_repository.upsert_bot(&bot_id_owned, "offline").await.ok();
                        return;
                    }
                };

                chat_repository.upsert_bot(&bot_id_owned, "online").await.ok();

                let session_id = Uuid::new_v4().to_string();
                let user_id = credentials.user_id.clone();

                chat_repository
                    .create_session(&session_id, &bot_id_owned, &user_id)
                    .await
                    .ok();

                let session_bot = Arc::new(WeChatBot::new(BotOptions::default()));
                session_bot.set_credentials(&credentials).await;

                let sid = session_id.clone();
                let bid = bot_id_owned.clone();
                {
                    let mut reg = bot_registry.write().await;
                    reg.insert(bot_id_owned.clone(), session_bot.clone());
                }

                let bot_for_ingest = session_bot.clone();
                let ingestor_clone = ingestor.clone();
                let bid2 = bid.clone();
                let sid2 = sid.clone();
                let uid2 = user_id.clone();
                session_bot.on_message(Box::new(move |message| {
                    let message = message.clone();
                    let bot = bot_for_ingest.clone();
                    let ing = ingestor_clone.clone();
                    let b = bid2.clone();
                    let s = sid2.clone();
                    let u = uid2.clone();
                    tokio::spawn(async move {
                        if let Err(error) = ing.ingest(bot.clone(), &b, &s, &message).await {
                            tracing::error!("ingest failed: {}", error);
                        }
                        let echo = format!("Echo_{}", message.text);
                        if let Err(e) = bot.reply(&message, &echo).await {
                            tracing::error!("echo reply failed: {}", e);
                        }
                        if let Err(e) = ing.ingest_sent(&s, &b, &u, &message.user_id, &echo).await {
                            tracing::error!("ingest_sent failed: {}", e);
                        }
                    });
                })).await;

                runtime
                    .register_session(BotSession {
                        session_id: sid.clone(),
                        bot_id: bid.clone(),
                        user_id: user_id.clone(),
                        bot: session_bot.clone(),
                    })
                    .await
                    .ok();

                let sid = session_id.clone();
                let runtime_clone = runtime.clone();
                let session_state_repo_clone = session_state_repo.clone();
                let heartbeat_tasks_clone = heartbeat_tasks.clone();

                tokio::spawn(async move {
                    let _ = runtime_clone.start_session(&sid, false).await;
                    session_state_repo_clone.set_online(&sid, true).await.ok();
                    let task_handle = tokio::spawn({
                        let repo = session_state_repo_clone.clone();
                        let s = sid.clone();
                        async move {
                            loop {
                                if let Err(error) = repo.touch_heartbeat(&s).await {
                                    tracing::warn!("touch heartbeat failed for {}: {}", s, error);
                                }
                                sleep(Duration::from_secs(10)).await;
                            }
                        }
                    });
                    heartbeat_tasks_clone
                        .write()
                        .await
                        .insert(sid.clone(), task_handle);
                });

                info!(
                    "session {} created for bot {} user {}",
                    session_id, bot_id_owned, user_id,
                );
            }
        });

        Ok(())
    }

    pub async fn start_session(&self, session_id: &str, force_login: bool) -> Result<()> {
        self.session_manager.start_session(session_id, force_login).await?;
        self.session_state_repo.set_online(session_id, true).await?;
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
}
