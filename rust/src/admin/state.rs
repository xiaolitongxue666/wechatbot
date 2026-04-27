use crate::admin::qr::QrUrlStore;
use crate::admin::repository::AdminRepository;
use crate::runtime::MultiBotRuntime;
use std::sync::Arc;

#[derive(Clone)]
pub struct AdminState {
    pub repo: AdminRepository,
    pub runtime: Option<Arc<MultiBotRuntime>>,
    pub qr_store: QrUrlStore,
    pub admin_host: String,
    pub admin_port: u16,
    pub session_online_timeout_secs: u64,
    pub qr_expire_secs: u64,
}

impl AdminState {
    pub fn register_link(&self, bot_id: &str) -> String {
        format!("http://{}:{}/bot/{}", self.admin_host, self.admin_port, bot_id)
    }
}
