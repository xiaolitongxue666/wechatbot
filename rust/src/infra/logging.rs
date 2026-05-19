use tracing_subscriber::EnvFilter;
use std::io::IsTerminal;

const DEFAULT_LOG_FILTER: &str = "info,wechatbot=info,sqlx=warn,reqwest=warn,hyper=warn";

pub fn init_tracing(service_name: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(DEFAULT_LOG_FILTER))
        .expect("default log filter must be valid");

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(std::io::stderr().is_terminal())
        .try_init();

    tracing::info!(service = service_name, "tracing initialized");
}

#[cfg(test)]
mod tests {
    use super::DEFAULT_LOG_FILTER;
    use tracing_subscriber::EnvFilter;

    #[test]
    fn default_filter_is_valid() {
        let parsed = EnvFilter::try_new(DEFAULT_LOG_FILTER);
        assert!(parsed.is_ok(), "invalid default filter: {DEFAULT_LOG_FILTER}");
    }
}
