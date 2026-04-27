use std::path::PathBuf;
use wechatbot::{run_admin_server, AppConfig};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config_path = std::env::var("WECHATBOT_CONFIG").unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("config/app.toml")
            .to_string_lossy()
            .into_owned()
    });
    let config = match AppConfig::load_from_file(&config_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("load config {config_path}: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = run_admin_server(config).await {
        eprintln!("admin server: {e}");
        std::process::exit(1);
    }
}
