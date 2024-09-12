use tracing::Level;

pub fn setup_tracing() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    env_logger::init();
}
