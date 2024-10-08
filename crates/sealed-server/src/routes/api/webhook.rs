use axum::{
    routing::{get, post},
    Router,
};

use crate::app_state::SharedAppState;

pub fn routes(shared_app_state: SharedAppState) -> Router<SharedAppState> {
    let router = axum::Router::new();
    let router = router.with_state(shared_app_state);
    router
        .route("/", post(webhook_handler))
        .route("/", get(webhook_get_handler))
}

pub async fn webhook_handler() -> impl axum::response::IntoResponse {
    "Webhook"
}

pub async fn webhook_get_handler() -> impl axum::response::IntoResponse {
    "Webhook GET"
}
