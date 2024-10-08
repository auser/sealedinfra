use kube::core::gvk::ParseGroupVersionError;

pub type SealedResult<T = (), E = SealedError> = Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum SealedError {
    #[error("CLI error: {0}")]
    Cli(String),
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Runtime error: {0}")]
    Runtime(#[from] anyhow::Error),
    #[error("Command error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Parsing error: {0}")]
    Parsing(#[from] ParseGroupVersionError),
    #[error("Timeout error: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("Git operation failed: {0}")]
    GitOperationFailed(String),
    #[error("Git url parse error: {0}")]
    GitUrlParseError(#[from] git_url_parse::GitUrlParseError),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Interrupted")]
    Interrupted,
    #[error("Failed to run command: {0} {1:?}")]
    FailedToRunUserCommand(String, Option<Box<dyn std::error::Error>>),
    #[error("System error: {0} {1:?}")]
    System(String, Option<Box<dyn std::error::Error>>),
    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    Kube {
        #[from]
        source: kube::Error,
    },
    /// Error in user input or Bionic resource definition, typically missing fields.
    //#[error("Invalid Bionic CRD: {0}")]
    //UserInput(String),
    #[error("Invalid Json: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    #[error("Invalid Kubernetes Yaml: {source}")]
    Yaml {
        #[from]
        source: serde_yaml::Error,
    },

    // Server errors
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Unsupported project type")]
    UnsupportedProjectType,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("No data")]
    NoData,
}

// Assuming SealedError is defined somewhere in your project, add this implementation:
impl From<Box<dyn std::error::Error>> for SealedError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        SealedError::Runtime(anyhow::anyhow!("{:#?}", err))
    }
}
