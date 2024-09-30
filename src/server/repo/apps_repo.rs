use serde::{Deserialize, Serialize};

use crate::{
    error::SealedResult,
    server::{app_state::AppDatabase, utils::schema::Pagination},
};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
#[allow(non_snake_case)]
pub struct FpApp {
    #[serde(skip_deserializing)]
    pub id: i64,
    pub name: String,
    pub description: String,
    pub app_config: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub enum FpAppTaskStatus {
    Pending,
    InProgress,
    Completed,
}

pub async fn get_apps(db: &AppDatabase, pagination: Pagination) -> SealedResult<Vec<FpApp>> {
    let limit = pagination.limit;
    let offset = pagination.offset;

    let apps = sqlx::query_as::<_, FpApp>(r#"SELECT * FROM apps ORDER BY id LIMIT $1 OFFSET $2"#)
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(db.get_pool())
        .await?;

    Ok(apps)
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[allow(non_snake_case)]
pub struct CreateAppRequest {
    pub name: String,
    pub description: Option<String>,
    pub app_config: Option<serde_json::Value>,
}

pub async fn create_app(db: &AppDatabase, app: CreateAppRequest) -> SealedResult<FpApp> {
    let new_app = sqlx::query_as::<_, FpApp>(
        r#"INSERT INTO apps (name, description, app_config) VALUES ($1, $2, $3) RETURNING *"#,
    )
    .bind(app.name)
    .bind(app.description.unwrap_or("".to_string()))
    .bind(app.app_config)
    .fetch_one(db.get_pool())
    .await?;

    Ok(new_app)
}
