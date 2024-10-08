use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
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
