use crate::admin::auth::require_permission;
use crate::admin::repository::AdminOverview;
use crate::admin::state::AdminState;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Serialize)]
struct ApiErrorResponse {
    error: String,
}

fn api_error(status: StatusCode, message: impl std::fmt::Display) -> Response {
    (
        status,
        Json(ApiErrorResponse {
            error: message.to_string(),
        }),
    )
        .into_response()
}

fn internal_error(err: impl std::fmt::Display) -> Response {
    api_error(StatusCode::INTERNAL_SERVER_ERROR, err)
}

fn bad_request(err: impl std::fmt::Display) -> Response {
    api_error(StatusCode::BAD_REQUEST, err)
}

fn not_found(err: impl std::fmt::Display) -> Response {
    api_error(StatusCode::NOT_FOUND, err)
}

fn is_session_not_found_error(err: &crate::error::WeChatBotError) -> bool {
    match err {
        crate::error::WeChatBotError::Other(message) => {
            message.contains("session") && message.contains("not found")
        }
        _ => false,
    }
}

fn format_dt_opt(dt: &Option<DateTime<Utc>>) -> String {
    dt.map(|timestamp| timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_default()
}

fn is_heartbeat_online(last_heartbeat_at: Option<DateTime<Utc>>, timeout_secs: u64) -> bool {
    match last_heartbeat_at {
        Some(timestamp) => Utc::now() - timestamp <= Duration::seconds(timeout_secs as i64),
        None => false,
    }
}

fn normalize_status_by_heartbeat(
    status: &str,
    last_heartbeat_at: Option<DateTime<Utc>>,
    timeout_secs: u64,
) -> String {
    let status_lower = status.to_ascii_lowercase();
    if status_lower == "online" && !is_heartbeat_online(last_heartbeat_at, timeout_secs) {
        "offline".to_string()
    } else {
        status.to_string()
    }
}

#[derive(Serialize)]
pub struct OverviewJson {
    pub total_bots: i64,
    pub online_bots: i64,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub messages_today: i64,
    pub forward_failures_today: i64,
}

impl From<AdminOverview> for OverviewJson {
    fn from(overview: AdminOverview) -> Self {
        Self {
            total_bots: overview.total_bots,
            online_bots: overview.online_bots,
            last_heartbeat_at: overview.last_heartbeat_at,
            messages_today: overview.messages_today,
            forward_failures_today: overview.forward_failures_today,
        }
    }
}

#[derive(Serialize)]
pub struct BotListRowJson {
    pub bot_id: String,
    pub status: String,
    pub is_online: bool,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub last_heartbeat_display: String,
    pub messages_today: i64,
    pub forward_failures_today: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct BotListJson {
    pub bots: Vec<BotListRowJson>,
}

#[derive(Serialize)]
pub struct BotSessionJson {
    pub session_id: String,
    pub user_id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct BotStatusJson {
    pub bot_id: String,
    pub status: String,
    pub is_online: bool,
    pub can_start: bool,
    pub has_qr_url: bool,
    pub heartbeat_display: String,
    pub start_action: Option<String>,
}

#[derive(Serialize)]
pub struct BotDetailJson {
    pub bot_id: String,
    pub status: String,
    pub is_online: bool,
    pub can_start: bool,
    pub has_runtime: bool,
    pub has_qr_url: bool,
    pub heartbeat_display: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub register_link: String,
    pub register_qr_image_url: Option<String>,
    pub sessions: Vec<BotSessionJson>,
}

#[derive(Serialize)]
pub struct CreateBotJson {
    pub bot_id: String,
    pub detail_api: String,
    pub register_link: String,
}

#[derive(Serialize)]
pub struct BotActionJson {
    pub bot_id: String,
    pub action: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct ForwardPolicyJson {
    pub bot_id: String,
    pub forwarding_enabled: bool,
    pub allowed_targets: Vec<String>,
}

#[derive(Deserialize)]
pub struct ForwardPolicyUpdateJson {
    pub forwarding_enabled: bool,
    pub allowed_targets: Vec<String>,
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    30
}

#[derive(Serialize)]
pub struct ChatMessageJson {
    pub received_at: DateTime<Utc>,
    pub from_user_id: String,
    pub to_user_id: String,
    pub content_type: String,
    pub text_content: String,
    pub direction: String,
}

#[derive(Serialize)]
pub struct SessionHistoryJson {
    pub session_id: String,
    pub bot_id: String,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
    pub rows: Vec<ChatMessageJson>,
}

#[derive(Deserialize)]
pub struct SystemLogQuery {
    #[serde(default = "default_system_log_lines")]
    pub lines: usize,
}

fn default_system_log_lines() -> usize {
    200
}

#[derive(Serialize)]
pub struct SystemLogJson {
    pub source: String,
    pub requested_lines: usize,
    pub returned_lines: usize,
    pub lines: Vec<String>,
}

fn build_bot_status_payload(
    bot_id: &str,
    db_status: &str,
    last_heartbeat_at: Option<DateTime<Utc>>,
    runtime_status: Option<crate::session::SessionStatus>,
    has_runtime: bool,
    has_qr_url: bool,
    timeout_secs: u64,
) -> BotStatusJson {
    let runtime_status_text = runtime_status.map(|status| format!("{status:?}"));
    let base_status = runtime_status_text.unwrap_or_else(|| db_status.to_string());
    let normalized_status = normalize_status_by_heartbeat(&base_status, last_heartbeat_at, timeout_secs);
    let is_online = normalized_status.eq_ignore_ascii_case("online");
    let can_start = has_runtime && !is_online;
    let start_action = if can_start {
        Some(format!(
            "/admin/api/bots/{}/start",
            urlencoding::encode(bot_id)
        ))
    } else {
        None
    };

    BotStatusJson {
        bot_id: bot_id.to_string(),
        status: normalized_status,
        is_online,
        can_start,
        has_qr_url,
        heartbeat_display: format_dt_opt(&last_heartbeat_at),
        start_action,
    }
}

pub async fn overview_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
) -> Result<Json<OverviewJson>, Response> {
    require_permission(&state, &headers, "bot.read", None).await?;
    let overview = state.repo.overview().await.map_err(internal_error)?;
    Ok(Json(OverviewJson::from(overview)))
}

pub async fn bot_list_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
) -> Result<Json<BotListJson>, Response> {
    require_permission(&state, &headers, "bot.read", None).await?;
    let rows = state.repo.list_bots().await.map_err(internal_error)?;
    let bots = rows
        .into_iter()
        .map(|row| {
            let normalized_status = normalize_status_by_heartbeat(
                &row.status,
                row.last_heartbeat_at,
                state.session_online_timeout_secs,
            );
            let is_online = normalized_status.eq_ignore_ascii_case("online");
            BotListRowJson {
                bot_id: row.bot_id,
                status: normalized_status,
                is_online,
                last_heartbeat_display: format_dt_opt(&row.last_heartbeat_at),
                last_heartbeat_at: row.last_heartbeat_at,
                messages_today: row.messages_today,
                forward_failures_today: row.forward_failures_today,
                updated_at: row.updated_at,
            }
        })
        .collect();
    Ok(Json(BotListJson { bots }))
}

pub async fn bot_detail_status_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<BotStatusJson>, Response> {
    require_permission(&state, &headers, "bot.read", Some(&bot_id)).await?;
    let bot = state.repo.get_bot(&bot_id).await.map_err(internal_error)?;
    let Some(bot) = bot else {
        return Err(not_found("bot not found"));
    };

    let sessions = state.repo.list_sessions(&bot_id).await.map_err(internal_error)?;
    let latest_session_id = sessions.first().map(|session| session.session_id.clone());
    let runtime_status = if let (Some(runtime), Some(session_id)) = (&state.runtime, &latest_session_id) {
        runtime.session_manager.status_of(session_id).await
    } else {
        None
    };

    let payload = build_bot_status_payload(
        &bot_id,
        &bot.status,
        bot.last_heartbeat_at,
        runtime_status,
        state.runtime.is_some(),
        state.qr_store.has_fresh(&bot_id, state.qr_expire_secs),
        state.session_online_timeout_secs,
    );
    Ok(Json(payload))
}

pub async fn bot_detail_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<BotDetailJson>, Response> {
    require_permission(&state, &headers, "bot.read", Some(&bot_id)).await?;
    let bot = state.repo.get_bot(&bot_id).await.map_err(internal_error)?;
    let Some(bot) = bot else {
        return Err(not_found("bot not found"));
    };

    let sessions = state.repo.list_sessions(&bot_id).await.map_err(internal_error)?;
    let latest_session_id = sessions.first().map(|session| session.session_id.clone());
    let runtime_status = if let (Some(runtime), Some(session_id)) = (&state.runtime, &latest_session_id) {
        runtime.session_manager.status_of(session_id).await
    } else {
        None
    };

    let register_qr_url = state.qr_store.get(&bot_id, state.qr_expire_secs).unwrap_or_default();
    let register_qr_image_url = if register_qr_url.is_empty() {
        None
    } else {
        Some(format!(
            "https://api.qrserver.com/v1/create-qr-code/?size=240x240&data={}",
            urlencoding::encode(&register_qr_url)
        ))
    };

    let status = build_bot_status_payload(
        &bot_id,
        &bot.status,
        bot.last_heartbeat_at,
        runtime_status,
        state.runtime.is_some(),
        state.qr_store.has_fresh(&bot_id, state.qr_expire_secs),
        state.session_online_timeout_secs,
    );
    let session_rows = sessions
        .into_iter()
        .map(|session| BotSessionJson {
            session_id: session.session_id,
            user_id: session.user_id,
            status: session.status,
            created_at: session.created_at,
            updated_at: session.updated_at,
        })
        .collect();
    Ok(Json(BotDetailJson {
        bot_id: bot.bot_id.clone(),
        status: status.status,
        is_online: status.is_online,
        can_start: status.can_start,
        has_runtime: state.runtime.is_some(),
        has_qr_url: status.has_qr_url,
        heartbeat_display: status.heartbeat_display,
        created_at: bot.created_at,
        updated_at: bot.updated_at,
        register_link: state.register_link(&bot.bot_id),
        register_qr_image_url,
        sessions: session_rows,
    }))
}

pub async fn bot_create_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
) -> Result<Response, Response> {
    require_permission(&state, &headers, "bot.write", None).await?;
    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(|| bad_request("runtime unavailable"))?;

    let bot_id = uuid::Uuid::new_v4().to_string();
    let qr_store = state.qr_store.clone();
    let bot_id_for_qr = bot_id.clone();
    let qr_callback = Box::new(move |url: &str| {
        qr_store.set(&bot_id_for_qr, url);
    });
    runtime
        .create_bot(&bot_id, qr_callback)
        .await
        .map_err(internal_error)?;

    let payload = CreateBotJson {
        detail_api: format!("/admin/api/bots/{}", urlencoding::encode(&bot_id)),
        register_link: state.register_link(&bot_id),
        bot_id,
    };
    Ok((StatusCode::CREATED, Json(payload)).into_response())
}

pub async fn bot_start_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<BotActionJson>, Response> {
    require_permission(&state, &headers, "bot.start_stop", Some(&bot_id)).await?;
    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(|| bad_request("runtime unavailable"))?;

    state.qr_store.remove(&bot_id);
    let qr_store = state.qr_store.clone();
    let bot_id_for_qr = bot_id.clone();
    let qr_callback = Box::new(move |url: &str| {
        qr_store.set(&bot_id_for_qr, url);
    });
    runtime
        .create_bot(&bot_id, qr_callback)
        .await
        .map_err(internal_error)?;

    Ok(Json(BotActionJson {
        bot_id,
        action: "start".to_string(),
        status: "accepted".to_string(),
    }))
}

pub async fn bot_stop_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<BotActionJson>, Response> {
    require_permission(&state, &headers, "bot.start_stop", Some(&bot_id)).await?;
    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(|| bad_request("runtime unavailable"))?;
    let sessions = state.repo.list_sessions(&bot_id).await.map_err(internal_error)?;
    if let Some(session) = sessions.first() {
        if let Err(error) = runtime.stop_session(&session.session_id).await {
            if !is_session_not_found_error(&error) {
                return Err(internal_error(error));
            }
        }
    }
    state
        .repo
        .set_bot_status(&bot_id, "offline")
        .await
        .map_err(internal_error)?;
    Ok(Json(BotActionJson {
        bot_id,
        action: "stop".to_string(),
        status: "accepted".to_string(),
    }))
}

pub async fn bot_delete_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<BotActionJson>, Response> {
    require_permission(&state, &headers, "bot.write", Some(&bot_id)).await?;
    let bot = state.repo.get_bot(&bot_id).await.map_err(internal_error)?;
    let Some(_) = bot else {
        return Err(not_found("bot not found"));
    };

    if let Some(runtime) = &state.runtime {
        let sessions = state.repo.list_sessions(&bot_id).await.map_err(internal_error)?;
        for session in sessions {
            runtime
                .stop_session(&session.session_id)
                .await
                .map_err(internal_error)?;
        }
    }
    state.qr_store.remove(&bot_id);
    state.repo.delete_bot_hard(&bot_id).await.map_err(internal_error)?;
    Ok(Json(BotActionJson {
        bot_id,
        action: "delete".to_string(),
        status: "accepted".to_string(),
    }))
}

pub async fn session_history_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<SessionHistoryJson>, Response> {
    require_permission(&state, &headers, "bot.read", None).await?;
    let session = state
        .repo
        .get_session(&session_id)
        .await
        .map_err(internal_error)?;
    let Some(session) = session else {
        return Err(not_found("session not found"));
    };

    require_permission(&state, &headers, "bot.read", Some(&session.bot_id)).await?;
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 200);
    let (rows, total) = state
        .repo
        .list_messages_page(&session_id, page, page_size)
        .await
        .map_err(internal_error)?;

    let total_pages = if total == 0 { 1 } else { total.div_ceil(page_size) };
    let rows = rows
        .into_iter()
        .map(|message| {
            let direction = if message.from_user_id == session.user_id {
                "out"
            } else {
                "in"
            };
            ChatMessageJson {
                received_at: message.received_at,
                from_user_id: message.from_user_id,
                to_user_id: message.to_user_id,
                content_type: message.content_type,
                text_content: message.text_content,
                direction: direction.to_string(),
            }
        })
        .collect();
    Ok(Json(SessionHistoryJson {
        session_id,
        bot_id: session.bot_id,
        page,
        page_size,
        total,
        total_pages,
        rows,
    }))
}

