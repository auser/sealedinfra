use kube::core::gvk::ParseGroupVersionError;
use sealed_common::error::SealedError;

pub type SealedOperatorResult<T = (), E = SealedOperatorError> = Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum SealedOperatorError {
    #[error("Runtime error: {0}")]
    Runtime(#[from] anyhow::Error),

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
}
