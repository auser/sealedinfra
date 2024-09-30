use crate::server::app_state::SharedAppState;
use axum::{response::IntoResponse, routing::get, Json, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::api::{apps::AppsOpenApi, healthcheck::HealthCheckOpenApi};

#[derive(OpenApi)]
#[openapi(info(
    title = "FPApps API",
    version = "0.1.0",
    description = "API for FPApps"
))]
pub struct OpenApiDoc;

pub fn routes(app_state: SharedAppState) -> Router<SharedAppState> {
    // let swagger_ui = SwaggerUi::new("/swagger-ui").url("/openapi.json", doc);

    Router::new()
        .route("/openapi.json", get(openapi_json))
        .merge(SwaggerUi::new("/swagger-ui").url("/docs/openapi.json", OpenApiDoc::openapi()))
        .with_state(app_state)
}

async fn openapi_json() -> impl IntoResponse {
    let mut doc = OpenApiDoc::openapi();
    doc.merge(AppsOpenApi::openapi());
    doc.merge(HealthCheckOpenApi::openapi());
    Json(doc)
}
