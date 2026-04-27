use crate::admin::repository::{AdminOverview, BotSessionRow};
use crate::admin::state::AdminState;
use crate::admin::ui::{i18n, I18n, UiPrefs, UiQuery};
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Json;
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
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
    session_id: String,
    tenant_id: String,
    owner_id: String,
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
}

pub async fn bot_list(
    State(state): State<AdminState>,
    Query(q): Query<UiQuery>,
    jar: CookieJar,
) -> Result<Response, Response> {
    let prefs = UiPrefs::resolve(q.clone(), &jar);
    let jar = UiPrefs::apply_cookies_from_query(&q, jar);
    let rows_db = state.repo.list_sessions().await.map_err(internal_error)?;
    let rows: Vec<BotListRowVm> = rows_db
        .into_iter()
        .map(|row| BotListRowVm {
            session_id: row.session_id,
            tenant_id: row.tenant_id,
            owner_id: row.owner_id,
            status: row.status,
            last_heartbeat: format_dt_opt(&row.last_heartbeat_at),
            updated_at: format_dt(row.updated_at),
        })
        .collect();
    let i = i18n(&prefs.lang);
    let page = BotListTpl {
        i18n: i,
        prefs: &prefs,
        lang_attr: if prefs.lang == "en" { "en" } else { "zh-CN" },
        nav_active: "bots",
        rows: &rows,
    };
    let html = page.render().map_err(internal_error)?;
    Ok((jar, Html(html)).into_response())
}

#[derive(Template)]
#[template(path = "admin/bot_detail.html")]
struct BotDetailTpl<'a> {
    i18n: I18n,
    prefs: &'a UiPrefs,
    lang_attr: &'a str,
    nav_active: &'a str,
    bot: &'a BotSessionRow,
    wx_user_display: String,
    heartbeat_display: String,
    created_display: String,
    updated_display: String,
}

pub async fn bot_detail(
    State(state): State<AdminState>,
    Path(session_id): Path<String>,
    Query(q): Query<UiQuery>,
    jar: CookieJar,
) -> Result<Response, Response> {
    let prefs = UiPrefs::resolve(q.clone(), &jar);
    let jar = UiPrefs::apply_cookies_from_query(&q, jar);
    let bot = state
        .repo
        .get_session(&session_id)
        .await
        .map_err(internal_error)?;
    let Some(bot) = bot else {
        let i = i18n(&prefs.lang);
        return Ok((
            jar,
            (StatusCode::NOT_FOUND, Html(not_found_html(&i, &prefs))),
        )
            .into_response());
    };
    let i = i18n(&prefs.lang);
    let wx_user_display = bot
        .wx_user_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("—")
        .to_string();
    let page = BotDetailTpl {
        i18n: i,
        prefs: &prefs,
        lang_attr: if prefs.lang == "en" { "en" } else { "zh-CN" },
        nav_active: "bots",
        bot: &bot,
        wx_user_display,
        heartbeat_display: format_dt_opt(&bot.last_heartbeat_at),
        created_display: format_dt(bot.created_at),
        updated_display: format_dt(bot.updated_at),
    };
    let html = page.render().map_err(internal_error)?;
    Ok((jar, Html(html)).into_response())
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
    let exists = state
        .repo
        .get_session(&session_id)
        .await
        .map_err(internal_error)?;
    if exists.is_none() {
        let i = i18n(&prefs.lang);
        return Ok((
            jar,
            (StatusCode::NOT_FOUND, Html(not_found_html(&i, &prefs))),
        )
            .into_response());
    }

    const PAGE_SIZE: u64 = 30;
    let page = hq.page.max(1);
    let (rows_db, total) = state
        .repo
        .list_messages_page(&session_id, page, PAGE_SIZE)
        .await
        .map_err(internal_error)?;
    let rows: Vec<ChatMessageVm> = rows_db
        .into_iter()
        .map(|m| ChatMessageVm {
            received_at: format_dt(m.received_at),
            from_user_id: m.from_user_id,
            to_user_id: m.to_user_id,
            content_type: m.content_type,
            text_content: m.text_content,
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
