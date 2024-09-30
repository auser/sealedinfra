use axum::{routing::get, Router};

use crate::server::app_state::SharedAppState;

pub fn routes(app_state: SharedAppState) -> Router<SharedAppState> {
    Router::new()
        .route("/", get(index_handler))
        .with_state(app_state)
}

pub async fn index_handler() -> impl axum::response::IntoResponse {
    "FPApps"
}
