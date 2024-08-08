pub type SealedResult<T = (), E = SealedError> = Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum SealedError {
    #[error("CLI error: {0}")]
    Cli(#[from] clap::error::Error),
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Runtime error: {0}")]
    Runtime(#[from] anyhow::Error),
    #[error("Command error: {0}")]
    Command(#[from] std::io::Error),
}

// Assuming SealedError is defined somewhere in your project, add this implementation:
impl From<Box<dyn std::error::Error>> for SealedError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        SealedError::Runtime(anyhow::anyhow!("{:#?}", err))
    }
}
