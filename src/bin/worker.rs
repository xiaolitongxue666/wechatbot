use std::path::PathBuf;
use wechatbot::{infra::logging::init_tracing, AppConfig, MultiBotRuntime};

#[tokio::main]
async fn main() {
    init_tracing("forwarder_worker");

    let config_path = std::env::var("WECHATBOT_CONFIG").unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("config/app.toml")
            .to_string_lossy()
            .into_owned()
    });
    let config = match AppConfig::load_from_file(&config_path).await {
        Ok(c) => c,
        Err(error) => {
            eprintln!("load config {config_path}: {error}");
            std::process::exit(1);
        }
    };

    let runtime = match MultiBotRuntime::from_config(config).await {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("init runtime failed: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = runtime.forwarder.run_forever().await {
        eprintln!("forwarder worker stopped: {error}");
        std::process::exit(1);
    }
}
