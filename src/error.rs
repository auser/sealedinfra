#[cfg(feature = "server")]
use axum::{http::StatusCode, Json};
use kube::core::gvk::ParseGroupVersionError;
use serde_json::Value;

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
    #[cfg(feature = "server")]
    #[error("Git2 error: {0}")]
    Git2(#[from] git2::Error),
    #[error("Git operation failed: {0}")]
    GitOperationFailed(String),

    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    Kube {
        #[from]
        source: kube::Error,
    },
    /// Error in user input or Bionic resource definition, typically missing fields.
    //#[error("Invalid Bionic CRD: {0}")]
    //UserInput(String),
    #[error("Invalid Kubernetes Json: {source}")]
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
    DatabaseError(#[from] sqlx::Error),
    #[error("Unsupported project type")]
    UnsupportedProjectType,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("No data")]
    NoData,
}

#[cfg(feature = "server")]
impl From<axum::http::StatusCode> for SealedError {
    fn from(status: axum::http::StatusCode) -> Self {
        SealedError::ServerError(status.to_string())
    }
}

#[cfg(feature = "server")]
impl From<(StatusCode, Json<Value>)> for SealedError {
    fn from(status: (StatusCode, Json<Value>)) -> Self {
        // TODO: include StatusCode in the error message
        SealedError::ServerError(status.1.to_string())
    }
}

// Assuming SealedError is defined somewhere in your project, add this implementation:
impl From<Box<dyn std::error::Error>> for SealedError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        SealedError::Runtime(anyhow::anyhow!("{:#?}", err))
    }
}
