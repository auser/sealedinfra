use std::{sync::Arc, time::Duration};

use sqlx::Pool;

use crate::error::SealedResult;

pub type SharedAppState = Arc<AppState>;

#[derive(Debug, Clone)]
pub struct AppDatabase {
    pub db: sqlx::postgres::PgPool,
}

impl AppDatabase {
    pub async fn new(db: sqlx::postgres::PgPool) -> SealedResult<Self> {
        let db = Self { db };
        db.run_migrations().await?;
        Ok(db)
    }

    pub fn get_pool(&self) -> &sqlx::postgres::PgPool {
        &self.db
    }

    async fn run_migrations(&self) -> SealedResult<()> {
        sqlx::migrate!("./migrations")
            .run(&self.db)
            .await
            .expect("Failed to run migrations");

        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct AppState {
    pub db: AppDatabase,
}

impl AppState {
    pub async fn new() -> SealedResult<Self> {
        let db = {
            let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must set");
            let db = sqlx::postgres::PgPoolOptions::new()
                .max_connections(10)
                .acquire_timeout(Duration::from_secs(5))
                .connect(&database_url)
                .await?;
            AppDatabase::new(db).await?
        };

        Ok(Self { db })
    }
}
