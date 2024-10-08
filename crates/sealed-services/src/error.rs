use sealed_common::error::SealedError;
use thiserror::Error;

pub type SealedServicesResult<T = (), E = SealedServicesError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum SealedServicesError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Database migration error: {0}")]
    DatabaseMigrationError(String),

    #[error("Interrupted")]
    Interrupted,
    #[error("Failed to run command: {0} {1:?}")]
    FailedToRunUserCommand(String, Option<Box<dyn std::error::Error>>),
    #[error("System error: {0} {1:?}")]
    System(String, Option<Box<dyn std::error::Error>>),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Runtime error: {0}")]
    RuntimeError(#[from] anyhow::Error),

    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),
}

impl From<SealedServicesError> for SealedError {
    fn from(error: SealedServicesError) -> Self {
        match error {
            SealedServicesError::DatabaseError(e) => SealedError::DatabaseError(e.to_string()),
            SealedServicesError::DatabaseMigrationError(e) => {
                SealedError::DatabaseError(e.to_string())
            }
            SealedServicesError::Interrupted => SealedError::Interrupted,
            SealedServicesError::FailedToRunUserCommand(e, _) => {
                SealedError::FailedToRunUserCommand(e, None)
            }
            SealedServicesError::System(e, _) => SealedError::System(e, None),
            SealedServicesError::IOError(e) => SealedError::IOError(e),
            SealedServicesError::RuntimeError(e) => SealedError::Runtime(anyhow::anyhow!(e)),
            SealedServicesError::GitError(e) => SealedError::GitOperationFailed(e.to_string()),
        }
    }
}

impl From<sealed_common::error::SealedError> for SealedServicesError {
    fn from(error: SealedError) -> Self {
        match error {
            SealedError::DatabaseError(e) => SealedServicesError::DatabaseError(e),
            SealedError::Interrupted => SealedServicesError::Interrupted,
            SealedError::FailedToRunUserCommand(e, _) => {
                SealedServicesError::FailedToRunUserCommand(e, None)
            }
            SealedError::System(e, _) => SealedServicesError::System(e, None),
            SealedError::IOError(e) => SealedServicesError::IOError(e),
            SealedError::Runtime(e) => SealedServicesError::RuntimeError(anyhow::anyhow!(e)),
            _ => SealedServicesError::RuntimeError(anyhow::anyhow!("unknown error")),
        }
    }
}
