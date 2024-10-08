use sealed_common::error::SealedError;
use thiserror::Error;

pub type SealedDatabaseResult<T = (), E = SealedDatabaseError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum SealedDatabaseError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    DatabaseMigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("Interrupted")]
    Interrupted,
    #[error("Failed to run command: {0} {1:?}")]
    FailedToRunUserCommand(String, Option<Box<dyn std::error::Error>>),
    #[error("System error: {0} {1:?}")]
    System(String, Option<Box<dyn std::error::Error>>),
}

impl From<SealedDatabaseError> for SealedError {
    fn from(error: SealedDatabaseError) -> Self {
        match error {
            SealedDatabaseError::DatabaseError(e) => SealedError::DatabaseError(e.to_string()),
            SealedDatabaseError::DatabaseMigrationError(e) => {
                SealedError::DatabaseError(e.to_string())
            }
            SealedDatabaseError::Interrupted => SealedError::Interrupted,
            SealedDatabaseError::FailedToRunUserCommand(e, _) => {
                SealedError::FailedToRunUserCommand(e, None)
            }
            SealedDatabaseError::System(e, _) => SealedError::System(e, None),
        }
    }
}
