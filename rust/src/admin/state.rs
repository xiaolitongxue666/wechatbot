use crate::admin::repository::AdminRepository;

#[derive(Clone)]
pub struct AdminState {
    pub repo: AdminRepository,
}
