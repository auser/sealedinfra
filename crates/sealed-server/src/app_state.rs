use std::sync::Arc;

use sealed_common::error::SealedResult;
use sealed_database::{database::get_app_database, AppDatabase};

pub type SharedAppState = Arc<AppState>;

// #[derive(Debug, Clone)]
// pub struct AppDatabase {
//     pub db: sqlx::postgres::PgPool,
// }

// impl AppDatabase {
//     pub async fn new(db: sqlx::postgres::PgPool) -> SealedResult<Self> {
//         let db = Self { db };
//         db.run_migrations().await?;
//         Ok(db)
//     }

//     pub fn get_pool(&self) -> &sqlx::postgres::PgPool {
//         &self.db
//     }

//     async fn run_migrations(&self) -> SealedResult<()> {
//         if let Err(e) = sqlx::migrate!("./migrations").run(&self.db).await {
//             tracing::error!("Failed to run migrations: {}", e);
//         };

//         Ok(())
//     }
// }
#[derive(Debug, Clone)]
pub struct AppState {
    pub db: AppDatabase,
}

impl AppState {
    pub async fn new() -> SealedResult<Self> {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must set");
        let db = get_app_database(&database_url).await?;

        Ok(Self { db })
    }
}
