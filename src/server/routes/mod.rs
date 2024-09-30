pub mod api;
pub mod docs;
pub mod frontend;

use axum::Router;

use super::app_state::SharedAppState;

pub fn routes(app_state: SharedAppState) -> Router {
    let router = axum::Router::new();
    router
        .nest("/api", api::routes(app_state.clone()))
        .nest("/", frontend::routes(app_state.clone()))
        .nest("/docs", docs::routes(app_state.clone()))
        .with_state(app_state)
}
