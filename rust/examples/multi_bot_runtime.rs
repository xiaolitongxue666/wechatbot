//! Run from `rust/`: `cargo run --example multi_bot_runtime`

use wechatbot::{infra::logging::init_tracing, AppConfig, MultiBotRuntime};

#[tokio::main]
async fn main() {
    init_tracing("multi_bot_runtime");
    let config = AppConfig::load_from_file("config/app.toml")
        .await
        .expect("load config failed");
    let runtime = MultiBotRuntime::from_config(config)
        .await
        .expect("init runtime failed");

    runtime
        .create_bot(
            "demo-bot",
            Box::new(|url| println!("QR_URL={url}")),
        )
        .await
        .expect("create bot failed");

    runtime
        .forwarder
        .run_forever()
        .await
        .expect("forwarder worker stopped unexpectedly");
}