pub async fn forward_policy_get(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<ForwardPolicyJson>, Response> {
    require_permission(&state, &headers, "forward.read", Some(&bot_id)).await?;
    let policy = state.repo.get_forward_policy(&bot_id).await.map_err(internal_error)?;
    let policy = policy.unwrap_or(crate::admin::repository::BotForwardPolicy {
        bot_id: bot_id.clone(),
        forwarding_enabled: true,
        allowed_targets: vec!["webhook".to_string()],
    });
    Ok(Json(ForwardPolicyJson {
        bot_id: policy.bot_id,
        forwarding_enabled: policy.forwarding_enabled,
        allowed_targets: policy.allowed_targets,
    }))
}

pub async fn forward_policy_put(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    Json(payload): Json<ForwardPolicyUpdateJson>,
) -> Result<Json<ForwardPolicyJson>, Response> {
    require_permission(&state, &headers, "forward.write", Some(&bot_id)).await?;
    let filtered_targets: Vec<String> = payload
        .allowed_targets
        .into_iter()
        .map(|target| target.trim().to_string())
        .filter(|target| !target.is_empty())
        .collect();
    if filtered_targets.is_empty() {
        return Err(bad_request("allowed_targets cannot be empty"));
    }
    state
        .repo
        .upsert_forward_policy(&bot_id, payload.forwarding_enabled, &filtered_targets)
        .await
        .map_err(internal_error)?;
    Ok(Json(ForwardPolicyJson {
        bot_id,
        forwarding_enabled: payload.forwarding_enabled,
        allowed_targets: filtered_targets,
    }))
}

fn admin_log_path() -> PathBuf {
    if let Ok(path) = std::env::var("WECHATBOT_ADMIN_LOG_FILE") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".admin.log")
}

