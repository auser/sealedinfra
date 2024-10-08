use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::IntoParams;

// List
#[derive(Deserialize, Debug, IntoParams)]
#[allow(unused)]
pub struct PaginationParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub struct Pagination {
    pub offset: i64,
    pub limit: i64,
}

// Create
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFpAppSchema {
    pub name: String,
    pub description: String,
    pub app_config: Value,
}

// Update
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateFpAppSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub app_config: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FpAppResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub app_config: Value,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
