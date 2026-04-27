use crate::admin::repository::AdminOverview;
use crate::admin::state::AdminState;
use crate::admin::ui::{i18n, I18n, UiPrefs, UiQuery};
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Json;
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

fn format_dt_opt(dt: &Option<DateTime<Utc>>) -> String {
    dt.map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "—".to_string())
}

fn format_dt(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn internal_error(err: impl std::fmt::Display) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
}

fn bad_request(err: impl std::fmt::Display) -> Response {
    (StatusCode::BAD_REQUEST, err.to_string()).into_response()
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

pub async fn root_redirect() -> Redirect {
    Redirect::temporary("/admin")
}

pub async fn healthz() -> &'static str {
    "ok"
}

#[derive(Serialize)]
pub struct OverviewJson {
    pub total_bots: i64,
    pub online_bots: i64,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub messages_today: i64,
    pub forward_dlq_count: i64,
    pub forward_not_success_count: i64,
}

impl From<AdminOverview> for OverviewJson {
    fn from(o: AdminOverview) -> Self {
        Self {
            total_bots: o.total_bots,
            online_bots: o.online_bots,
            last_heartbeat_at: o.last_heartbeat_at,
            messages_today: o.messages_today,
            forward_dlq_count: o.forward_dlq_count,
            forward_not_success_count: o.forward_not_success_count,
        }
    }
}

pub async fn overview_json(
    State(state): State<AdminState>,
) -> Result<Json<OverviewJson>, Response> {
    let overview = state.repo.overview().await.map_err(internal_error)?;
    Ok(Json(OverviewJson::from(overview)))
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct DashboardTpl<'a> {
    i18n: I18n,
    prefs: &'a UiPrefs,
    lang_attr: &'a str,
    nav_active: &'a str,
    overview: &'a AdminOverview,
    last_heartbeat_display: String,
}

pub async fn dashboard(
    State(state): State<AdminState>,
    Query(q): Query<UiQuery>,
    jar: CookieJar,
) -> Result<Response, Response> {
    let prefs = UiPrefs::resolve(q.clone(), &jar);
    let jar = UiPrefs::apply_cookies_from_query(&q, jar);
    let overview = state.repo.overview().await.map_err(internal_error)?;
    let i = i18n(&prefs.lang);
    let last_hb = format_dt_opt(&overview.last_heartbeat_at);
    let page = DashboardTpl {
        i18n: i,
        prefs: &prefs,
        lang_attr: if prefs.lang == "en" { "en" } else { "zh-CN" },
        nav_active: "dash",
        overview: &overview,
        last_heartbeat_display: last_hb,
    };
    let html = page.render().map_err(internal_error)?;
    Ok((jar, Html(html)).into_response())
}

struct BotListRowVm {
    bot_id: String,
    status: String,
    last_heartbeat: String,
    updated_at: String,
}

#[derive(Template)]
#[template(path = "admin/bot_list.html")]
struct BotListTpl<'a> {
    i18n: I18n,
    prefs: &'a UiPrefs,
    lang_attr: &'a str,
    nav_active: &'a str,
    rows: &'a [BotListRowVm],
    has_runtime: bool,
}

pub async fn bot_list(
    State(state): State<AdminState>,
    Query(q): Query<UiQuery>,
    jar: CookieJar,
) -> Result<Response, Response> {
    let prefs = UiPrefs::resolve(q.clone(), &jar);
    let jar = UiPrefs::apply_cookies_from_query(&q, jar);
    let rows_db = state.repo.list_bots().await.map_err(internal_error)?;
    let has_runtime = state.runtime.is_some();
    let mut rows = Vec::with_capacity(rows_db.len());
    for row in rows_db {
        let effective_status = normalize_status_by_heartbeat(
            &row.status,
            row.last_heartbeat_at,
            state.session_online_timeout_secs,
        );
        rows.push(BotListRowVm {
            bot_id: row.bot_id,
            status: effective_status,
            last_heartbeat: format_dt_opt(&row.last_heartbeat_at),
            updated_at: format_dt(row.updated_at),
        });
    }
    let i = i18n(&prefs.lang);
    let page = BotListTpl {
        i18n: i,
        prefs: &prefs,
        lang_attr: if prefs.lang == "en" { "en" } else { "zh-CN" },
        nav_active: "bots",
        rows: &rows,
        has_runtime,
    };
    let html = page.render().map_err(internal_error)?;
    Ok((jar, Html(html)).into_response())
}

