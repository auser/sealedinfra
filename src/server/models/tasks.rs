use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
// #[allow(non_snake_case)]
// pub struct FpApp {
//     pub id: i64,
//     pub name: String,
//     pub description: String,
//     pub created_at: chrono::DateTime<chrono::Utc>,
//     pub updated_at: chrono::DateTime<chrono::Utc>,
// }
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
#[allow(non_snake_case)]
pub struct FpAppTasks {
    pub id: i64,
    pub app_id: i64,
    pub task_action: TaskAction,
    pub status: FpAppTaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub enum TaskAction {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub enum FpAppTaskStatus {
    Pending,
    InProgress,
    Completed,
}
