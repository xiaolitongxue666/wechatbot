use crate::admin::handlers::{
    admin_system_logs_json, bot_create_json, bot_delete_json, bot_detail_json,
    bot_detail_status_json, bot_list_json, bot_register, bot_start_json, bot_stop_json,
    forward_policy_get, forward_policy_put, healthz, overview_json, root_redirect,
    session_history_json, worker_system_logs_json,
};
use crate::admin::qr::QrUrlStore;
use crate::admin::repository::AdminRepository;
use crate::admin::state::AdminState;
use crate::config::AppConfig;
use crate::error::Result;
use crate::runtime::MultiBotRuntime;
use crate::storage::postgres::PostgresChatRepository;
use axum::http::StatusCode;
use axum::routing::{any, get, post};
use axum::Router;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use tower_http::trace::TraceLayer;

pub fn admin_router(pool: PgPool) -> Router {
    admin_router_with_runtime(
        pool,
        None,
        QrUrlStore::new(),
        "dev-admin-token".to_string(),
        "127.0.0.1".to_string(),
        8787,
        3600,
        3600,
    )
}

pub fn admin_router_with_runtime(
    pool: PgPool,
    runtime: Option<Arc<MultiBotRuntime>>,
    qr_store: QrUrlStore,
    default_admin_api_token: String,
    admin_host: String,
    admin_port: u16,
    session_online_timeout_secs: u64,
    qr_expire_secs: u64,
) -> Router {
    let state = AdminState {
        repo: AdminRepository::new(pool, session_online_timeout_secs as i64),
        runtime,
        qr_store,
        default_admin_api_token,
        admin_host,
        admin_port,
        session_online_timeout_secs,
        qr_expire_secs,
    };
    let web_admin_dist_dir = web_admin_dist_dir();
    let admin_api = Router::new()
        .route("/overview", get(overview_json))
        .route("/bots", get(bot_list_json).post(bot_create_json))
        .route("/bots/{bot_id}", get(bot_detail_json).delete(bot_delete_json))
        .route("/bots/{bot_id}/start", post(bot_start_json))
        .route("/bots/{bot_id}/stop", post(bot_stop_json))
        .route("/bots/{bot_id}/status", get(bot_detail_status_json))
        .route(
            "/bots/{bot_id}/forward-policy",
            get(forward_policy_get).put(forward_policy_put),
        )
        .route("/sessions/{session_id}/history", get(session_history_json))
        .route("/system-logs/admin", get(admin_system_logs_json))
        .route("/system-logs/worker", get(worker_system_logs_json))
        .fallback(|| async { StatusCode::NOT_FOUND })
        .with_state(state.clone());
    let admin_spa = ServeDir::new(web_admin_dist_dir.clone())
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(web_admin_dist_dir.join("index.html")));
    let admin = Router::new()
        .nest("/api", admin_api)
        .route("/api/{*path}", any(|| async { StatusCode::NOT_FOUND }))
        .fallback_service(admin_spa);

    let public = Router::new()
        .route("/bot/{bot_id}", get(bot_register))
        .with_state(state);

    Router::new()
        .route("/", get(root_redirect))
        .route("/healthz", get(healthz))
        .nest("/admin", admin)
        .merge(public)
        .layer(TraceLayer::new_for_http())
}

pub async fn run_admin_server(config: AppConfig) -> Result<()> {
    let database_url = config.database_url()?.to_string();
    let pool = PostgresChatRepository::connect(&database_url)
        .await?
        .pool()
        .clone();
    let bootstrap_repo = AdminRepository::new(pool.clone(), config.runtime.session_online_timeout_secs as i64);
    bootstrap_repo
        .upsert_admin_user("admin-default", "Default Admin", "admin", &config.admin.api_token)
        .await?;
    let runtime = Arc::new(MultiBotRuntime::from_config(config.clone()).await?);
    let bind = std::env::var("WECHATBOT_ADMIN_BIND").unwrap_or(config.admin.bind.clone());
    let (host, port) = parse_bind(&bind);
    let app = admin_router_with_runtime(
        pool,
        Some(runtime),
        QrUrlStore::new(),
        config.admin.api_token.clone(),
        host,
        port,
        config.runtime.session_online_timeout_secs,
        config.runtime.qr_expire_secs,
    );
    serve_bind(app, &bind).await
}

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

fn web_admin_dist_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("WECHATBOT_WEB_ADMIN_DIST_DIR") {
        return PathBuf::from(dir);
    }
    // 重组后默认 admin/web/dist（原 web-admin/dist）；集成测试 admin_frontend 依赖此目录已构建
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("admin/web/dist")
}

fn parse_bind(bind: &str) -> (String, u16) {
    let default_port = 8787;
    if let Some((host, port_str)) = bind.rsplit_once(':') {
        let port = port_str.parse::<u16>().unwrap_or(default_port);
        (host.to_string(), port)
    } else {
        (bind.to_string(), default_port)
    }
}