struct SessionRowVm {
    session_id: String,
    user_id: String,
    status: String,
    created_at: String,
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

#[derive(Template)]
#[template(path = "admin/bot_detail.html")]
struct BotDetailTpl<'a> {
    i18n: I18n,
    prefs: &'a UiPrefs,
    lang_attr: &'a str,
    nav_active: &'a str,
    bot_id: &'a str,
    bot_status: &'a str,
    heartbeat_display: String,
    created_display: String,
    updated_display: String,
    qr_url: String,
    qr_image_url: String,
    register_link: String,
    has_runtime: bool,
    is_online: bool,
    can_start: bool,
    start_action: Option<String>,
    sessions: &'a [SessionRowVm],
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
    let status_from_runtime = runtime_status.map(|status| format!("{:?}", status));
    let base_status = status_from_runtime.unwrap_or_else(|| db_status.to_string());
    let normalized_status = normalize_status_by_heartbeat(&base_status, last_heartbeat_at, timeout_secs);
    let is_online = normalized_status.eq_ignore_ascii_case("online");
    let can_start = has_runtime && !is_online;
    let start_action = if can_start {
        Some(format!("/admin/bots/{}/start", urlencoding::encode(bot_id)))
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

pub async fn bot_detail_status_json(
    State(state): State<AdminState>,
    Path(bot_id): Path<String>,
) -> Result<Json<BotStatusJson>, Response> {
    let bot = state.repo.get_bot(&bot_id).await.map_err(internal_error)?;
    let Some(bot) = bot else {
        return Err((StatusCode::NOT_FOUND, "bot not found".to_string()).into_response());
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

pub async fn bot_detail(
    State(state): State<AdminState>,
    Path(bot_id): Path<String>,
    Query(q): Query<UiQuery>,
    jar: CookieJar,
) -> Result<Response, Response> {
    let prefs = UiPrefs::resolve(q.clone(), &jar);
    let jar = UiPrefs::apply_cookies_from_query(&q, jar);
    let bot = state.repo.get_bot(&bot_id).await.map_err(internal_error)?;
    let Some(bot) = bot else {
        let i = i18n(&prefs.lang);
        return Ok((
            jar,
            (StatusCode::NOT_FOUND, Html(not_found_html(&i, &prefs))),
        ).into_response());
    };
    let i = i18n(&prefs.lang);

    let qr_url = state.qr_store.get(&bot_id, state.qr_expire_secs);
    let qr_image_url = qr_url
        .as_ref()
        .map(|url| {
            format!(
                "https://api.qrserver.com/v1/create-qr-code/?size=220x220&data={}",
                urlencoding::encode(url)
            )
        })
        .unwrap_or_default();

    let register_link = state.register_link(&bot_id);

    let sessions_db = state.repo.list_sessions(&bot_id).await.map_err(internal_error)?;
    let latest_session_id = sessions_db.first().map(|session| session.session_id.clone());
    let runtime_status = if let (Some(runtime), Some(session_id)) = (&state.runtime, &latest_session_id) {
        runtime.session_manager.status_of(session_id).await
    } else {
        None
    };

    let status_payload = build_bot_status_payload(
        &bot_id,
        &bot.status,
        bot.last_heartbeat_at,
        runtime_status,
        state.runtime.is_some(),
        qr_url.is_some(),
        state.session_online_timeout_secs,
    );
    let sessions: Vec<SessionRowVm> = sessions_db
        .iter()
        .map(|s| SessionRowVm {
            session_id: s.session_id.clone(),
            user_id: s.user_id.clone(),
            status: s.status.clone(),
            created_at: format_dt(s.created_at),
        })
        .collect();

    let page = BotDetailTpl {
        i18n: i,
        prefs: &prefs,
        lang_attr: if prefs.lang == "en" { "en" } else { "zh-CN" },
        nav_active: "bots",
        bot_id: &bot_id,
        bot_status: &status_payload.status,
        heartbeat_display: status_payload.heartbeat_display.clone(),
        created_display: format_dt(bot.created_at),
        updated_display: format_dt(bot.updated_at),
        qr_url: qr_url.unwrap_or_default(),
        qr_image_url,
        register_link,
        has_runtime: state.runtime.is_some(),
        is_online: status_payload.is_online,
        can_start: status_payload.can_start,
        start_action: status_payload.start_action.clone(),
        sessions: &sessions,
    };
    let html = page.render().map_err(internal_error)?;
    Ok((jar, Html(html)).into_response())
}

#[derive(Template)]
#[template(path = "admin/bot_create.html")]
struct BotCreateTpl<'a> {
    i18n: I18n,
    prefs: &'a UiPrefs,
    lang_attr: &'a str,
    nav_active: &'a str,
    has_runtime: bool,
}

pub async fn bot_create_form(
    State(state): State<AdminState>,
    Query(q): Query<UiQuery>,
    jar: CookieJar,
) -> Result<Response, Response> {
    let prefs = UiPrefs::resolve(q.clone(), &jar);
    let jar = UiPrefs::apply_cookies_from_query(&q, jar);
    let has_runtime = state.runtime.is_some();
    let i = i18n(&prefs.lang);
    let page = BotCreateTpl {
        i18n: i,
        prefs: &prefs,
        lang_attr: if prefs.lang == "en" { "en" } else { "zh-CN" },
        nav_active: "bots",
        has_runtime,
    };
    let html = page.render().map_err(internal_error)?;
    Ok((jar, Html(html)).into_response())
}

pub async fn bot_create_submit(
    State(state): State<AdminState>,
) -> Result<Response, Response> {
    let prefs = UiPrefs::default();
    let i = i18n(&prefs.lang);

    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(|| bad_request(i.no_runtime))?;

    let bot_id = uuid::Uuid::new_v4().to_string();

    let qr_store = state.qr_store.clone();
    let bid_for_qr = bot_id.clone();
    let qr_callback = Box::new(move |url: &str| {
        qr_store.set(&bid_for_qr, url);
    });

    runtime
        .create_bot(&bot_id, qr_callback)
        .await
        .map_err(internal_error)?;

    let detail_url = format!("/admin/bots/{}", urlencoding::encode(&bot_id));
    Ok(Redirect::to(&detail_url).into_response())
}

pub async fn bot_start(
    State(state): State<AdminState>,
    Path(bot_id): Path<String>,
) -> Result<Response, Response> {
    let i = i18n("zh");
    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(|| bad_request(i.no_runtime))?;

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

    let back_url = format!("/admin/bots/{}", urlencoding::encode(&bot_id));
    Ok(Redirect::to(&back_url).into_response())
}

pub async fn bot_stop(
    State(state): State<AdminState>,
    Path(bot_id): Path<String>,
) -> Result<Response, Response> {
    let i = i18n("zh");
    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(|| bad_request(i.no_runtime))?;

    let sessions = state.repo.list_sessions(&bot_id).await.map_err(internal_error)?;
    if let Some(session) = sessions.first() {
        runtime
            .stop_session(&session.session_id)
            .await
            .map_err(internal_error)?;
    }

    let back_url = format!("/admin/bots/{}", urlencoding::encode(&bot_id));
    Ok(Redirect::to(&back_url).into_response())
}

pub async fn bot_delete(
    State(state): State<AdminState>,
    Path(bot_id): Path<String>,
    Query(q): Query<UiQuery>,
    jar: CookieJar,
) -> Result<Response, Response> {
    let prefs = UiPrefs::resolve(q.clone(), &jar);
    let jar = UiPrefs::apply_cookies_from_query(&q, jar);

    let bot = state.repo.get_bot(&bot_id).await.map_err(internal_error)?;
    let Some(_) = bot else {
        let i = i18n(&prefs.lang);
        return Ok((
            jar,
            (StatusCode::NOT_FOUND, Html(not_found_html(&i, &prefs))),
        )
            .into_response());
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
    state
        .repo
        .delete_bot_hard(&bot_id)
        .await
        .map_err(internal_error)?;

    let list_url = format!("/admin/bots?{}", prefs.query_suffix());
    Ok((jar, Redirect::to(&list_url)).into_response())
}

fn not_found_html(i18n: &I18n, prefs: &UiPrefs) -> String {
    format!(
        r#"<!DOCTYPE html><html lang="{}"><head><meta charset="utf-8"><title>404</title>
        <link rel="stylesheet" href="/static/admin/style.css"></head>
        <body class="theme-{}"><p class="muted">{}</p>
        <p><a href="/admin/bots?{}">{}</a></p></body></html>"#,
        if prefs.lang == "en" { "en" } else { "zh-CN" },
        prefs.theme,
        i18n.bot_not_found,
        prefs.query_suffix(),
        i18n.nav_bots,
    )
}

#[cfg(test)]
mod tests {
    use super::build_bot_status_payload;
    use chrono::Utc;

    #[test]
    fn build_status_allows_start_when_offline_and_runtime_available() {
        let payload = build_bot_status_payload("bot-offline", "offline", None, None, true, false, 3600);
        assert!(payload.can_start);
        assert_eq!(payload.start_action.as_deref(), Some("/admin/bots/bot-offline/start"));
    }

    #[test]
    fn build_status_disallows_start_when_online() {
        let payload = build_bot_status_payload("bot-online", "online", Some(Utc::now()), None, true, false, 3600);
        assert!(!payload.can_start);
        assert!(payload.start_action.is_none());
    }

    #[test]
    fn build_status_disallows_start_without_runtime() {
        let payload = build_bot_status_payload("bot-nort", "offline", None, None, false, false, 3600);
        assert!(!payload.can_start);
        assert!(payload.start_action.is_none());
    }
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(flatten)]
    pub ui: UiQuery,
}

fn default_page() -> u64 {
    1
}

struct ChatMessageVm {
    received_at: String,
    from_user_id: String,
    to_user_id: String,
    content_type: String,
    text_content: String,
    direction: String,
}

#[derive(Template)]
#[template(path = "admin/history.html")]
struct HistoryTpl<'a> {
    i18n: I18n,
    prefs: &'a UiPrefs,
    lang_attr: &'a str,
    nav_active: &'a str,
    session_id: &'a str,
    rows: &'a [ChatMessageVm],
    page: u64,
    page_size: u64,
    total: u64,
    total_pages: u64,
    has_prev: bool,
    has_next: bool,
    prev_qs: String,
    next_qs: String,
}

pub async fn bot_history(
    State(state): State<AdminState>,
    Path(session_id): Path<String>,
    Query(hq): Query<HistoryQuery>,
    jar: CookieJar,
) -> Result<Response, Response> {
    let ui = hq.ui;
    let prefs = UiPrefs::resolve(ui.clone(), &jar);
    let jar = UiPrefs::apply_cookies_from_query(&ui, jar);
    let session_row = state
        .repo
        .get_session(&session_id)
        .await
        .map_err(internal_error)?;
    if session_row.is_none() {
        let i = i18n(&prefs.lang);
        return Ok((
            jar,
            (StatusCode::NOT_FOUND, Html(not_found_html(&i, &prefs))),
        ).into_response());
    }
    let bot_user_id = session_row.as_ref().map(|s| s.user_id.clone()).unwrap_or_default();

    const PAGE_SIZE: u64 = 30;
    let page = hq.page.max(1);
    let (rows_db, total) = state
        .repo
        .list_messages_page(&session_id, page, PAGE_SIZE)
        .await
        .map_err(internal_error)?;
    let rows: Vec<ChatMessageVm> = rows_db
        .into_iter()
        .map(|m| {
            let direction = if m.from_user_id == bot_user_id {
                "out"
            } else {
                "in"
            };
            ChatMessageVm {
                received_at: format_dt(m.received_at),
                from_user_id: m.from_user_id,
                to_user_id: m.to_user_id,
                content_type: m.content_type,
                text_content: m.text_content,
                direction: direction.to_string(),
            }
        })
        .collect();
    let total_pages = if total == 0 {
        1
    } else {
        total.div_ceil(PAGE_SIZE)
    };
    let has_prev = page > 1;
    let has_next = page < total_pages;
    let base_ui = prefs.query_suffix();
    let prev_qs = format!("page={}&{}", page.saturating_sub(1).max(1), base_ui);
    let next_qs = format!("page={}&{}", page.saturating_add(1), base_ui);
    let i = i18n(&prefs.lang);
    let page_tpl = HistoryTpl {
        i18n: i,
        prefs: &prefs,
        lang_attr: if prefs.lang == "en" { "en" } else { "zh-CN" },
        nav_active: "bots",
        session_id: &session_id,
        rows: &rows,
        page,
        page_size: PAGE_SIZE,
        total,
        total_pages,
        has_prev,
        has_next,
        prev_qs,
        next_qs,
    };
    let html = page_tpl.render().map_err(internal_error)?;
    Ok((jar, Html(html)).into_response())
}

// Public registration page

#[derive(Template)]
#[template(path = "bot_register.html")]
struct BotRegisterTpl {
    bot_id: String,
    qr_url: String,
    qr_image_url: String,
}

pub async fn bot_register(
    State(state): State<AdminState>,
    Path(bot_id): Path<String>,
) -> Result<Response, Response> {
    let qr_url = state.qr_store.get(&bot_id, state.qr_expire_secs).unwrap_or_default();
    let qr_image_url = if qr_url.is_empty() {
        String::new()
    } else {
        format!(
            "https://api.qrserver.com/v1/create-qr-code/?size=240x240&data={}",
            urlencoding::encode(&qr_url)
        )
    };
    let page = BotRegisterTpl {
        bot_id,
        qr_url,
        qr_image_url,
    };
    let html = page.render().map_err(internal_error)?;
    Ok(Html(html).into_response())
}
