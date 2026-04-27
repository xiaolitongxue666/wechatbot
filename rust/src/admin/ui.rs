//! UI preferences: language, theme, cookie/query merging.

use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

const COOKIE_LANG: &str = "wechatbot_admin_lang";
const COOKIE_THEME: &str = "wechatbot_admin_theme";

#[derive(Debug, Clone, Default)]
pub struct UiPrefs {
    pub lang: String,
    pub theme: String,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct UiQuery {
    pub lang: Option<String>,
    pub theme: Option<String>,
}

impl UiPrefs {
    pub fn resolve(query: UiQuery, jar: &CookieJar) -> Self {
        let lang_from_cookie = jar
            .get(COOKIE_LANG)
            .map(|c| c.value().trim().to_string())
            .filter(|v| !v.is_empty());
        let theme_from_cookie = jar
            .get(COOKIE_THEME)
            .map(|c| c.value().trim().to_string())
            .filter(|v| !v.is_empty());

        let lang = query
            .lang
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or(lang_from_cookie)
            .unwrap_or_else(|| "zh".to_string());

        let lang = if lang == "en" { "en".to_string() } else { "zh".to_string() };

        let theme = query
            .theme
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or(theme_from_cookie)
            .unwrap_or_else(|| "dark".to_string());

        let theme = if theme == "light" {
            "light".to_string()
        } else {
            "dark".to_string()
        };

        Self { lang, theme }
    }

    /// Returns updated jar if query changed prefs (Set-Cookie on response).
    pub fn apply_cookies_from_query(query: &UiQuery, jar: CookieJar) -> CookieJar {
        let mut jar = jar;
        if let Some(ref raw) = query.lang {
            let v = raw.trim();
            if v == "en" || v == "zh" {
                jar = jar.add(build_pref_cookie(COOKIE_LANG, v));
            }
        }
        if let Some(ref raw) = query.theme {
            let v = raw.trim();
            if v == "light" || v == "dark" {
                jar = jar.add(build_pref_cookie(COOKIE_THEME, v));
            }
        }
        jar
    }