fn worker_log_path() -> PathBuf {
    if let Ok(path) = std::env::var("WECHATBOT_WORKER_LOG_FILE") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".worker.log")
}

fn tail_lines(content: &str, max_lines: usize) -> Vec<String> {
    if max_lines == 0 {
        return Vec::new();
    }
    let mut all_lines: Vec<&str> = content.lines().collect();
    if all_lines.len() <= max_lines {
        return all_lines
            .into_iter()
            .map(|line| line.to_string())
            .collect();
    }
    let start_index = all_lines.len() - max_lines;
    all_lines
        .split_off(start_index)
        .into_iter()
        .map(|line| line.to_string())
        .collect()
}

async fn read_system_log(path: &PathBuf, max_lines: usize) -> Result<Vec<String>, Response> {
    match fs::read_to_string(path).await {
        Ok(content) => Ok(tail_lines(&content, max_lines)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(internal_error(format!("read log failed: {error}"))),
    }
}

#[derive(Deserialize, Serialize)]
pub struct BotSendRequest {
    user_id: String,
    text: String,
}

#[derive(Serialize)]
pub struct BotSendResponse {
    bot_id: String,
    user_id: String,
    status: String,
}

pub async fn bot_send_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    Json(payload): Json<BotSendRequest>,
) -> Result<Json<BotSendResponse>, Response> {
    require_permission(&state, &headers, "bot.start_stop", Some(&bot_id)).await?;

    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(|| bad_request("runtime unavailable"))?;

    let reg = runtime.bot_registry.read().await;
    let bot = reg
        .get(&bot_id)
        .ok_or_else(|| bad_request(format!("bot {bot_id} not found in registry")))?
        .clone();
    drop(reg);

    bot.send(&payload.user_id, &payload.text)
        .await
        .map_err(|e| internal_error(format!("send failed: {e}")))?;

    Ok(Json(BotSendResponse {
        bot_id,
        user_id: payload.user_id,
        status: "accepted".to_string(),
    }))
}

pub async fn admin_system_logs_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Query(query): Query<SystemLogQuery>,
) -> Result<Json<SystemLogJson>, Response> {
    require_permission(&state, &headers, "bot.read", None).await?;
    let requested_lines = query.lines.clamp(1, 1000);
    let lines = read_system_log(&admin_log_path(), requested_lines).await?;
    Ok(Json(SystemLogJson {
        source: "admin".to_string(),
        requested_lines,
        returned_lines: lines.len(),
        lines,
    }))
}

pub async fn worker_system_logs_json(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Query(query): Query<SystemLogQuery>,
) -> Result<Json<SystemLogJson>, Response> {
    require_permission(&state, &headers, "bot.read", None).await?;
    let requested_lines = query.lines.clamp(1, 1000);
    let lines = read_system_log(&worker_log_path(), requested_lines).await?;
    Ok(Json(SystemLogJson {
        source: "worker".to_string(),
        requested_lines,
        returned_lines: lines.len(),
        lines,
    }))
}
