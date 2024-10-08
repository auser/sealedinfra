use sealed_common::error::SealedError;
use thiserror::Error;

pub type SealedCliResult<T = (), E = SealedCliError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum SealedCliError {
    #[error("Unable to initialize config: {0}")]
    InitConfig(String),
    #[error("Unable to parse config: {0}")]
    ParseConfig(String),
    #[error("Unable to run command: {0}")]
    Command(#[from] clap::error::Error),
    #[error("Unable to run terraform: {0}")]
    Terraform(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

impl From<SealedCliError> for SealedError {
    fn from(error: SealedCliError) -> Self {
        match error {
            SealedCliError::InitConfig(e) => SealedError::Cli(e),
            SealedCliError::ParseConfig(e) => SealedError::Cli(e),
            SealedCliError::Command(e) => SealedError::Cli(e.to_string()),
            SealedCliError::Terraform(e) => SealedError::Cli(e),
            SealedCliError::Runtime(e) => SealedError::Cli(e),
        }
    }
}

impl From<SealedError> for SealedCliError {
    fn from(error: SealedError) -> Self {
        SealedCliError::Terraform(error.to_string())
    }
}

impl From<sealed_operator::error::SealedOperatorError> for SealedCliError {
    fn from(error: sealed_operator::error::SealedOperatorError) -> Self {
        SealedCliError::Runtime(error.to_string())
    }
}

impl From<anyhow::Error> for SealedCliError {
    fn from(error: anyhow::Error) -> Self {
        SealedCliError::Runtime(error.to_string())
    }
}

impl From<sealed_services::error::SealedServicesError> for SealedCliError {
    fn from(error: sealed_services::error::SealedServicesError) -> Self {
        SealedCliError::Runtime(error.to_string())
    }
}

impl From<std::boxed::Box<dyn std::error::Error>> for SealedCliError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        SealedCliError::Runtime(error.to_string())
    }
}
