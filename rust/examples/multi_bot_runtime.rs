//! Run from `rust/`: `cargo run --example multi_bot_runtime`

use std::sync::Arc;
use wechatbot::{AppConfig, BotOptions, MultiBotRuntime, WeChatBot};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = AppConfig::load_from_file("config/app.toml")
        .await
        .expect("load config failed");
    let runtime = MultiBotRuntime::from_config(config)
        .await
        .expect("init runtime failed");

    let bot = Arc::new(WeChatBot::new(BotOptions {
        on_qr_url: Some(Box::new(|url| println!("QR_URL={url}"))),
        ..Default::default()
    }));
    runtime
        .register_bot("tenant-demo", "owner-demo", "session-demo", bot)
        .await
        .expect("register bot failed");
    runtime
        .start_session("session-demo", false)
        .await
        .expect("start session failed");

    runtime
        .forwarder
        .run_forever()
        .await
        .expect("forwarder worker stopped unexpectedly");
}
