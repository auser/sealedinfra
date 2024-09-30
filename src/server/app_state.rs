use std::{sync::Arc, time::Duration};

use crate::error::SealedResult;

pub type SharedAppState = Arc<AppState>;

#[derive(Debug, Clone)]
pub struct AppDatabase {
    #[cfg(feature = "sqlite")]
    pub db: Pool<sqlx::sqlite::SqlitePool>,
    #[cfg(feature = "postgres")]
    pub db: sqlx::postgres::PgPool,
}

impl AppDatabase {
    #[cfg(feature = "sqlite")]
    pub async fn new(db: sqlx::Pool<sqlx::Sqlite>) -> SealedResult<Self> {
        let db = Self { db };
        db.run_migrations().await?;
        Ok(db)
    }
    #[cfg(feature = "postgres")]
    pub async fn new(db: sqlx::postgres::PgPool) -> SealedResult<Self> {
        let db = Self { db };
        db.run_migrations().await?;
        Ok(db)
    }

    #[cfg(feature = "sqlite")]
    pub fn get_pool(&self) -> &Pool<sqlx::Sqlite> {
        &self.db
    }
    #[cfg(feature = "postgres")]
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
        #[cfg(feature = "sqlite")]
        let db = {
            let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must set");
            let path = if database_url.starts_with("sqlite://") {
                let path = database_url.strip_prefix("sqlite://").unwrap();
                let dir = std::path::Path::new(path).parent().unwrap();
                if !dir.exists() {
                    std::fs::create_dir_all(dir).expect("Failed to create directory");
                }
                path
            } else {
                "sqlite::memory:"
            };

            println!("Connecting to database: {}", path);
            let options = SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true);

            let db = match SqlitePoolOptions::default()
                .max_connections(10)
                .acquire_timeout(Duration::from_secs(5))
                .connect_with(options)
                .await
            {
                Ok(db) => db,
                Err(e) => {
                    println!("Error connecting to database: {}", e);
                    std::process::exit(1);
                }
            };
            AppDatabase::new(db).await?
        };
        #[cfg(feature = "postgres")]
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