    pub fn query_suffix(&self) -> String {
        format!("lang={}&theme={}", self.lang, self.theme)
    }
}

fn build_pref_cookie(name: &'static str, value: &str) -> Cookie<'static> {
    Cookie::build((name, value.to_string()))
        .path("/")
        .http_only(false)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(365))
        .build()
}

#[derive(Clone, Copy)]
pub struct I18n {
    pub nav_dashboard: &'static str,
    pub nav_bots: &'static str,
    pub nav_create_bot: &'static str,
    pub overview_total_bots: &'static str,
    pub overview_online_bots: &'static str,
    pub overview_last_heartbeat: &'static str,
    pub overview_messages_today: &'static str,
    pub overview_dlq: &'static str,
    pub overview_forward_pending: &'static str,
    pub col_session_id: &'static str,
    pub col_tenant: &'static str,
    pub col_owner: &'static str,
    pub col_status: &'static str,
    pub col_heartbeat: &'static str,
    pub col_updated: &'static str,
    pub action_detail: &'static str,
    pub action_history: &'static str,
    pub theme_label: &'static str,
    pub lang_label: &'static str,
    pub theme_dark: &'static str,
    pub theme_light: &'static str,
    pub lang_zh: &'static str,
    pub lang_en: &'static str,
    pub page_dashboard: &'static str,
    pub page_bot_list: &'static str,
    pub page_bot_detail: &'static str,
    pub page_bot_create: &'static str,
    pub page_history: &'static str,
    pub history_time: &'static str,
    pub history_from: &'static str,
    pub history_to: &'static str,
    pub history_type: &'static str,
    pub history_text: &'static str,
    pub pagination_prev: &'static str,
    pub pagination_next: &'static str,
    pub no_data: &'static str,
    pub bot_not_found: &'static str,
    pub form_tenant_id: &'static str,
    pub form_owner_id: &'static str,
    pub form_session_id: &'static str,
    pub btn_create: &'static str,
    pub btn_start: &'static str,
    pub btn_stop: &'static str,
    pub btn_restart: &'static str,
    pub action_create: &'static str,
    pub qr_scan_prompt: &'static str,
    pub qr_waiting: &'static str,
    pub action_new_bot: &'static str,
    pub status_online: &'static str,
    pub status_offline: &'static str,
    pub status_unknown: &'static str,
    pub no_runtime: &'static str,
    pub bot_created: &'static str,
    pub bot_started: &'static str,
    pub bot_stopped: &'static str,
}

const ZH: I18n = I18n {
    nav_dashboard: "仪表盘",
    nav_bots: "Bot 列表",
    nav_create_bot: "创建 Bot",
    overview_total_bots: "Bot 总数",
    overview_online_bots: "在线（估算）",
    overview_last_heartbeat: "最近心跳",
    overview_messages_today: "今日消息",
    overview_dlq: "转发 DLQ",
    overview_forward_pending: "转发未成功",
    col_session_id: "会话 ID",
    col_tenant: "租户",
    col_owner: "属主",
    col_status: "状态",
    col_heartbeat: "心跳",
    col_updated: "更新时间",
    action_detail: "详情",
    action_history: "对话历史",
    theme_label: "主题",
    lang_label: "语言",
    theme_dark: "暗黑",
    theme_light: "亮色",
    lang_zh: "中文",
    lang_en: "English",
    page_dashboard: "管理后台 · 仪表盘",
    page_bot_list: "Bot 列表",
    page_bot_detail: "Bot 详情",
    page_bot_create: "创建 Bot",
    page_history: "对话历史",
    history_time: "时间",
    history_from: "发送方",
    history_to: "接收方",
    history_type: "类型",
    history_text: "文本",
    pagination_prev: "上一页",
    pagination_next: "下一页",
    no_data: "暂无数据",
    bot_not_found: "未找到该 Bot",
    form_tenant_id: "租户 ID",
    form_owner_id: "属主 ID",
    form_session_id: "会话 ID",
    btn_create: "创建",
    btn_start: "启动",
    btn_stop: "停止",
    btn_restart: "重启",
    action_create: "创建 Bot",
    qr_scan_prompt: "请用微信扫描下方二维码，或直接打开链接",
    qr_waiting: "等待二维码生成...",
    action_new_bot: "新建 Bot",
    status_online: "在线",
    status_offline: "离线",
    status_unknown: "未知",
    no_runtime: "Bot 运行时未初始化，无法执行此操作",
    bot_created: "Bot 已创建，即将跳转到详情页...",
    bot_started: "Bot 已启动",
    bot_stopped: "Bot 已停止",
};

const EN: I18n = I18n {
    nav_dashboard: "Dashboard",
    nav_bots: "Bots",
    nav_create_bot: "New Bot",
    overview_total_bots: "Total bots",
    overview_online_bots: "Online (est.)",
    overview_last_heartbeat: "Last heartbeat",
    overview_messages_today: "Messages today",
    overview_dlq: "Forward DLQ",
    overview_forward_pending: "Forward not success",
    col_session_id: "Session ID",
    col_tenant: "Tenant",
    col_owner: "Owner",
    col_status: "Status",
    col_heartbeat: "Heartbeat",
    col_updated: "Updated",
    action_detail: "Detail",
    action_history: "History",
    theme_label: "Theme",
    lang_label: "Language",
    theme_dark: "Dark",
    theme_light: "Light",
    lang_zh: "中文",
    lang_en: "English",
    page_dashboard: "Admin · Dashboard",
    page_bot_list: "Bot list",
    page_bot_detail: "Bot detail",
    page_bot_create: "Create Bot",
    page_history: "Conversation history",
    history_time: "Time",
    history_from: "From",
    history_to: "To",
    history_type: "Type",
    history_text: "Text",
    pagination_prev: "Previous",
    pagination_next: "Next",
    no_data: "No data",
    bot_not_found: "Bot not found",
    form_tenant_id: "Tenant ID",
    form_owner_id: "Owner ID",
    form_session_id: "Session ID",
    btn_create: "Create",
    btn_start: "Start",
    btn_stop: "Stop",
    btn_restart: "Restart",
    action_create: "Create Bot",
    qr_scan_prompt: "Scan QR code with WeChat, or open the link directly",
    qr_waiting: "Waiting for QR code...",
    action_new_bot: "New Bot",
    status_online: "Online",
    status_offline: "Offline",
    status_unknown: "Unknown",
    no_runtime: "Bot runtime not initialized, cannot perform this action",
    bot_created: "Bot created, redirecting to detail...",
    bot_started: "Bot started",
    bot_stopped: "Bot stopped",
};

pub fn i18n(lang: &str) -> I18n {
    if lang == "en" {
        EN
    } else {
        ZH
    }
}

#[cfg(test)]
mod tests {
    use super::{UiPrefs, UiQuery};
    use axum_extra::extract::cookie::CookieJar;

    #[test]
    fn prefs_default_zh_dark() {
        let p = UiPrefs::resolve(UiQuery::default(), &CookieJar::new());
        assert_eq!(p.lang, "zh");
        assert_eq!(p.theme, "dark");
    }

    #[test]
    fn prefs_query_overrides() {
        let p = UiPrefs::resolve(
            UiQuery {
                lang: Some("en".into()),
                theme: Some("light".into()),
            },
            &CookieJar::new(),
        );
        assert_eq!(p.lang, "en");
        assert_eq!(p.theme, "light");
    }
}
