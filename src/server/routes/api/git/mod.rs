use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde_json::Value;

use crate::{error::SealedResult, server::app_state::SharedAppState};

pub fn routes(app_state: SharedAppState) -> Router<SharedAppState> {
    Router::new()
        .route(
            "/git/:repo_name.git/git-receive-pack",
            post(handle_git_request),
        )
        .with_state(app_state)
}

async fn handle_git_request(
    State(state): State<SharedAppState>,
    Path(repo_name): Path<String>,
) -> SealedResult<impl IntoResponse, (StatusCode, Json<Value>)> {
    Ok(())
}
