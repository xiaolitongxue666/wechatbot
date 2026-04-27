use crate::admin::handlers::{bot_detail, bot_history, bot_list, dashboard, healthz, overview_json, root_redirect};
use crate::admin::repository::AdminRepository;
use crate::admin::state::AdminState;
use crate::config::AppConfig;
use crate::error::Result;
use crate::storage::postgres::PostgresChatRepository;
use axum::routing::get;
use axum::Router;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::signal;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

/// Build router with an existing pool (for tests).
pub fn admin_router(pool: PgPool) -> Router {
    let state = AdminState {
        repo: AdminRepository::new(pool),
    };
    let static_dir = static_dir();
    let admin = Router::new()
        .route("/", get(dashboard))
        .route("/bots", get(bot_list))
        .route("/bots/{session_id}", get(bot_detail))
        .route("/bots/{session_id}/history", get(bot_history))
        .route("/api/overview", get(overview_json))
        .with_state(state);

    Router::new()
        .route("/", get(root_redirect))
        .route("/healthz", get(healthz))
        .nest("/admin", admin)
        .nest_service("/static", ServeDir::new(static_dir))
        .layer(TraceLayer::new_for_http())
}

pub async fn run_admin_server(config: AppConfig) -> Result<()> {
    let database_url = config.database_url()?.to_string();
    let pool = PostgresChatRepository::connect(&database_url)
        .await?
        .pool()
        .clone();
    let app = admin_router(pool);
    let bind = std::env::var("WECHATBOT_ADMIN_BIND").unwrap_or(config.admin.bind.clone());
    serve_bind(app, &bind).await
}

/// Serve admin using an existing Postgres pool (skips opening a new connection from config).
pub async fn run_admin_repository_pool(pool: PgPool, bind: &str) -> Result<()> {
    let app = admin_router(pool);
    serve_bind(app, bind).await
}

async fn serve_bind(app: Router, bind: &str) -> Result<()> {
    let addr: SocketAddr = bind.parse().map_err(|e| {
        crate::error::WeChatBotError::Other(format!("invalid admin bind {bind}: {e}"))
    })?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::error::WeChatBotError::Other(format!("bind {addr}: {e}")))?;
    tracing::info!("admin listening on http://{addr}");

    let shutdown = shutdown_signal();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(|e| crate::error::WeChatBotError::Other(format!("admin server: {e}")))?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("received Ctrl+C, shutting down gracefully");
        }
        _ = terminate => {
            tracing::info!("received SIGTERM, shutting down gracefully");
        }
    }
}

fn static_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("WECHATBOT_ADMIN_STATIC_DIR") {
        return PathBuf::from(dir);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")
}
