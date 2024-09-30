use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;

use crate::{
    error::SealedResult,
    server::{
        app_state::SharedAppState,
        repo::apps_repo::{self},
        utils::{
            schema::{Pagination, PaginationParams},
            server_utils::handle_error,
        },
    },
};

pub fn routes(app_state: SharedAppState) -> Router<SharedAppState> {
    Router::new()
        .route("/", post(create_new_app))
        .route("/", get(list_apps))
        .with_state(app_state)
}

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "FPApps API",
        version = "0.1.0",
        description = "API for FPApps"
    ),
    paths(
        list_apps,
        create_new_app
    ),
    components(
        schemas(
            apps_repo::FpApp,
            apps_repo::CreateAppRequest,
        )
    ),
    tags(
        (name = "Apps", description = "Infrastructure related endpoints")
    )
)]
pub struct AppsOpenApi;

#[utoipa::path(
    tag = "Get all apps",
    get,
    path = "/api/apps",
    operation_id = "list_apps",
    responses(
        (status = 200, description = "List of apps", body = [FpApp], content_type = "application/json")
    ),
    // params(
    //     ("limit" = u32, description = "Limit the number of apps returned"),
    //     ("offset" = u32, description = "Offset the number of apps returned"),
    // )
)]
pub async fn list_apps(
    Query(opts): Query<PaginationParams>,
    State(state): State<SharedAppState>,
) -> impl IntoResponse {
    let opts = Pagination {
        offset: opts.offset.unwrap_or(0),
        limit: opts.limit.unwrap_or(10),
    };

    match apps_repo::get_apps(&state.db, opts).await {
        Ok(apps) => Ok(Json(apps)),
        Err(err) => Err(handle_error(err)),
    }
}

#[utoipa::path(
    tag = "Create new app",
    post,
    path = "/api/apps",
    request_body = CreateAppRequest,
    responses(
        (status = 201, description = "Create a new app", body = FpApp),
        (status = 500, description = "Internal server error", body = Value)
    ),
)]
pub async fn create_new_app(
    State(state): State<SharedAppState>,
    Json(create_app_request): Json<apps_repo::CreateAppRequest>,
) -> SealedResult<impl IntoResponse, (StatusCode, Json<Value>)> {
    match apps_repo::create_app(&state.db, create_app_request).await {
        Ok(app) => Ok((StatusCode::CREATED, Json(app)).into_response()),
        Err(err) => Err(handle_error(err)),
    }
}
