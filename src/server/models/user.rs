use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{error::SealedResult, server::app_state::AppDatabase};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub(crate) async fn find_by_username(
        username: &str,
        db_pool: &AppDatabase,
    ) -> SealedResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE lower(username) = lower($1) limit 1",
        )
        .bind(username)
        .fetch_optional(db_pool.get_pool())
        .await
        .ok()
        .flatten();

        Ok(user)
    }

    pub(crate) async fn find_by_email(
        email: &str,
        db_pool: &AppDatabase,
    ) -> SealedResult<Option<User>> {
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE lower(email) = lower($1) limit 1")
                .bind(email)
                .fetch_optional(db_pool.get_pool())
                .await
                .ok()
                .flatten();

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Sqlite;

    use super::*;

    #[tokio::test]
    async fn test_find_by_username() {
        let sqlitedb = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        let db = AppDatabase::new(sqlitedb).await.unwrap();
        let user = User::find_by_username("testuser", &db).await;
        assert_eq!(user.is_ok(), true);
        assert_eq!(user.unwrap().is_some(), true);
    }
}
