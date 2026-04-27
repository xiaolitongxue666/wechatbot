//! HTTP admin dashboard (Axum + Askama).

mod handlers;
pub mod qr;
pub mod repository;
mod server;
mod state;
mod ui;

pub use server::{admin_router, admin_router_with_runtime, run_admin_repository_pool, run_admin_server};
