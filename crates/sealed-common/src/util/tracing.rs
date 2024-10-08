use std::str::FromStr;
use tracing::{level_filters::LevelFilter, Level};

pub async fn setup_tracing(level: Option<LevelFilter>) {
    let level = level.unwrap_or(LevelFilter::INFO);
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    env_logger::init();
}

pub async fn init_tracing_from_env() {
    let level = std::env::var("RUST_LOG").unwrap_or("warn".to_string());
    let level = Level::from_str(&level).unwrap_or(Level::INFO);
    setup_tracing(Some(LevelFilter::from(level))).await;
}
