use kube::core::gvk::ParseGroupVersionError;

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
    #[error("Parsing error: {0}")]
    Parsing(#[from] ParseGroupVersionError),
    #[error("Timeout error: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),

    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    Kube {
        #[from]
        source: kube::Error,
    },
    /// Error in user input or Bionic resource definition, typically missing fields.
    //#[error("Invalid Bionic CRD: {0}")]
    //UserInput(String),
    #[error("Invalid Kubernetes Yaml: {source}")]
    Yaml {
        #[from]
        source: serde_json::Error,
    },
}

// Assuming SealedError is defined somewhere in your project, add this implementation:
impl From<Box<dyn std::error::Error>> for SealedError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        SealedError::Runtime(anyhow::anyhow!("{:#?}", err))
    }
}
