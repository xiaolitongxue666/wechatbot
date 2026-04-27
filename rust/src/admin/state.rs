use crate::admin::qr::QrUrlStore;
use crate::admin::repository::AdminRepository;
use crate::runtime::MultiBotRuntime;
use std::sync::Arc;

#[derive(Clone)]
pub struct AdminState {
    pub repo: AdminRepository,
    pub runtime: Option<Arc<MultiBotRuntime>>,
    pub qr_store: QrUrlStore,
}
