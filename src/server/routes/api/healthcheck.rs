use axum::{response::IntoResponse, routing::get, Json, Router};

use crate::server::app_state::SharedAppState;

pub fn routes(app_state: SharedAppState) -> Router<SharedAppState> {
    Router::new()
        .route("/health", get(health_check_handler))
        .with_state(app_state)
}

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Health API",
        version = "0.1.0",
        description = "API for Healthcheck"
    ),
    paths(
        health_check_handler
    ),
    tags(
        (name = "Healthcheck", description = "Healthcheck related endpoints")
    )
)]
pub struct HealthCheckOpenApi;

#[utoipa::path(
    tag = "Healthcheck",
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "API Services running", body = Value)
    ),
)]
pub async fn health_check_handler() -> impl IntoResponse {
    const MESSAGE: &str = "API Services running";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": MESSAGE
    });

    Json(json_response)
}
