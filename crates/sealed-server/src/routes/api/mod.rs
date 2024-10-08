pub mod apps;

// pub mod git;
pub mod healthcheck;
pub mod webhook;

use std::sync::Arc;

use apps::routes as apps_routes;
use axum::Router;
use healthcheck::routes as healthcheck_routes;
use webhook::routes as webhook_routes;

use crate::app_state::SharedAppState;
// use git::routes as git_routes;

pub fn routes(app_state: SharedAppState) -> Router<SharedAppState> {
    Router::new()
        .nest("/", healthcheck_routes(Arc::clone(&app_state)))
        .nest("/webhook", webhook_routes(Arc::clone(&app_state)))
        .nest("/apps", apps_routes(Arc::clone(&app_state)))
    // .nest("/git", git_routes(Arc::clone(&app_state)))
}
