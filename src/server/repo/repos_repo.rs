use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{error::SealedResult, server::app_state::AppDatabase};

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
#[allow(non_snake_case)]
pub struct FpRepo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub visibility: String,
    pub license: Option<String>,
    pub forked_from: Option<String>,
    pub mirrored_from: Option<String>,
    pub archived: Option<bool>,
    pub disabled: Option<bool>,
    pub owner: i32,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[allow(non_snake_case)]
pub struct CreateRepoRequest {
    pub name: String,
    pub description: Option<String>,
    pub visibility: String,
    pub license: Option<String>,
    pub forked_from: Option<String>,
    pub mirrored_from: Option<String>,
    pub archived: Option<bool>,
    pub disabled: Option<bool>,
    pub owner: i32,
}

#[allow(unused)]
pub async fn create_repo(db: &AppDatabase, repo: CreateRepoRequest) -> SealedResult<FpRepo> {
    let new_repo = sqlx::query_as::<_, FpRepo>(
        r#"INSERT INTO 
            repositories 
            (name, description, visibility, license, forked_from, mirrored_from, archived, disabled, owner)
            VALUES 
            ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
            RETURNING *"#,
    )
    .bind(repo.name)
    .bind(repo.description)
    .bind(repo.visibility)
    .bind(repo.license)
    .bind(repo.forked_from)
    .bind(repo.mirrored_from)
    .bind(repo.archived)
    .bind(repo.disabled)
    .bind(repo.owner)
    .fetch_one(db.get_pool())
    .await?;

    Ok(new_repo)
}
