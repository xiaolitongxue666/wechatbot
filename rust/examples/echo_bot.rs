//! Run from `rust/`: `cargo run --example echo_bot`
//!
//! Python 对照（在 `python/`）：`uv run python examples/echo_bot.py`
//! 扫码排查：终端会打印 `QR_URL=...`，与 Python `login(force=True)` 输出对比 host/path/query 是否一致。

use std::sync::Arc;

use tokio::time::{sleep, Duration};
use wechatbot::{BotOptions, WeChatBot};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let bot = Arc::new(WeChatBot::new(BotOptions {
        on_qr_url: Some(Box::new(|url| {
            println!("QR_URL={}", url);
            println!("\nScan this URL in WeChat:\n{}\n", url);
        })),
        on_error: Some(Box::new(|err| {
            eprintln!("Error: {}", err);
        })),
        ..Default::default()
    }));

    // 默认复用本地登录凭据；设置 FORCE_QR=1 可强制重新扫码。
    let force_qr_login = std::env::var("FORCE_QR").ok().as_deref() == Some("1");
    let creds = bot.login(force_qr_login).await.expect("login failed");
    println!("Logged in: {} ({})", creds.account_id, creds.user_id);

    let bot_for_handler = Arc::clone(&bot);
    bot
        .on_message(Box::new(move |msg| {
            println!("[{}] {}: {}", msg.content_type_str(), msg.user_id, msg.text);

            let bot = Arc::clone(&bot_for_handler);
            let msg = msg.clone();
            tokio::spawn(async move {
                let _ = bot.send_typing(&msg.user_id).await;
                sleep(Duration::from_millis(500)).await;
                if let Err(e) = bot.reply(&msg, &format!("Echo: {}", msg.text)).await {
                    eprintln!("Echo reply failed: {}", e);
                }
            });
        }))
        .await;

    println!("Listening for messages (Ctrl+C to stop)");
    bot.run().await.expect("run failed");
}

trait ContentTypeStr {
    fn content_type_str(&self) -> &str;
}

impl ContentTypeStr for wechatbot::IncomingMessage {
    fn content_type_str(&self) -> &str {
        match self.content_type {
            wechatbot::ContentType::Text => "text",
            wechatbot::ContentType::Image => "image",
            wechatbot::ContentType::Voice => "voice",
            wechatbot::ContentType::File => "file",
            wechatbot::ContentType::Video => "video",
        }
    }
}
