#![allow(unused)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[allow(non_snake_case)]
pub struct GitRepo {
    pub id: i64,
    pub name: String,
    pub owner: i64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GitRequest {
    pub(crate) username: String,
    pub(crate) repository: String,
}
