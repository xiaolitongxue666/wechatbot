use crate::bot::WeChatBot;
use crate::error::{Result, WeChatBotError};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    PendingQr,
    WaitingConfirm,
    Online,
    Expired,
    Offline,
}

#[derive(Clone)]
pub struct BotSession {
    pub session_id: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub bot: Arc<WeChatBot>,
}

struct SessionRuntime {
    status: SessionStatus,
    task_handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

pub struct BotSessionManager {
    sessions: Arc<RwLock<HashMap<String, BotSession>>>,
    runtimes: Arc<RwLock<HashMap<String, SessionRuntime>>>,
}

impl Default for BotSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BotSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            runtimes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_session(&self, session: BotSession) -> Result<()> {
        let session_id = session.session_id.clone();
        self.sessions.write().await.insert(session_id.clone(), session);
        self.runtimes.write().await.insert(
            session_id,
            SessionRuntime {
                status: SessionStatus::Offline,
                task_handle: None,
                stop_flag: Arc::new(AtomicBool::new(false)),
            },
        );
        Ok(())
    }

    pub async fn start_session(&self, session_id: &str, force_login: bool) -> Result<()> {
        let session = self
            .sessions
            .read()
            .await
            .get(session_id)
            .cloned()
            .ok_or_else(|| WeChatBotError::Other(format!("session {session_id} not found")))?;

        let session_id_owned = session_id.to_string();
        let runtimes = Arc::clone(&self.runtimes);
        let bot = Arc::clone(&session.bot);
        let stop_flag = {
            let mut runtimes_guard = self.runtimes.write().await;
            let runtime = runtimes_guard
                .get_mut(session_id)
                .ok_or_else(|| WeChatBotError::Other(format!("session {session_id} runtime not found")))?;
            runtime.stop_flag.store(false, Ordering::SeqCst);
            Arc::clone(&runtime.stop_flag)
        };
        let handle = tokio::spawn(async move {
            let mut should_force_login = force_login;
            let mut reconnect_delay = Duration::from_secs(1);
            loop {
                if stop_flag.load(Ordering::SeqCst) {
                    update_status(&runtimes, &session_id_owned, SessionStatus::Offline).await;
                    break;
                }

                update_status(&runtimes, &session_id_owned, SessionStatus::PendingQr).await;
                if let Err(error) = bot.login(should_force_login).await {
                    warn!("session {} login failed: {}", session_id_owned, error);
                    update_status(&runtimes, &session_id_owned, SessionStatus::Expired).await;
                    sleep(reconnect_delay).await;
                    reconnect_delay = std::cmp::min(reconnect_delay * 2, Duration::from_secs(10));
                    should_force_login = true;
                    continue;
                }
                update_status(&runtimes, &session_id_owned, SessionStatus::Online).await;

                if let Err(error) = bot.run().await {
                    warn!("session {} run stopped with error: {}", session_id_owned, error);
                    update_status(&runtimes, &session_id_owned, SessionStatus::Expired).await;
                    sleep(reconnect_delay).await;
                    reconnect_delay = std::cmp::min(reconnect_delay * 2, Duration::from_secs(10));
                    should_force_login = true;
                    continue;
                }

                if stop_flag.load(Ordering::SeqCst) {
                    update_status(&runtimes, &session_id_owned, SessionStatus::Offline).await;
                    break;
                }
            }
        });
        self.runtimes
            .write()
            .await
            .entry(session_id.to_string())
            .and_modify(|runtime| runtime.task_handle = Some(handle));
        info!("session {} started", session_id);
        Ok(())
    }

    pub async fn stop_session(&self, session_id: &str) -> Result<()> {
        let session = self
            .sessions
            .read()
            .await
            .get(session_id)
            .cloned()
            .ok_or_else(|| WeChatBotError::Other(format!("session {session_id} not found")))?;
        if let Some(runtime) = self.runtimes.write().await.get_mut(session_id) {
            runtime.stop_flag.store(true, Ordering::SeqCst);
        }
        session.bot.stop().await;
        self.set_status(session_id, SessionStatus::Offline).await;
        Ok(())
    }

    pub async fn status_of(&self, session_id: &str) -> Option<SessionStatus> {
        self.runtimes.read().await.get(session_id).map(|runtime| runtime.status)
    }

    async fn set_status(&self, session_id: &str, status: SessionStatus) {
        self.runtimes
            .write()
            .await
            .entry(session_id.to_string())
            .and_modify(|runtime| runtime.status = status);
    }
}

async fn update_status(
    runtimes: &Arc<RwLock<HashMap<String, SessionRuntime>>>,
    session_id: &str,
    status: SessionStatus,
) {
    runtimes
        .write()
        .await
        .entry(session_id.to_string())
        .and_modify(|runtime| runtime.status = status);
}
