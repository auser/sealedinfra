use std::time::Duration;

use crate::error::{SealedDatabaseError, SealedDatabaseResult};

pub async fn get_app_database(database_url: &str) -> SealedDatabaseResult<AppDatabase> {
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;
    AppDatabase::new(db).await
}

#[derive(Debug, Clone)]
pub struct AppDatabase {
    pub db: sqlx::postgres::PgPool,
}

impl AppDatabase {
    pub async fn new(db: sqlx::postgres::PgPool) -> SealedDatabaseResult<Self> {
        let db = Self { db };
        db.run_migrations().await?;
        Ok(db)
    }

    pub fn get_pool(&self) -> &sqlx::postgres::PgPool {
        &self.db
    }

    async fn run_migrations(&self) -> SealedDatabaseResult<()> {
        // TODO
        if let Err(e) = sqlx::migrate!("../../migrations").run(&self.db).await {
            tracing::error!("Failed to run migrations: {}", e);
        };

        Ok(())
    }

    pub async fn run_migrations_with_dir<'a, S>(&self, dir: S) -> SealedDatabaseResult<()>
    where
        S: AsRef<std::path::Path> + sqlx::migrate::MigrationSource<'a>,
    {
        let migrator = sqlx::migrate::Migrator::new(dir)
            .await
            .map_err(SealedDatabaseError::DatabaseMigrationError)?;

        migrator
            .run(&self.db)
            .await
            .map_err(SealedDatabaseError::DatabaseMigrationError)?;
        // if let Err(e) = sqlx::migrate!(&format!("{}/migrations", dir))
        //     .run(&self.db)
        //     .await
        // {
        //     tracing::error!("Failed to run migrations: {}", e);
        // };

        Ok(())
    }
}
