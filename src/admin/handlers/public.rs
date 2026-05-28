use crate::admin::state::AdminState;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};

pub async fn root_redirect() -> Redirect {
    Redirect::temporary("/admin")
}

pub async fn healthz() -> &'static str {
    "ok"
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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
    let escaped_bot_id = escape_html(&bot_id);
    let escaped_qr_url = escape_html(&qr_url);
    let escaped_qr_image_url = escape_html(&qr_image_url);
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Bot Register</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; background: #0b1220; color: #dbe7ff; }}
    .container {{ max-width: 760px; margin: 36px auto; padding: 24px; border-radius: 14px; background: #111c33; }}
    .muted {{ color: #9bb4dd; }}
    .qr {{ margin-top: 20px; }}
    .qr img {{ width: 240px; height: 240px; border-radius: 8px; background: #fff; }}
    code {{ background: #1a2744; padding: 2px 8px; border-radius: 6px; }}
  </style>
</head>
<body>
  <div class="container">
    <h1>Bot Registration</h1>
    <p class="muted">bot_id: <code>{escaped_bot_id}</code></p>
    <p class="muted">请使用微信扫码完成登录。</p>
    <div class="qr">
      {qr_block}
    </div>
  </div>
</body>
</html>"#,
        qr_block = if escaped_qr_image_url.is_empty() {
            "<p class=\"muted\">当前暂无可用二维码，请稍后刷新。</p>".to_string()
        } else {
            format!(
                "<img src=\"{}\" alt=\"bot-register-qr\" /><p class=\"muted\"><a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">打开原始二维码链接</a></p>",
                escaped_qr_image_url, escaped_qr_url
            )
        }
    );
    Ok(Html(html).into_response())
}
