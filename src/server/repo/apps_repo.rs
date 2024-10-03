use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    error::SealedResult,
    server::{app_state::AppDatabase, utils::schema::Pagination},
};

use super::DateWithTimeZone;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
#[allow(non_snake_case)]
pub struct FpApp {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub app_config: Option<serde_json::Value>,
    // pub created_at: DateWithTimeZone,
    // pub updated_at: DateWithTimeZone,
    pub repository_url: Option<String>,
    pub branch: Option<String>,
    pub image: Option<String>,
    pub tag: Option<String>,
}

#[allow(unused)]
pub enum FpAppTaskStatus {
    Pending,
    InProgress,
    Completed,
}

pub async fn get_apps(db: &AppDatabase, pagination: Pagination) -> SealedResult<Vec<FpApp>> {
    let limit = pagination.limit;
    let offset = pagination.offset;

    let apps = sqlx::query_as::<_, FpApp>(
        r#"
    SELECT * FROM 
    apps ORDER BY id LIMIT $1 OFFSET $2"#,
    )
    .bind(limit as i32)
    .bind(offset as i32)
    .fetch_all(db.get_pool())
    .await?;

    Ok(apps)
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[allow(non_snake_case)]
pub struct CreateAppRequest {
    /// Optional name of the app
    #[serde(default)]
    pub name: Option<String>,
    /// Optional description of the app
    #[serde(default)]
    pub description: Option<String>,
    /// Optional app config
    #[serde(default)]
    pub app_config: Option<serde_json::Value>,
    /// Optional repository url
    pub repository_url: Option<String>,
    /// Optional branch
    pub branch: Option<String>,
    /// Optional image
    pub image: Option<String>,
    /// Optional tag
    pub tag: Option<String>,
    pub created_at: DateWithTimeZone,
    pub updated_at: DateWithTimeZone,
}

pub async fn create_app(db: &AppDatabase, app: CreateAppRequest) -> SealedResult<FpApp> {
    let new_app = sqlx::query_as::<_, FpApp>(
        r#"INSERT INTO 
            apps 
            (name, description, app_config, repository_url, branch, image, tag)
            VALUES 
            ($1, $2, $3, $4, $5, $6, $7) 
            RETURNING *"#,
    )
    .bind(app.name.unwrap_or("".to_string()))
    .bind(app.description.unwrap_or("".to_string()))
    .bind(app.app_config)
    .bind(app.repository_url.unwrap_or("".to_string()))
    .bind(app.branch.unwrap_or("".to_string()))
    .bind(app.image.unwrap_or("".to_string()))
    .bind(app.tag.unwrap_or("".to_string()))
    .fetch_one(db.get_pool())
    .await?;

    Ok(new_app)
}
